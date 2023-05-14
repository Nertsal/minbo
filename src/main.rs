use color_eyre::eyre::{eyre, Context};
use tracing::instrument;

mod client;
mod secret;
mod token;
mod util;
mod websocket;

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

    // Create a channel for Twitch events
    let (sender, receiver) = tokio::sync::mpsc::channel(32);

    // Verify token and setup client
    let mut client = client::Client::new(&secrets, receiver)
        .await
        .wrap_err("Failed to setup client")?;

    // Launch websocket client in a separate thread for EventSub
    let websocket = websocket::WebsocketClient::new(sender);
    let websocket_handle = tokio::spawn(async move { websocket.run().await });

    let channel = client
        .helix
        .get_channel_from_login("Nertsal", &client.token)
        .await?
        .ok_or(eyre!("Channel not found"))?;
    println!("Channel: {:?}", channel);

    let emotes = client
        .helix
        .get_channel_emotes_from_login("Nertsal", &client.token)
        .await?;
    println!("Emotes: {:?}", emotes);

    if websocket_handle.is_finished() {
        // Error occured - probably setup is wrong - report
        websocket_handle
            .await
            .wrap_err("Websocket stopped early")??;
        return Err(eyre!("Websocket stopped early, but no error got returned"));
    }

    // Listen for event from websocket in this thread
    let err = loop {
        if let Err(err) = client.process_events().await {
            break err;
        }
    };

    // Abort websocket handle
    websocket_handle.abort();
    if let Err(err) = (async { websocket_handle.await? }).await {
        log::error!("\nWebsocket error:\n{}", err);
    }

    Err(err)
}
