use color_eyre::eyre::Context;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};

use crate::client::TwitchMessage;

use super::commands::{AuthorityLevel, CommandCall};
use super::*;

impl Model {
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
                    self.chat
                        .chatters
                        .insert(message.sender.name.clone(), color);
                }

                // Check command
                let call = CommandCall {
                    message: &message.message_text,
                    authority: AuthorityLevel::from_badges(&message.badges),
                };
                let actions = self
                    .handle_command_call(call)
                    .wrap_err("when handling a command call")?;

                self.chat.items.push(ChatItem::Message(Box::new(message)));
                Ok(actions)
            }
            TwitchMessage::UserNotice(notice) => {
                self.chat.items.push(ChatItem::Event(notice.system_message));
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
                let max = self.chat.items.len().max(1) - 1;
                self.chat.selected_item = Some(
                    self.chat
                        .selected_item
                        .map(|i| (i + 1).min(max))
                        .unwrap_or(max),
                );
            }
            'k' => {
                let max = self.chat.items.len().max(1) - 1;
                self.chat.selected_item = Some(
                    self.chat
                        .selected_item
                        .map(|i| i.saturating_sub(1))
                        .unwrap_or(max),
                );
            }
            'G' => {
                self.chat.selected_item = None;
            }
            _ => {}
        }
    }
}
