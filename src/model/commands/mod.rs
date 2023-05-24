mod action;
mod authority;
mod init;
mod parse;
mod tree;

pub use self::action::CommandAction;
pub use self::authority::AuthorityLevel;
pub use self::parse::{ArgsError, CommandParseError};
pub use self::tree::CommandTree;

use crate::config::SimpleCommands;

use super::action::Action;
use super::*;

use std::collections::BTreeMap;

use minmands::{command, CommandBuilder, CommandNode, ParseError};

// Note: Make sure to add new field to [Commands::iter_mut] method.
pub struct Commands {
    /// Specified in the config and updated on config reload.
    configured: Vec<CommandTree>,
    /// Hardcoded commands.
    hardcoded: Vec<CommandTree>,
}

#[derive(Debug, Clone, Copy)]
pub struct CommandCall<'a> {
    pub message: &'a str,
    pub authority: AuthorityLevel,
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
}

impl Model {
    pub fn handle_command_call(&mut self, call: CommandCall) -> color_eyre::Result<Vec<AppAction>> {
        // Parse commands
        let mut actions = Vec::new();
        for command in self.commands.iter_mut() {
            // Cooldown is checked and updated inside `parse`
            match command.parse(call) {
                Ok(action) => actions.push(action),
                Err(parse::CommandParseError::Parse(_)) => continue, // Did not parse
                Err(parse::CommandParseError::Args(err)) => {
                    // Parsed, but action could not be formed
                    log::debug!(
                        "Action could not be formed: {}\n  for call: {:?}\n  for command: {:?}",
                        err,
                        call,
                        command
                    );
                    continue;
                }
                Err(parse::CommandParseError::Call(_)) => {
                    // Invalid call
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
