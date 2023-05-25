use super::{
    commands::{AuthorityLevel, CommandCall},
    *,
};

#[derive(Debug, Clone)]
pub enum Action {
    HandleCommand {
        command: String,
        authority: AuthorityLevel,
    },
    /// Reload the configuration file.
    ReloadConfig,
    /// Echo the message.
    Say(String),
}

impl Model {
    pub fn execute(&mut self, action: Action) -> Vec<AppAction> {
        log::debug!("Executing action: {:?}", action);
        match action {
            Action::HandleCommand { command, authority } => {
                let call = CommandCall {
                    message: &command,
                    authority,
                };
                self.handle_command_call(call)
            }
            Action::ReloadConfig => {
                // Pass the action to the app, so the model is kept pure
                vec![AppAction::ReloadConfig]
            }
            Action::Say(message) => vec![AppAction::Say { message }],
        }
    }
}
