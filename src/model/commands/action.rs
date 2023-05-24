use super::{parse::ArgsError, *};

/// Command callable actions that might require extra arguments.
/// Arguments are refered to in the docs like `$0` (for the first argument).
#[derive(Debug, Clone)]
pub enum CommandAction {
    /// Reload the configuration file.
    ReloadConfig,
    /// Echo the message.
    Say(String),
    /// Say hello to $0.
    Hello,
    /// Say bye to $0.
    Bye,
    /// Say good night to $0.
    GoodNight,
}

// macro_rules! extract_args {
//     // Extract a specific length list
//     ($from:expr, [$($in:tt),*$(...)?]) => {
//         extract_args!($from, [$($in),*$(...)?] -> [])
//     };
//     // Extract an empty list
//     ($from:expr, [] -> [$(out:tt),*]) => {
//         if $from.is_empty() {
//             [$(out),*]
//         } else {
//             return Err(ArgsError::TooMany)
//         }
//     };
// }

impl CommandAction {
    pub fn into_action(self, mut arguments: Vec<String>) -> Result<Action, ArgsError> {
        match self {
            CommandAction::ReloadConfig => Ok(Action::ReloadConfig),
            CommandAction::Say(msg) => Ok(Action::Say(msg)),
            CommandAction::Hello => {
                verify_args(&arguments, 1, true)?;
                let name = arguments.pop().unwrap();
                let msg = format!("Hi, {name} ^^");
                Ok(Action::Say(msg))
            }
            CommandAction::Bye => {
                verify_args(&arguments, 1, true)?;
                let name = arguments.pop().unwrap();
                let msg = format!("cya, {name}!");
                Ok(Action::Say(msg))
            }
            CommandAction::GoodNight => {
                verify_args(&arguments, 1, true)?;
                let name = arguments.pop().unwrap();
                let msg = format!("Good night, {name} ^^");
                Ok(Action::Say(msg))
            }
        }
    }
}

fn verify_args(args: &[String], len: usize, exact: bool) -> Result<(), ArgsError> {
    if args.len() < len {
        Err(ArgsError::NotEnough)
    } else if exact && args.len() > len {
        Err(ArgsError::TooMany)
    } else {
        Ok(())
    }
}
