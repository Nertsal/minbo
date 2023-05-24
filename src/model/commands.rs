use std::collections::BTreeMap;

use crate::config::SimpleCommands;

use super::action::Action;
use super::*;

use minmands::{command, CommandBuilder, CommandNode, ParseError};

// Note: Make sure to add new field to [Commands::iter_mut] method.
pub struct Commands {
    /// Specified in the config and updated on config reload.
    configured: Vec<CommandTree>,
    /// Hardcoded commands.
    hardcoded: Vec<CommandTree>,
}

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

impl Model {
    pub fn handle_command_call(&mut self, call: &str) -> color_eyre::Result<Vec<AppAction>> {
        // Parse commands
        let mut actions = Vec::new();
        for command in self.commands.iter_mut() {
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
}

impl Commands {
    /// Update cooldown.
    pub fn update(&mut self, delta_time: f64) {
        for command in self.iter_mut() {
            command.update(delta_time);
        }
    }

    fn iter_mut(&mut self) -> impl Iterator<Item = &mut CommandTree> {
        iter_tools::chain![&mut self.configured, &mut self.hardcoded]
    }

    /// Reloads `configured` command list.
    pub fn reload(&mut self, config: &SimpleCommands) {
        self.configured = config
            .commands
            .iter()
            .map(|(command, response)| {
                CommandTree::new(
                    config.cooldown,
                    CommandBuilder::new()
                        .literal([format!("!{}", command)])
                        .finalize(true, CommandAction::Say(response.to_owned())),
                )
            })
            .collect();
    }

    pub fn init(config: &SimpleCommands) -> Self {
        let system = [CommandTree::new(
            0.0,
            command!(
                "!reload";
                true, CommandAction::ReloadConfig
            ),
        )];

        let greetings = [
            ("!hello", CommandAction::Hello),
            ("!bye", CommandAction::Bye),
            ("!gn", CommandAction::GoodNight),
        ]
        .into_iter()
        .map(|(command, action)| {
            CommandTree::new(
                30.0,
                CommandBuilder::new()
                    .literal([command])
                    .word()
                    .finalize(true, action),
            )
        });

        let hardcoded = iter_tools::chain![system, greetings];

        let mut commands = Self {
            configured: vec![], // Set on reload
            hardcoded: hardcoded.collect(),
        };
        commands.reload(config);
        commands
    }
}
