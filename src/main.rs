use serde::Deserialize;
use twitch_api::helix::HelixClient;
use twitch_api::twitch_oauth2::{AccessToken, UserToken};

#[derive(clap::Parser)]
struct Args {}

struct Secrets {
    login: SecretLogin,
}

type LoginName = String;

#[derive(Deserialize)]
struct SecretLogin {
    // login_name: LoginName,
    oauth_token: String,
    channel_login: LoginName,
}

/// Reads from a file and attempts to parse its contents from toml format.
fn read_toml<T: serde::de::DeserializeOwned>(
    path: impl AsRef<std::path::Path>,
) -> color_eyre::Result<T> {
    let content = std::fs::read_to_string(path)?;
    let result = toml::from_str(&content)?;
    Ok(result)
}

/// Loads the secrets from the secrets folder.
fn load_secrets() -> color_eyre::Result<Secrets> {
    const SECRETS: &str = "secrets";
    let path = std::path::Path::new(SECRETS);

    let login = read_toml(path.join("login.toml"))?;

    Ok(Secrets { login })
}

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    // Parse CLI arguments
    let _args: Args = clap::Parser::parse();

    // Load secrets
    let secrets = load_secrets()?;

    // Create the HelixClient, which is used to make requests to the Twitch API
    let client: HelixClient<reqwest::Client> = HelixClient::default();
    // Create a UserToken, which is used to authenticate requests
    let token = UserToken::from_token(
        &client,
        AccessToken::from(secrets.login.oauth_token.as_str()),
    )
    .await?;

    println!(
        "Channel: {:?}",
        client
            .get_channel_from_login(&secrets.login.channel_login, &token)
            .await?
    );

    Ok(())
}
