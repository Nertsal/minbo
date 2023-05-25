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
                let actions = self.handle_command_call(call);

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
    pub fn handle_terminal_event(&mut self, event: Event) -> Vec<AppAction> {
        match event {
            Event::Key(key) => self.handle_key(key),
            _ => vec![],
        }
    }

    fn handle_key(&mut self, event: KeyEvent) -> Vec<AppAction> {
        // C-c to exit
        if let KeyCode::Char('c') = event.code {
            if event.modifiers.contains(KeyModifiers::CONTROL) {
                self.running = false;
                return vec![];
            }
        }

        // TODO: focused window only
        let actions = self.chat.handle_key(event);

        let mut app_actions = Vec::new();
        for action in actions {
            app_actions.extend(self.execute(action));
        }
        app_actions
    }
}
