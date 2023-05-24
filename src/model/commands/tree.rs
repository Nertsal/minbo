use super::{parse::CallError, *};

#[derive(Debug, Clone)]
pub struct CommandTree {
    root: CommandNode<CommandAction>,
    authority_level: AuthorityLevel,
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

impl CommandTree {
    pub fn new(root: CommandNode<CommandAction>) -> Self {
        Self {
            root,
            authority_level: AuthorityLevel::Viewer,
            cooldown: 0.0,
            cooldown_timers: BTreeMap::new(),
        }
    }

    pub fn with_cooldown(mut self, cooldown: f64) -> Self {
        self.cooldown = cooldown;
        self
    }

    pub fn with_authority(mut self, level: AuthorityLevel) -> Self {
        self.authority_level = level;
        self
    }

    pub fn parse(&mut self, call: CommandCall) -> Result<Action, CommandParseError> {
        // Parse
        let parsed = self.root.parse(call.message)?;

        // Check authority
        if call.authority < self.authority_level {
            return Err(CallError::Unauthorized.into());
        }

        // Check cooldown
        if self.cooldown_timers.contains_key(&parsed.arguments) {
            return Err(CallError::OnCooldown.into());
        }

        // Set cooldown
        self.cooldown_timers
            .insert(parsed.arguments.clone(), self.cooldown);

        // Get action
        let action = parsed.value.into_action(parsed.arguments)?;
        Ok(action)
    }
}
