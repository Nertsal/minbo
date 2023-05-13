use color_eyre::eyre::{eyre, Context};
use tracing::instrument;

mod client;
mod secret;
mod token;
mod util;

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

    // Verify token and setup client
    let client = client::Client::new(&secrets)
        .await
        .wrap_err("Failed to setup client")?;

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

    Ok(())
}
