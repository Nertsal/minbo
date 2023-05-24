use super::*;

#[derive(Debug, Clone)]
pub enum Action {
    /// Echo the message.
    Say(String),
}

impl Model {
    pub fn execute(&mut self, action: Action) -> color_eyre::Result<Vec<AppAction>> {
        log::debug!("Executing action: {:?}", action);
        match action {
            Action::Say(message) => Ok(vec![AppAction::Say { message }]),
        }
    }
}
