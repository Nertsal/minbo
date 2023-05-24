use super::*;

#[derive(Debug, Clone)]
pub enum Action {
    /// Say hello to `name`.
    Hello { name: String },
}

impl Model {
    pub fn execute(&mut self, action: Action) -> color_eyre::Result<Vec<AppAction>> {
        log::debug!("Executing action: {:?}", action);
        match action {
            Action::Hello { name } => {
                let message = format!("Hi, {name} ^^");
                Ok(vec![AppAction::Say { message }])
            }
        }
    }
}
