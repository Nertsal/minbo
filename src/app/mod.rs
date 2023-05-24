mod render;

use self::render::Render;

use color_eyre::eyre::Context;
use crossterm::event::{DisableMouseCapture, EnableMouseCapture};
use crossterm::execute;
use crossterm::terminal::{disable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use tui::backend::CrosstermBackend;

use crate::client::TwitchClient;
use crate::config::Config;
use crate::model::Model;

const TARGET_DELTA_TIME: f64 = 1.0 / 5.0;

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

pub enum AppAction {
    /// Send message to twitch chat.
    Say { message: String },
}

impl App {
    pub fn new(
        client: TwitchClient,
        config: Config,
        channel_login: String,
    ) -> color_eyre::Result<Self> {
        Ok(Self {
            client,
            terminal: Self::init_terminal().wrap_err("when setting up a terminal")?,
            model: Model::new(config),
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
        let mut time = tokio::time::Instant::now();

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
            let mut actions = Vec::new();
            let mut redraw = false; // TODO: smarter redraw

            // Update
            let delta_time = time.elapsed().as_secs_f64();
            time = tokio::time::Instant::now();
            self.update(delta_time)
                .await
                .wrap_err("when updating in event loop")?;

            // Twitch IRC
            while let Some(message) = self
                .client
                .try_recv()
                .wrap_err("when receiving a message")?
            {
                actions.extend(
                    self.model
                        .handle_twitch_event(message)
                        .wrap_err("when handling a twitch event")?,
                );
                redraw = true;
            }

            // Terminal
            while crossterm::event::poll(std::time::Duration::from_secs(0))
                .wrap_err("when polling a terminal event")?
            {
                let event = crossterm::event::read().wrap_err("when reading a terminal event")?;
                actions.extend(
                    self.model
                        .handle_terminal_event(event)
                        .wrap_err("when handling a terminal event")?,
                );
                redraw = true;
            }

            // Actions
            for action in actions {
                self.execute(action)
                    .await
                    .wrap_err("when executing an action")?;
            }

            // Render
            if redraw {
                self.render
                    .draw(&mut self.terminal, &self.model)
                    .wrap_err("when rendering the model")?;
            }

            let delta_time = time.elapsed().as_secs_f64();
            let sleep_time = (TARGET_DELTA_TIME - delta_time).max(0.0);
            tokio::time::sleep(tokio::time::Duration::from_secs_f64(sleep_time)).await;
        }

        self.clean_up().wrap_err("when cleaning up")?;

        Ok(())
    }

    async fn execute(&mut self, action: AppAction) -> color_eyre::Result<()> {
        match action {
            AppAction::Say { message } => {
                self.client
                    .irc
                    .say(self.channel_login.clone(), message)
                    .await
                    .wrap_err("when sending a message to twitch")?;
            }
        }
        Ok(())
    }

    /// Update the app over time.
    async fn update(&mut self, delta_time: f64) -> color_eyre::Result<()> {
        let actions = self
            .model
            .update(delta_time)
            .wrap_err("when updating the model")?;
        for action in actions {
            self.execute(action)
                .await
                .wrap_err("when executing an action")?;
        }
        Ok(())
    }
}
