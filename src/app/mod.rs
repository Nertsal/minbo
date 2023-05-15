mod model;
mod render;

use self::model::Model;
use self::render::Render;

use color_eyre::eyre::Context;

use crate::client::TwitchClient;

/// The application/interface for the bot.
pub struct App {
    client: TwitchClient,
    model: Model,
    render: Render,
}

impl App {
    pub fn new(client: TwitchClient) -> color_eyre::Result<Self> {
        Ok(Self {
            client,
            model: Model::new(),
            render: Render::new().wrap_err("when setting up a renderer")?,
        })
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

        self.render
            .draw(&self.model)
            .wrap_err("when rendering the model")?;

        // Event loop
        while self.model.running {
            // Twitch IRC
            if let Some(message) = self
                .client
                .try_recv()
                .wrap_err("when receiving a message")?
            {
                self.model
                    .handle_twitch_event(message)
                    .wrap_err("when processing a message")?;
            }

            // Terminal
            if crossterm::event::poll(std::time::Duration::from_secs(0))
                .wrap_err("when polling a terminal event")?
            {
                let event = crossterm::event::read().wrap_err("when reading a terminal event")?;
                self.model.handle_terminal_event(event);
            }

            // TODO: lazy
            // Render
            self.render
                .draw(&self.model)
                .wrap_err("when rendering the model")?;

            // TODO: Proper fps control
            tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        }

        Ok(())
    }

    // /// Update the app over time.
    // fn update(&mut self, _delta_time: f32) -> color_eyre::Result<()> {
    //     Ok(())
    // }
}
