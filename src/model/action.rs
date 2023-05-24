use super::*;

#[derive(Debug, Clone)]
pub enum Action {
    /// Reload the configuration file.
    ReloadConfig,
    /// Echo the message.
    Say(String),
}

impl Model {
    pub fn execute(&mut self, action: Action) -> color_eyre::Result<Vec<AppAction>> {
        log::debug!("Executing action: {:?}", action);
        match action {
            Action::ReloadConfig => {
                // Pass the action to the app, so the model is kept pure
                Ok(vec![AppAction::ReloadConfig])
            }
            Action::Say(message) => Ok(vec![AppAction::Say { message }]),
        }
    }
}
