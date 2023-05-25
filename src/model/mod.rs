mod action;
mod chat;
mod commands;
mod handle_event;

use std::collections::HashMap;

use tui::style::Color;
use twitch_irc::message::PrivmsgMessage;

use crate::app::AppAction;
use crate::config::Config;

pub use self::chat::*;
use self::commands::Commands;

pub struct Model {
    /// Set to false to shutdown gracefully.
    pub running: bool,
    pub commands: Commands,
    pub chat: Chat,
}

impl Model {
    pub fn new(config: &Config) -> Self {
        Self {
            running: true,
            commands: Commands::init(&config.commands),
            chat: Chat::new(),
        }
    }

    /// Reload configuration.
    pub fn reload(&mut self, config: &Config) {
        self.commands.reload(&config.commands);
    }

    pub fn update(&mut self, delta_time: f64) -> color_eyre::Result<Vec<AppAction>> {
        self.commands.update(delta_time);
        Ok(vec![])
    }
}
