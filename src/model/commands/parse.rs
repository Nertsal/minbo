use super::*;

#[derive(Debug, Clone)]
pub enum CommandParseError {
    Parse(ParseError),
    Args(ArgsError),
    Call(CallError),
}

#[derive(Debug, Clone)]
pub enum ArgsError {
    TooMany,
    NotEnough,
}

#[derive(Debug, Clone)]
pub enum CallError {
    OnCooldown,
    Unauthorized,
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

impl From<CallError> for CommandParseError {
    fn from(v: CallError) -> Self {
        Self::Call(v)
    }
}

impl std::error::Error for ArgsError {}

impl std::fmt::Display for ArgsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ArgsError::TooMany => write!(f, "Too many arguments"),
            ArgsError::NotEnough => write!(f, "Not enough arguments"),
        }
    }
}

impl std::error::Error for CallError {}

impl std::fmt::Display for CallError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CallError::OnCooldown => write!(f, "Command is on cooldown"),
            CallError::Unauthorized => write!(f, "Unauthorized"),
        }
    }
}
