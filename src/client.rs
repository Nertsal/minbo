use color_eyre::eyre::eyre;
use tokio::sync::mpsc::{error::TryRecvError, UnboundedReceiver};
use twitch_irc::{login::StaticLoginCredentials, message::ServerMessage, SecureTCPTransport};

use crate::secret::Secrets;

pub type TwitchIRCClient = twitch_irc::TwitchIRCClient<SecureTCPTransport, StaticLoginCredentials>;
type TwitchReceiver = UnboundedReceiver<ServerMessage>;

pub type TwitchMessage = ServerMessage;

/// Client to communicate with Twitch API.
pub struct TwitchClient {
    /// Client to send requests to twitch.
    pub irc: TwitchIRCClient,
    /// Receiver of Twitch events.
    receiver: TwitchReceiver,
}

impl TwitchClient {
    pub fn new(secrets: &Secrets) -> color_eyre::Result<Self> {
        // Setup up Twitch IRC connection
        let config =
            twitch_irc::ClientConfig::new_simple(twitch_irc::login::StaticLoginCredentials::new(
                secrets.client.login_name.clone(),
                Some(secrets.client.oauth_token.clone()),
            ));
        let (receiver, irc) = TwitchIRCClient::new(config);

        Ok(Self { irc, receiver })
    }

    /// Returns a message if any is queued.
    /// Returns an error if the client got disconnected.
    pub fn try_recv(&mut self) -> color_eyre::Result<Option<TwitchMessage>> {
        match self.receiver.try_recv() {
            Ok(message) => Ok(Some(message)),
            Err(TryRecvError::Empty) => Ok(None),
            Err(TryRecvError::Disconnected) => Err(eyre!("Twitch IRC disconnected")),
        }
    }
}
