use crossterm::event::{KeyCode, KeyEvent};

use super::{action::Action, commands::AuthorityLevel, *};

#[derive(Debug)]
pub struct Chat {
    pub mode: ChatMode,
    pub items: Vec<ChatItem>,
    pub chatters: HashMap<String, Color>,
    pub selected_item: Option<usize>,
    /// Input line for the user to make commands as a [Host](AuthorityLevel::Host).
    pub input: InputBox,
}

#[derive(Debug)]
pub enum ChatItem {
    Message(ChatMessage),
    Event(String),
}

#[derive(Debug, Clone)]
pub struct ChatMessage {
    pub sender_name: String,
    pub text: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChatMode {
    /// Scroll through messages.
    Normal,
    /// Insert text into a command prompt.
    Insert,
}

impl Chat {
    pub fn new() -> Self {
        Self {
            mode: ChatMode::Normal,
            items: vec![],
            chatters: HashMap::new(),
            selected_item: None,
            input: InputBox::new(),
        }
    }

    pub fn handle_key(&mut self, event: KeyEvent) -> Vec<Action> {
        match self.mode {
            ChatMode::Normal => self.handle_key_normal(event),
            ChatMode::Insert => self.handle_key_insert(event),
        }
    }

    fn handle_key_normal(&mut self, event: KeyEvent) -> Vec<Action> {
        if let KeyCode::Char(c) = event.code {
            match c {
                'j' => {
                    let max = self.items.len().max(1) - 1;
                    self.selected_item =
                        Some(self.selected_item.map(|i| (i + 1).min(max)).unwrap_or(max));
                }
                'k' => {
                    let max = self.items.len().max(1) - 1;
                    self.selected_item = Some(
                        self.selected_item
                            .map(|i| i.saturating_sub(1))
                            .unwrap_or(max),
                    );
                }
                'G' => {
                    self.selected_item = None;
                }
                'i' => {
                    self.mode = ChatMode::Insert;
                }
                _ => {}
            }
        }
        vec![]
    }

    fn handle_key_insert(&mut self, event: KeyEvent) -> Vec<Action> {
        let mut actions = vec![];
        match event.code {
            KeyCode::Esc => self.mode = ChatMode::Normal,
            KeyCode::Enter => {
                let command = self.input.take();
                self.items.push(ChatItem::Message(ChatMessage {
                    sender_name: "Host".to_string(),
                    text: command.clone(),
                }));
                actions.push(Action::HandleCommand {
                    command,
                    authority: AuthorityLevel::Host,
                });
            }
            _ => self.input.handle_key(event),
        }
        actions
    }
}

impl Default for Chat {
    fn default() -> Self {
        Self::new()
    }
}
