use super::*;

impl Commands {
    /// Reloads `configured` command list.
    pub fn reload(&mut self, config: &SimpleCommands) {
        self.configured = config
            .commands
            .iter()
            .map(|(command, response)| {
                CommandTree::new(
                    CommandBuilder::new()
                        .literal([format!("!{}", command)])
                        .finalize(true, CommandAction::Say(response.to_owned())),
                )
                .with_cooldown(config.cooldown)
            })
            .collect();
    }

    pub fn init(config: &SimpleCommands) -> Self {
        let system = [CommandTree::new(command!(
            "!reload";
            true, CommandAction::ReloadConfig
        ))
        .with_authority(AuthorityLevel::Broadcaster)];

        let greetings = [
            ("!hello", CommandAction::Hello),
            ("!bye", CommandAction::Bye),
            ("!gn", CommandAction::GoodNight),
        ]
        .into_iter()
        .map(|(command, action)| {
            CommandTree::new(
                CommandBuilder::new()
                    .literal([command])
                    .word()
                    .finalize(true, action),
            )
            .with_cooldown(30.0)
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
