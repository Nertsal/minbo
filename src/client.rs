use color_eyre::eyre::eyre;
use tokio::sync::mpsc::error::TryRecvError;
use twitch_irc::message::ServerMessage;

use crate::{TwitchClient, TwitchReceiver};

/// Client to communicate with Twitch API.
pub struct Client {
    /// Client to send requests to twitch.
    pub twitch: TwitchClient,
    /// Receiver of Twitch events.
    pub receiver: TwitchReceiver,
}

impl Client {
    pub async fn new(twitch: TwitchClient, receiver: TwitchReceiver) -> color_eyre::Result<Client> {
        Ok(Client { twitch, receiver })
    }

    /// Processes all messages in the `receiver` buffer sent from Twitch.
    pub async fn process_events(&mut self) -> color_eyre::Result<()> {
        loop {
            match self.receiver.try_recv() {
                Ok(message) => {
                    self.process_message(message).await?;
                }
                Err(TryRecvError::Empty) => return Ok(()),
                Err(TryRecvError::Disconnected) => return Err(eyre!("Twitch IRC disconnected")),
            }
        }
    }

    /// Process an event from Twitch.
    async fn process_message(&mut self, message: ServerMessage) -> color_eyre::Result<()> {
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
}
