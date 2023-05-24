mod action;
mod commands;

use std::collections::HashMap;

use color_eyre::eyre::Context;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use tui::style::Color;
use twitch_irc::message::PrivmsgMessage;

use crate::app::AppAction;
use crate::client::TwitchMessage;
use crate::config::Config;

use self::commands::{AuthorityLevel, CommandCall, Commands};

pub struct Model {
    /// Set to false to shutdown gracefully.
    pub running: bool,
    pub chat: Vec<ChatItem>,
    pub chatters: HashMap<String, Color>,
    pub selected_item: Option<usize>,
    pub commands: Commands,
}

#[derive(Debug)]
pub enum ChatItem {
    /// Boxed due to large size.
    Message(Box<PrivmsgMessage>),
    Event(String),
}

impl Model {
    pub fn new(config: &Config) -> Self {
        Self {
            running: true,
            chat: vec![],
            chatters: HashMap::new(),
            selected_item: None,
            commands: Commands::init(&config.commands),
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

    /// Process an event from Twitch.
    pub fn handle_twitch_event(
        &mut self,
        message: TwitchMessage,
    ) -> color_eyre::Result<Vec<AppAction>> {
        log::debug!("Twitch IRC: {:?}", message);
        match message {
            TwitchMessage::Privmsg(message) => {
                // Remember the chatter
                if let Some(color) = message.name_color {
                    let color = Color::Rgb(color.r, color.g, color.b);
                    self.chatters.insert(message.sender.name.clone(), color);
                }

                // Check command
                let call = CommandCall {
                    message: &message.message_text,
                    authority: AuthorityLevel::from_badges(&message.badges),
                };
                let actions = self
                    .handle_command_call(call)
                    .wrap_err("when handling a command call")?;

                self.chat.push(ChatItem::Message(Box::new(message)));
                Ok(actions)
            }
            TwitchMessage::UserNotice(notice) => {
                self.chat.push(ChatItem::Event(notice.system_message));
                Ok(vec![])
            }
            _ => Ok(vec![]),
        }
    }

    /// Process a terminal event.
    pub fn handle_terminal_event(&mut self, event: Event) -> color_eyre::Result<Vec<AppAction>> {
        match event {
            Event::FocusGained => {}
            Event::FocusLost => {}
            Event::Key(key) => self.handle_key(key),
            Event::Mouse(_) => {}
            Event::Paste(_) => {}
            Event::Resize(_, _) => {}
        }
        Ok(vec![])
    }

    fn handle_key(&mut self, event: KeyEvent) {
        if let KeyCode::Char(c) = event.code {
            self.handle_char(c, event.modifiers)
        }
    }

    fn handle_char(&mut self, c: char, modifiers: KeyModifiers) {
        match c {
            'c' if modifiers.contains(KeyModifiers::CONTROL) => {
                self.running = false;
            }
            'j' => {
                let max = self.chat.len().max(1) - 1;
                self.selected_item =
                    Some(self.selected_item.map(|i| (i + 1).min(max)).unwrap_or(max));
            }
            'k' => {
                let max = self.chat.len().max(1) - 1;
                self.selected_item = Some(
                    self.selected_item
                        .map(|i| i.saturating_sub(1))
                        .unwrap_or(max),
                );
            }
            'G' => {
                self.selected_item = None;
            }
            _ => {}
        }
    }
}
