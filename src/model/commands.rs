use std::collections::BTreeMap;

use super::action::Action;
use super::*;

use minmands::{command, CommandNode, ParseError};

#[derive(Debug, Clone)]
pub struct CommandTree {
    root: CommandNode<CommandAction>,
    /// Command cooldown in seconds.
    cooldown: f64,
    /// Time until cooldown expires for individual argument variants.
    cooldown_timers: BTreeMap<Vec<String>, f64>,
}

impl CommandTree {
    /// Update cooldown.
    pub fn update(&mut self, delta_time: f64) {
        for time in self.cooldown_timers.values_mut() {
            *time -= delta_time;
        }
        self.cooldown_timers.retain(|_, time| *time > 0.0);
    }
}

/// Command callable actions that might require extra arguments.
/// Arguments are refered to in the docs like `$0` (for the first argument).
#[derive(Debug, Clone)]
pub enum CommandAction {
    /// Says hello to $0.
    Hello,
}

#[derive(Debug, Clone)]
pub enum CommandParseError {
    Parse(ParseError),
    Args(ArgsError),
}

impl From<ParseError> for CommandParseError {
    fn from(v: ParseError) -> Self {
        Self::Parse(v)
    }
}

impl From<ArgsError> for CommandParseError {
    fn from(v: ArgsError) -> Self {
        Self::Args(v)
    }
}

#[derive(Debug, Clone)]
pub enum ArgsError {
    OnCooldown,
    TooMany,
    NotEnough,
}

impl std::error::Error for ArgsError {}

impl std::fmt::Display for ArgsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ArgsError::OnCooldown => write!(f, "Command is on cooldown"),
            ArgsError::TooMany => write!(f, "Too many arguments"),
            ArgsError::NotEnough => write!(f, "Not enough arguments"),
        }
    }
}

impl CommandTree {
    pub fn new(cooldown: f64, root: CommandNode<CommandAction>) -> Self {
        Self {
            root,
            cooldown,
            cooldown_timers: BTreeMap::new(),
        }
    }

    pub fn parse(&mut self, command: &str) -> Result<Action, CommandParseError> {
        let parsed = self.root.parse(command)?;
        if self.cooldown_timers.contains_key(&parsed.arguments) {
            return Err(CommandParseError::Args(ArgsError::OnCooldown));
        }
        self.cooldown_timers
            .insert(parsed.arguments.clone(), self.cooldown);
        let action = parsed.value.into_action(parsed.arguments)?;
        Ok(action)
    }
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
            CommandAction::Hello => {
                verify_args(&arguments, 1, true)?;
                let name = arguments.pop().unwrap();
                Ok(Action::Hello { name })
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

impl Model {
    pub fn handle_command_call(&mut self, call: &str) -> color_eyre::Result<Vec<AppAction>> {
        // Parse commands
        let mut actions = Vec::new();
        for command in &mut self.commands {
            // Cooldown is checked and updated inside `parse`
            match command.parse(call) {
                Ok(action) => actions.push(action),
                Err(CommandParseError::Parse(_)) => continue, // Did not parse
                Err(CommandParseError::Args(err)) => {
                    // Parsed, but action could not be formed
                    log::debug!(
                        "Action could not be formed: {}\n  for call: {:?}\n  for command: {:?}",
                        err,
                        call,
                        command
                    );
                    continue;
                }
            }
        }

        let mut app_actions = Vec::new();
        for action in actions {
            let actions = self.execute(action)?;
            app_actions.extend(actions);
        }
        Ok(app_actions)
    }

    pub fn init_commands() -> Vec<CommandTree> {
        vec![CommandTree::new(
            30.0, // 30 sec cooldown
            command!(
                "!hello";
                word;
                true, CommandAction::Hello
            ),
        )]
    }
}
