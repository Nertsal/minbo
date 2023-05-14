use color_eyre::eyre::eyre;
use tokio::sync::mpsc::error::TryRecvError;
use twitch_irc::message::ServerMessage;

use crate::{secret::Secrets, TwitchClient, TwitchReceiver};

/// Client to communicate with Twitch API.
pub struct Client {
    /// Client to send requests to twitch.
    pub twitch: TwitchClient,
    /// Receiver of Twitch events.
    receiver: TwitchReceiver,
}

impl Client {
    pub fn new(secrets: &Secrets) -> color_eyre::Result<Client> {
        // Setup up Twitch IRC connection
        let config =
            twitch_irc::ClientConfig::new_simple(twitch_irc::login::StaticLoginCredentials::new(
                secrets.client.login_name.clone(),
                Some(secrets.client.oauth_token.clone()),
            ));
        let (receiver, twitch) = TwitchClient::new(config);

        Ok(Client { twitch, receiver })
    }

    /// Returns a message if any is queued.
    /// Returns an error if the client got disconnected.
    pub fn try_recv(&mut self) -> color_eyre::Result<Option<ServerMessage>> {
        match self.receiver.try_recv() {
            Ok(message) => Ok(Some(message)),
            Err(TryRecvError::Empty) => Ok(None),
            Err(TryRecvError::Disconnected) => Err(eyre!("Twitch IRC disconnected")),
        }
    }
}
