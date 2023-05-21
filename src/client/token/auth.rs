use std::collections::HashMap;

use color_eyre::eyre::{eyre, Context};
use rand::thread_rng;
use reqwest::Url;
use twitch_irc::login::{GetAccessTokenResponse, UserAccessToken};

use crate::util::ttv::wait_for_request_uri;

use super::Scope;

/// Authenticate using authorization code grant flow
/// <https://dev.twitch.tv/docs/authentication/getting-tokens-oauth#authorization-code-grant-flow>
pub async fn authenticate(
    client_id: &str,
    client_secret: &str,
    force_verify: bool,
    scopes: &[Scope],
) -> color_eyre::Result<UserAccessToken> {
    // We will redirect user to this url to retrieve the token
    // This url should be same as specified in the twitch registered app
    let redirect_uri = "http://localhost:3000";

    // From twitch docs:
    // Although optional, you are strongly encouraged to pass a state string to
    // help prevent Cross-Site Request Forgery (CSRF) attacks. The server
    // returns this string to you in your redirect URI (see the state parameter
    // in the fragment portion of the URI). If this string doesn’t match the
    // state string that you passed, ignore the response. The state string
    // should be randomly generated and unique for each OAuth request.
    use rand::distributions::Distribution;
    let state: String = rand::distributions::Alphanumeric
        .sample_iter(thread_rng())
        .take(16)
        .map(|c| c as char)
        .collect();

    let mut authorize_url = Url::parse("https://id.twitch.tv/oauth2/authorize").unwrap();
    {
        // Set up query
        let mut query = authorize_url.query_pairs_mut();
        query.append_pair("client_id", client_id);
        query.append_pair("force_verify", &force_verify.to_string());
        query.append_pair("redirect_uri", redirect_uri);
        query.append_pair("response_type", "code");
        query.append_pair(
            "scope",
            &scopes
                .iter()
                .map(|scope| scope.as_str())
                .collect::<Vec<&str>>()
                .join(" "),
        );
        query.append_pair("state", &state);
    }

    log::info!("Opening {}", authorize_url);
    open::that(authorize_url.as_str()).wrap_err("when opening authorization link")?; // Open browser

    log::debug!("Waiting for the user to be redirected to {}", redirect_uri);
    let redirected_url = wait_for_request_uri()
        .await
        .wrap_err("when requesting authentication")?;
    let query: HashMap<_, _> = redirected_url.query_pairs().collect();

    if **query
        .get("state")
        .ok_or_else(|| eyre!("query has no state"))?
        != state
    {
        panic!("Hey, are you being hacked or something?");
    }
    if let Some(error) = query.get("error") {
        let description = query
            .get("error_description")
            .ok_or_else(|| eyre!("error without description: {error}"))?;
        color_eyre::eyre::bail!("{error}: {description}");
    }
    let code: &str = query
        .get("code")
        .ok_or_else(|| eyre!("query has no code"))?;

    log::debug!("Got the code, getting the token");
    let mut form = HashMap::new();
    form.insert("client_id", client_id);
    form.insert("client_secret", client_secret);
    form.insert("code", code);
    form.insert("grant_type", "authorization_code");
    form.insert("redirect_uri", redirect_uri);

    let response: GetAccessTokenResponse = reqwest::Client::new()
        .post("https://id.twitch.tv/oauth2/token")
        .form(&form)
        .send()
        .await
        .wrap_err("when sending token request")?
        .json()
        .await
        .wrap_err("when parsing token")?;

    Ok(UserAccessToken::from(response))
}
