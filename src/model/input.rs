use crossterm::event::{KeyCode, KeyEvent};

#[derive(Debug, Clone, Default)]
pub struct InputBox {
    pub content: String,
    /// Cursor position.
    pub cursor: usize,
}

impl InputBox {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn take(&mut self) -> String {
        self.cursor = 0;
        std::mem::take(&mut self.content)
    }

    fn check_cursor(&mut self) {
        if self.cursor > self.content.len() {
            self.cursor = self.content.len();
        }
    }

    fn remove(&mut self, pos: usize) {
        if pos < self.content.len() {
            self.content.remove(pos);
        }
    }

    fn insert(&mut self, pos: usize, c: char) {
        if pos <= self.content.len() {
            self.content.insert(pos, c);
        }
    }

    pub fn handle_key(&mut self, event: KeyEvent) {
        match event.code {
            KeyCode::Char(c) => {
                self.insert(self.cursor, c);
                self.cursor += 1;
            }
            KeyCode::Backspace => self.remove(self.cursor.saturating_sub(1)),
            KeyCode::Delete => self.remove(self.cursor),
            KeyCode::Home => self.cursor = 0,
            KeyCode::End => self.cursor = self.content.len(),
            KeyCode::Left => self.cursor = self.cursor.saturating_sub(1),
            KeyCode::Right => self.cursor += 1,
            _ => {}
        }
        self.check_cursor();
    }
}
