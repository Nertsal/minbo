use color_eyre::eyre::Context;
use tokio::sync::mpsc::UnboundedReceiver;
use tracing::instrument;
use twitch_irc::{
    login::StaticLoginCredentials, message::ServerMessage, SecureTCPTransport, TwitchIRCClient,
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
    // Setup error handling
    install_tracing()?;
    color_eyre::install()?;

    // Parse CLI arguments
    let _args: Args = clap::Parser::parse();

    // Load secrets
    let secrets = secret::Secrets::load().wrap_err("when loading secrets")?;

    // Configure the client
    let mut client = client::Client::new(&secrets).wrap_err("when setting up client")?;

    client
        .twitch
        .join("nertsal".to_string())
        .wrap_err("when joining a channel")?;
    client
        .twitch
        .say("nertsal".to_string(), "Hello".to_string())
        .await
        .wrap_err("when sending a message")?;

    // Event loop
    loop {
        if let Some(message) = client.try_recv().wrap_err("when receiving a message")? {
            process_message(message).wrap_err("when processing a message")?;
        }
    }
}

/// Process an event from Twitch.
fn process_message(message: ServerMessage) -> color_eyre::Result<()> {
    log::debug!("Received message: {:?}", message);
    match message {
        ServerMessage::Notice(notice) => log::info!("Notice: {}", notice.message_text),
        ServerMessage::Privmsg(message) => {
            log::info!("[Chat] {}: {}", message.channel_login, message.message_text)
        }
        ServerMessage::UserNotice(notice) => log::info!("Event: {}", notice.system_message),
        _ => {}
    }
    Ok(())
}
