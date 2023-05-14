use color_eyre::eyre::Context;

use crate::client::{TwitchClient, TwitchMessage};

/// The application/interface for the bot.
pub struct App {
    client: TwitchClient,
}

impl App {
    pub fn new(client: TwitchClient) -> Self {
        Self { client }
    }

    pub async fn run(mut self) -> color_eyre::Result<()> {
        self.client
            .irc
            .join("nertsal".to_string())
            .wrap_err("when joining a channel")?;
        self.client
            .irc
            .say("nertsal".to_string(), "Hello".to_string())
            .await
            .wrap_err("when sending a message")?;

        // Event loop
        loop {
            if let Some(message) = self
                .client
                .try_recv()
                .wrap_err("when receiving a message")?
            {
                self.process_twitch_message(message)
                    .wrap_err("when processing a message")?;
            }
        }
    }

    // /// Update the app over time.
    // fn update(&mut self, _delta_time: f32) -> color_eyre::Result<()> {
    //     Ok(())
    // }

    /// Process an event from Twitch.
    fn process_twitch_message(&mut self, message: TwitchMessage) -> color_eyre::Result<()> {
        log::debug!("Received message: {:?}", message);
        match message {
            TwitchMessage::Notice(notice) => log::info!("Notice: {}", notice.message_text),
            TwitchMessage::Privmsg(message) => {
                log::info!("[Chat] {}: {}", message.channel_login, message.message_text)
            }
            TwitchMessage::UserNotice(notice) => log::info!("Event: {}", notice.system_message),
            _ => {}
        }
        Ok(())
    }
}
