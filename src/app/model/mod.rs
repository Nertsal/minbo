use std::collections::HashMap;

use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use tui::style::Color;
use twitch_irc::message::PrivmsgMessage;

use crate::client::TwitchMessage;

pub struct Model {
    /// Set to false to shutdown gracefully.
    pub running: bool,
    pub chat: Vec<ChatItem>,
    pub chatters: HashMap<String, Color>,
}

#[derive(Debug)]
pub enum ChatItem {
    /// Boxed due to large size.
    Message(Box<PrivmsgMessage>),
    Event(String),
}

impl Model {
    pub fn new() -> Self {
        Self {
            running: true,
            chat: vec![],
            chatters: HashMap::new(),
        }
    }

    /// Process an event from Twitch.
    pub fn handle_twitch_event(&mut self, message: TwitchMessage) -> color_eyre::Result<()> {
        log::debug!("Twitch IRC: {:?}", message);
        match message {
            TwitchMessage::Privmsg(message) => {
                if let Some(color) = message.name_color {
                    let color = Color::Rgb(color.r, color.g, color.b);
                    self.chatters.insert(message.sender.name.clone(), color);
                }
                self.chat.push(ChatItem::Message(Box::new(message)));
            }
            TwitchMessage::UserNotice(notice) => {
                self.chat.push(ChatItem::Event(notice.system_message));
            }
            _ => {}
        }
        Ok(())
    }

    /// Process a terminal event.
    pub fn handle_terminal_event(&mut self, event: Event) {
        match event {
            Event::FocusGained => {}
            Event::FocusLost => {}
            Event::Key(key) => self.handle_key(key),
            Event::Mouse(_) => {}
            Event::Paste(_) => {}
            Event::Resize(_, _) => {}
        }
    }

    fn handle_key(&mut self, event: KeyEvent) {
        match event.code {
            KeyCode::Char(c) => self.handle_char(c, event.modifiers),
            _ => {}
        }
    }

    fn handle_char(&mut self, c: char, modifiers: KeyModifiers) {
        match c {
            'c' if modifiers.contains(KeyModifiers::CONTROL) => {
                // Ctrl+C
                self.running = false;
            }
            _ => {}
        }
    }
}
