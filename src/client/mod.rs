mod token;

use color_eyre::eyre::{eyre, Context};
use tokio::sync::mpsc::{error::TryRecvError, UnboundedReceiver};
use twitch_irc::{login::RefreshingLoginCredentials, message::ServerMessage, SecureTCPTransport};

use crate::secret::Secrets;

use self::token::CustomTokenStorage;

pub type TwitchIRCClient =
    twitch_irc::TwitchIRCClient<SecureTCPTransport, RefreshingLoginCredentials<CustomTokenStorage>>;
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
    pub async fn new(secrets: &Secrets) -> color_eyre::Result<Self> {
        // Fetch access token
        let storage = CustomTokenStorage::init(secrets)
            .await
            .wrap_err("when fetching tokens")?;

        // The bot's username will be fetched based on your access token
        let credentials = RefreshingLoginCredentials::init(
            secrets.client.client_id.clone(),
            secrets.client.client_secret.clone(),
            storage,
        );
        // It is also possible to use the same credentials in other places
        // such as API calls by cloning them.

        // Setup up Twitch IRC connection
        let config = twitch_irc::ClientConfig::new_simple(credentials);
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
