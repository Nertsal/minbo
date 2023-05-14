use color_eyre::eyre::eyre;
use tokio::sync::mpsc;
use twitch_api::{twitch_oauth2::UserToken, HelixClient};

use crate::websocket::Event;

/// Client to send requests to Twitch API.
pub struct Client {
    pub helix: HelixClient<'static, reqwest::Client>,
    pub token: UserToken,
    /// Receiver of Twitch event sub events.
    pub receiver: mpsc::Receiver<Event>,
}

impl Client {
    // IMPORTANT: Careful not to expose `secrets` in the error message.
    pub async fn new(
        secrets: &crate::secret::Secrets,
        receiver: mpsc::Receiver<Event>,
    ) -> color_eyre::Result<Client> {
        // Create the HelixClient, which is used to make requests to the Twitch API
        let helix: HelixClient<reqwest::Client> = HelixClient::default();

        let token = crate::token::get_user_token(secrets, &helix).await?;

        Ok(Client {
            helix,
            token,
            receiver,
        })
    }

    /// Processes all messages in the buffer sent from the Twitch EventSub websocket.
    pub async fn process_events(&mut self) -> color_eyre::Result<()> {
        loop {
            match self.receiver.try_recv() {
                Ok(event) => {
                    self.process_event(event).await?;
                }
                Err(mpsc::error::TryRecvError::Empty) => return Ok(()),
                Err(mpsc::error::TryRecvError::Disconnected) => {
                    return Err(eyre!("Twitch event sub got disconnected"))
                }
            }
        }
    }

    /// Process an event from the Twitch EventSub websocket.
    async fn process_event(&mut self, event: Event) -> color_eyre::Result<()> {
        log::debug!("Received event: {:?}", event);
        match event {
            Event::Welcome(session_id) => self.process_welcome_message(session_id).await,
            Event::Notification(event) => self.process_notification(*event).await,
        }
    }

    /// Process the welcome message from Twitch EventSub.
    /// Used to setup event subscription.
    async fn process_welcome_message(&mut self, session_id: String) -> color_eyre::Result<()> {
        let transport = twitch_api::eventsub::Transport::websocket(session_id);
        self.helix
            .create_eventsub_subscription(
                twitch_api::eventsub::channel::ChannelFollowV2::new("158844187", "158844187"),
                transport.clone(),
                &self.token,
            )
            .await?;
        log::info!("Connected to twitch event sub");
        Ok(())
    }

    async fn process_notification(
        &mut self,
        _event: twitch_api::eventsub::Event,
    ) -> color_eyre::Result<()> {
        // TODO
        Ok(())
    }
}
