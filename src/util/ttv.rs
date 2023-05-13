use color_eyre::eyre::{eyre, Context};
use reqwest::Url;
use tracing::instrument;

/// Run a local server and wait for an http request, and return the uri
#[instrument]
pub async fn wait_for_request_uri() -> color_eyre::Result<Url> {
    let addr: std::net::SocketAddr = "127.0.0.1:3000".parse().unwrap();
    log::debug!("Listening {}", addr);
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .wrap_err("Failed to bind port")?;
    // We just wait for the first connection
    log::debug!("Waiting for connection...");
    let (stream, _) = listener
        .accept()
        .await
        .wrap_err("Failed to accept a connection")?;
    log::debug!("Got connection");

    // Use a channel because how else?
    let (sender, mut receiver) = tokio::sync::mpsc::unbounded_channel::<hyper::Uri>();
    // We could use oneshot channel but this service fn must be impl FnMut
    // because there may be multiple requests even on single connection?
    let service = |request: hyper::Request<hyper::Body>| {
        let sender = sender.clone();
        async move {
            sender
                .send(request.uri().clone())
                .wrap_err("Failed to send request")?;
            Ok::<_, color_eyre::Report>(hyper::Response::new(hyper::Body::from(
                "You may now close this tab",
            )))
        }
    };
    // No keepalive so we return immediately
    hyper::server::conn::Http::new()
        .http1_keep_alive(false)
        .http2_keep_alive_interval(None)
        .serve_connection(stream, hyper::service::service_fn(service))
        .await
        .wrap_err("Failed to serve")?;
    let uri = receiver
        .recv()
        .await
        .ok_or_else(|| eyre!("Failed to wait for the request"))?;
    Ok(Url::parse("http://localhost:3000")
        .unwrap()
        .join(uri.path_and_query().unwrap().as_str())?)
}
