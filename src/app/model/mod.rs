use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};

pub struct Model {
    /// Set to false to shutdown gracefully.
    pub running: bool,
    pub chat_messages: Vec<ChatMessage>,
}

pub struct ChatMessage {
    pub sender_name: String,
    pub message: String,
}

impl Model {
    pub fn new() -> Self {
        Self {
            running: true,
            chat_messages: vec![
                ChatMessage {
                    sender_name: "Nertsal".to_string(),
                    message: "hello, world".to_string(),
                },
                ChatMessage {
                    sender_name: "kuviman".to_string(),
                    message: "definitely me".to_string(),
                },
            ],
        }
    }

    pub fn handle_event(&mut self, event: Event) {
        match event {
            Event::FocusGained => todo!(),
            Event::FocusLost => todo!(),
            Event::Key(key) => self.handle_key(key),
            Event::Mouse(_) => todo!(),
            Event::Paste(_) => todo!(),
            Event::Resize(_, _) => todo!(),
        }
    }

    fn handle_key(&mut self, event: KeyEvent) {
        match event.code {
            crossterm::event::KeyCode::Char(c) => self.handle_char(c, event.modifiers),
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
