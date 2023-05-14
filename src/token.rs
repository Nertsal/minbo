use std::io::Write;
use std::str::FromStr;

use color_eyre::eyre::{eyre, Context};
use serde::{Deserialize, Serialize};
use twitch_api::helix::HelixClient;
use twitch_api::twitch_oauth2::tokens::UserTokenBuilder;
use twitch_api::twitch_oauth2::{
    AccessToken, ClientId, ClientSecret, RefreshToken, Scope, TwitchToken, UserToken,
};

const SCOPES: [Scope; 2] = [
    // Scope::ChannelReadRedemptions,
    Scope::ChatEdit, // Send messages to chat
    Scope::ChatRead, // Read chat messages
];

/// Authenticate using authorization code grant flow.
/// <https://dev.twitch.tv/docs/authentication/getting-tokens-oauth#authorization-code-grant-flow>
async fn authenticate(
    helix: &HelixClient<'static, reqwest::Client>,
    client_id: &str,
    client_secret: &str,
    force_verify: bool,
    scopes: &[Scope],
) -> color_eyre::Result<UserToken> {
    // Must match the twitch application's registered redirect url
    let redirect_url = reqwest::Url::parse("http://localhost:3000").unwrap();
    let mut builder = UserTokenBuilder::new(client_id, client_secret, redirect_url.clone())
        .force_verify(force_verify)
        .set_scopes(scopes.to_vec());
    let (authorize_url, _csrf_code) = builder.generate_url();

    // Direct user to `url`
    log::info!("Opening {}", authorize_url);
    open::that(authorize_url.as_str())?;

    // Wait for the redirect
    log::debug!("Waiting for the user to be redirected to {}", redirect_url);
    let redirected_uri = crate::util::ttv::wait_for_request_uri().await?;

    let query: std::collections::HashMap<_, _> = redirected_uri.query_pairs().collect();

    // Check for error
    if let Some(error) = query.get("error") {
        let error_desc = query
            .get("error_description")
            .ok_or_else(|| eyre!("{}: no description", error))?;
        return Err(eyre!("{}: {}", error, error_desc));
    }

    // Get `state` and `code` from the redirected uri
    let state = query
        .get("state")
        .ok_or_else(|| eyre!("State not found in returned query"))?;
    let code = query
        .get("code")
        .ok_or_else(|| eyre!("Code not found in returned query"))?;

    // Get the token
    let token = builder
        .get_user_token(helix, state, code)
        .await
        .wrap_err("Failed to get UserToken")?;
    Ok(token)
}

/// Acquires the token for twitch API.
/// Uses an existing token it possible, otherwise
/// prompts the user to authorize the application.
pub async fn get_user_token(
    secrets: &crate::secret::Secrets,
    helix: &HelixClient<'static, reqwest::Client>,
) -> color_eyre::Result<UserToken> {
    #[derive(Serialize, Deserialize)]
    struct Tokens {
        access_token: String,
        refresh_token: String,
        scope: Vec<Scope>,
    }

    let login = &secrets.client.login_name;
    log::debug!("Getting ttv token for {:?}", login);
    let tokens_file_path = secrets.path.join("tokens").join(format!("{}.toml", login));
    let (tokens, user_token): (Tokens, UserToken) = if tokens_file_path.is_file() {
        log::debug!("Reading existing tokens");
        let tokens: Tokens = crate::util::fs::read_toml(&tokens_file_path)
            .wrap_err("Failed to read an existing token")?;

        let mut access_token =
            AccessToken::from_str(&tokens.access_token).wrap_err("Invalid acccess token")?;
        let mut refresh_token =
            RefreshToken::from_str(&tokens.refresh_token).wrap_err("Invalid refresh token")?;

        let client_id = ClientId::new(secrets.client.client_id.clone());
        let client_secret = ClientSecret::new(secrets.client.client_secret.clone());

        // Validate token
        let validated = match access_token.validate_token(helix).await {
            Ok(token) => token,
            Err(err) => {
                let twitch_api::twitch_oauth2::tokens::errors::ValidationError::NotAuthorized = err else {
                    return Err(err.into());
                };
                // Refresh token
                log::debug!("Token expired, refreshing");
                let (new_access_token, _expires_in, new_refresh_token) = refresh_token
                    .refresh_token(helix, &client_id, &client_secret)
                    .await?; // TODO: prompt user for a new token
                access_token = new_access_token;
                refresh_token =
                    new_refresh_token.ok_or_else(|| eyre!("Did not get a new refresh token"))?;

                // Validate again, this time should be good
                access_token
                    .validate_token(helix)
                    .await
                    .wrap_err("Failed to validate token")?
            }
        };

        let user_token =
            UserToken::new(access_token, Some(refresh_token), validated, client_secret)?;

        (tokens, user_token)
    } else {
        log::info!("Auth not setup, prepare to login as {:?}", login);
        let user_token = authenticate(
            helix,
            &secrets.client.client_id,
            &secrets.client.client_secret,
            true,
            &SCOPES,
        )
        .await
        .wrap_err("Failed to authenticate")?;

        let tokens = Tokens {
            access_token: user_token.access_token.as_str().to_owned(),
            refresh_token: user_token
                .refresh_token
                .as_ref()
                .unwrap()
                .as_str()
                .to_owned(),
            scope: user_token.scopes().to_vec(),
        };

        (tokens, user_token)
    };

    // Save the token
    std::fs::create_dir_all(tokens_file_path.parent().unwrap())
        .wrap_err("Failed to create tokens directory")?;
    let file = std::fs::File::create(&tokens_file_path).wrap_err("Failed to create token file")?;
    let mut writer = std::io::BufWriter::new(file);
    let s = toml::to_string_pretty(&tokens).wrap_err("Failed to serialize token")?;
    write!(writer, "{}", s).wrap_err("Failed to save token")?;

    log::debug!("Token retrieved successfully");

    Ok(user_token)
}
