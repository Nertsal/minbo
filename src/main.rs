use color_eyre::eyre::Context;
use tracing::instrument;

mod app;
mod client;
mod secret;
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
        .with(tui_logger::TuiTracingSubscriberLayer)
        .init();
    Ok(())
}

#[derive(clap::Parser)]
struct Args {
    #[clap(help = "Channel name to connect to")]
    channel_login: String,
}

#[tokio::main]
#[instrument]
async fn main() -> color_eyre::Result<()> {
    // Setup logging and error handling
    tui_logger::set_log_file("log.txt")?;
    install_tracing()?;
    color_eyre::install()?;

    // Parse CLI arguments
    let args: Args = clap::Parser::parse();

    // Load secrets
    let secrets = secret::Secrets::load().wrap_err("when loading secrets")?;

    // Configure the client
    let client = client::TwitchClient::new(&secrets)
        .await
        .wrap_err("when setting up client")?;

    // Start the app
    app::App::new(client, args.channel_login)
        .wrap_err("when setting up app")?
        .run()
        .await
}
