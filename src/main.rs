use color_eyre::eyre::Context;
use tokio::sync::mpsc::UnboundedReceiver;
use tracing::instrument;
use twitch_irc::{
    login::StaticLoginCredentials, message::ServerMessage, ClientConfig, SecureTCPTransport,
    TwitchIRCClient,
};

mod client;
mod secret;
mod util;

type TwitchClient = TwitchIRCClient<SecureTCPTransport, StaticLoginCredentials>;
type TwitchReceiver = UnboundedReceiver<ServerMessage>;

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

#[tokio::main]
#[instrument]
async fn main() -> color_eyre::Result<()> {
    install_tracing()?;
    color_eyre::install()?;

    // Parse CLI arguments
    let _args: Args = clap::Parser::parse();

    // Load secrets
    let secrets = secret::Secrets::load().wrap_err("Failed to load secrets")?;

    // Setup up Twitch IRC connection
    let config = ClientConfig::new_simple(StaticLoginCredentials::new(
        secrets.client.login_name.clone(),
        Some(secrets.client.oauth_token.clone()),
    ));
    let (irc_receiver, irc_client) = TwitchClient::new(config);

    // Configure the client
    let mut client = client::Client::new(irc_client, irc_receiver)
        .await
        .wrap_err("failed to setup client")?;

    client.process_events().await?;

    client.twitch.join("nertsal".to_string())?;
    client
        .twitch
        .say("nertsal".to_string(), "Hello".to_string())
        .await?;

    loop {
        client.process_events().await?;
    }
}
