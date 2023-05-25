use super::*;

#[derive(Debug)]
pub struct Chat {
    pub items: Vec<ChatItem>,
    pub chatters: HashMap<String, Color>,
    pub selected_item: Option<usize>,
    /// Input line for the user to make commands as a [Host](AuthorityLevel::Host).
    pub input: String,
}

#[derive(Debug)]
pub enum ChatItem {
    /// Boxed due to large size.
    Message(Box<PrivmsgMessage>),
    Event(String),
}

impl Chat {
    pub fn new() -> Self {
        Self {
            items: vec![],
            chatters: HashMap::new(),
            selected_item: None,
            input: String::new(),
        }
    }
}

impl Default for Chat {
    fn default() -> Self {
        Self::new()
    }
}
