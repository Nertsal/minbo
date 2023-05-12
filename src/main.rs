use color_eyre::eyre::Context;
use color_eyre::Help;
use serde::Deserialize;
use tracing::instrument;
use twitch_api::helix::HelixClient;
use twitch_api::twitch_oauth2::{AccessToken, UserToken};

fn install_tracing() -> color_eyre::Result<()> {
    use tracing_error::ErrorLayer;
    use tracing_subscriber::prelude::*;
    use tracing_subscriber::{fmt, EnvFilter};

    let fmt_layer = fmt::layer().with_target(false);
    let filter_layer = EnvFilter::try_from_default_env().or_else(|_| EnvFilter::try_new("info"))?;
    tracing_subscriber::registry()
        .with(filter_layer)
        .with(fmt_layer)
        .with(ErrorLayer::default())
        .init();
    Ok(())
}

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

fn read_to_string(path: impl AsRef<std::path::Path>) -> color_eyre::Result<String> {
    let path = path.as_ref();
    let s =
        std::fs::read_to_string(path).wrap_err_with(|| format!("Failed to read from {path:?}"))?;
    Ok(s)
}

/// Reads from a file and attempts to parse its contents from toml format.
fn read_toml<T: serde::de::DeserializeOwned>(
    path: impl AsRef<std::path::Path>,
) -> color_eyre::Result<T> {
    let content = read_to_string(path)?;
    let result = toml::from_str(&content)?;
    Ok(result)
}

/// Loads the secrets from the secrets folder.
#[instrument]
fn load_secrets() -> color_eyre::Result<Secrets> {
    const SECRETS: &str = "secrets";
    let path = std::path::Path::new(SECRETS);

    let login = read_toml(path.join("login.toml")).wrap_err("Failed to load login secret")?;

    Ok(Secrets { login })
}

struct Client {
    helix: HelixClient<'static, reqwest::Client>,
    token: UserToken,
}

// IMPORTANT: Careful not to expose `oauth_token` in the error message.
async fn setup_client(oauth_token: &str) -> color_eyre::Result<Client> {
    // Create the HelixClient, which is used to make requests to the Twitch API
    let helix: HelixClient<reqwest::Client> = HelixClient::default();
    // Create a UserToken, which is used to authenticate requests
    let token = UserToken::from_token(&helix, AccessToken::from(oauth_token))
        .await
        .wrap_err("Failed to verify oauth token")
        .suggestion("Token might be expired: consider acquiring a new one")?;
    Ok(Client { helix, token })
}

#[tokio::main]
#[instrument]
async fn main() -> color_eyre::Result<()> {
    install_tracing()?;
    color_eyre::install()?;

    // Parse CLI arguments
    let _args: Args = clap::Parser::parse();

    // Load secrets
    let secrets = load_secrets().wrap_err("Failed to load secrets")?;

    // Verify token and setup client
    let client = setup_client(&secrets.login.oauth_token)
        .await
        .wrap_err("Failed to setup client")?;

    println!(
        "Channel: {:?}",
        client
            .helix
            .get_channel_from_login(&secrets.login.channel_login, &client.token)
            .await?
    );

    Ok(())
}
