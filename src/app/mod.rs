mod model;
mod render;

use self::model::Model;
use self::render::Render;

use color_eyre::eyre::Context;
use crossterm::event::{DisableMouseCapture, EnableMouseCapture};
use crossterm::execute;
use crossterm::terminal::{disable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use tui::backend::CrosstermBackend;

use crate::client::TwitchClient;

type Backend = CrosstermBackend<std::io::Stdout>;
type Terminal = tui::Terminal<Backend>;

/// The application/interface for the bot.
pub struct App {
    client: TwitchClient,
    terminal: Terminal,
    model: Model,
    render: Render,
    /// Name of the channel to connect to.
    channel_login: String,
}

impl App {
    pub fn new(client: TwitchClient, channel_login: String) -> color_eyre::Result<Self> {
        Ok(Self {
            client,
            terminal: Self::init_terminal().wrap_err("when setting up a terminal")?,
            model: Model::new(),
            render: Render::new(),
            channel_login,
        })
    }

    /// Configure the terminal.
    fn init_terminal() -> color_eyre::Result<Terminal> {
        // Configure stdout
        let mut stdout = std::io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)
            .wrap_err("when setting up stdout")?;

        // Setup backend
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend).wrap_err("when setting up terminal")?;

        // Enable raw mode to control keyboard inputs
        crossterm::terminal::enable_raw_mode().wrap_err("when enabling terminal raw mode")?;

        Ok(terminal)
    }

    // Restore terminal.
    fn clean_up(&mut self) -> color_eyre::Result<()> {
        disable_raw_mode()?;
        execute!(
            self.terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        self.terminal.show_cursor()?;
        Ok(())
    }

    pub async fn run(mut self) -> color_eyre::Result<()> {
        self.client
            .irc
            .join(self.channel_login.clone())
            .wrap_err("when joining a channel")?;
        // self.client
        //     .irc
        //     .say(self.channel_login.clone(), "Hello".to_string())
        //     .await
        //     .wrap_err("when sending a message")?;

        self.terminal.clear()?;
        self.render
            .draw(&mut self.terminal, &self.model)
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
                .draw(&mut self.terminal, &self.model)
                .wrap_err("when rendering the model")?;

            // TODO: Proper fps control
            tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        }

        self.clean_up().wrap_err("when cleaning up")?;

        Ok(())
    }

    // /// Update the app over time.
    // fn update(&mut self, _delta_time: f32) -> color_eyre::Result<()> {
    //     Ok(())
    // }
}
