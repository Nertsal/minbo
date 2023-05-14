use color_eyre::eyre::Context;
use reqwest::Url;
use tokio::sync::mpsc;
use tokio_tungstenite::tungstenite;
use tracing::Instrument;
use twitch_api::eventsub::{EventsubWebsocketData, ReconnectPayload, WelcomePayload};

type Socket =
    tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>;

/// Type for events received from the Twitch EventSub websocket.
#[derive(Debug)]
pub enum Event {
    /// Welcome message with a session id.
    Welcome(String),
    /// Boxed due to large size.
    Notification(Box<twitch_api::eventsub::Event>),
}

/// Connects to the specified url and listens for events.
pub struct WebsocketClient {
    /// The session id given by twitch server.
    session_id: Option<String>,
    /// The url of the event sub.
    connect_url: Url,
    /// Sender of the events to the main thread.
    sender: mpsc::Sender<Event>,
}

impl WebsocketClient {
    pub fn new(sender: mpsc::Sender<Event>) -> Self {
        Self {
            session_id: None,
            connect_url: twitch_api::TWITCH_EVENTSUB_WEBSOCKET_URL.clone(),
            sender,
        }
    }

    /// Run the websocket subscriber
    #[tracing::instrument(name = "subscriber", skip_all, fields())]
    pub async fn run(mut self) -> color_eyre::Result<()> {
        // Establish the stream
        let mut socket = self
            .connect()
            .await
            .wrap_err("when establishing connection")?;
        // Loop over the stream, processing messages as they come in.
        loop {
            if let Some(msg) = futures::StreamExt::next(&mut socket).await {
                let span = tracing::info_span!("message received", raw_message = ?msg);
                let msg = match msg {
                    Err(tungstenite::Error::Protocol(
                        tungstenite::error::ProtocolError::ResetWithoutClosingHandshake,
                    )) => {
                        log::warn!(
                            "Connection was sent an unexpected frame or was reset, reestablishing it"
                        );
                        socket = self
                            .connect()
                            .instrument(span)
                            .await
                            .wrap_err("when reestablishing connection")?;
                        continue;
                    }
                    _ => msg.wrap_err("when getting message")?,
                };
                self.process_message(msg).instrument(span).await?
            }
        }
    }

    /// Connect to the websocket and return the stream
    async fn connect(&self) -> color_eyre::Result<Socket> {
        log::info!("Connecting to Twitch EventSub: {}", self.connect_url);
        let config = tungstenite::protocol::WebSocketConfig {
            max_send_queue: None,
            max_message_size: Some(64 << 20), // 64 MiB
            max_frame_size: Some(16 << 20),   // 16 MiB
            accept_unmasked_frames: false,
        };
        let (socket, _) =
            tokio_tungstenite::connect_async_with_config(&self.connect_url, Some(config))
                .await
                .wrap_err("Failed to connect to websocket")?;
        Ok(socket)
    }

    /// Process a message from the websocket
    async fn process_message(&mut self, msg: tungstenite::Message) -> color_eyre::Result<()> {
        log::debug!("Websocket: {}", msg);
        match msg {
            tungstenite::Message::Close(_) => todo!(),
            tungstenite::Message::Text(s) => {
                // Parse the message into a [twitch_api::eventsub::EventsubWebsocketData]
                match twitch_api::eventsub::Event::parse_websocket(&s)? {
                    EventsubWebsocketData::Welcome {
                        payload: WelcomePayload { session },
                        ..
                    }
                    | EventsubWebsocketData::Reconnect {
                        payload: ReconnectPayload { session },
                        ..
                    } => {
                        self.session_id = Some(session.id.to_string());
                        if let Some(url) = session.reconnect_url {
                            self.connect_url = url.parse()?;
                        }
                        self.sender
                            .send(Event::Welcome(session.id.to_string()))
                            .await
                            .wrap_err("when sending an event")?;
                        Ok(())
                    }
                    // Here is where you would handle the events you want to listen to
                    EventsubWebsocketData::Notification {
                        metadata: _,
                        payload,
                    } => {
                        log::debug!("Event: {:?}", payload);
                        self.sender
                            .send(Event::Notification(Box::new(payload)))
                            .await
                            .wrap_err("when sending an event")?;
                        Ok(())
                    }
                    EventsubWebsocketData::Revocation {
                        metadata,
                        payload: _,
                    } => color_eyre::eyre::bail!("got revocation event: {metadata:?}"),
                    EventsubWebsocketData::Keepalive {
                        metadata: _,
                        payload: _,
                    } => Ok(()),
                    _ => Ok(()),
                }
            }
            _ => Ok(()),
        }
    }
}
