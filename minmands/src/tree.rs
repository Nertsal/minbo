#[derive(Debug, Clone, Copy)]
pub enum ArgumentType {
    /// Parse a single word.
    Word,
    /// Parse till the end of line.
    Line,
    /// Parse till the end of input.
    Tail,
}

impl std::fmt::Display for ArgumentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ArgumentType::Word => write!(f, "Word"),
            ArgumentType::Line => write!(f, "Line"),
            ArgumentType::Tail => write!(f, "Tail"),
        }
    }
}

/// A node in the command tree.
/// Generic over the 'marker' type that is returned upon succefully parsing a branch.
#[derive(Debug, Clone)]
pub enum CommandNode<T: Clone> {
    Literal {
        literals: Vec<String>,
        child_nodes: Vec<CommandNode<T>>,
    },
    Argument {
        argument_type: ArgumentType,
        child_nodes: Vec<CommandNode<T>>,
    },
    ArgumentChoice {
        choices: Vec<String>,
        child_nodes: Vec<CommandNode<T>>,
    },
    Final {
        /// If `true`, then this node will actiate only when the message is fully consumed,
        /// when it reached this node. If `false`, then this node will always activate if reached.
        expects_empty_message: bool,
        value: T,
    },
}

macro_rules! child_nodes {
    ( $child_nodes: expr, $message: expr, $arguments: expr ) => {
        for child_node in $child_nodes {
            if let Ok(arguments) = child_node.parse_impl($message, $arguments.clone()) {
                return Ok(arguments);
            }
        }
    };
}

#[derive(Debug, Clone)]
pub struct ParsedCommand<T> {
    /// The 'marker' value.
    pub value: T,
    pub arguments: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ParseError {
    /// Parsed arguments.
    pub parsed: Vec<String>,
    /// Error message.
    pub msg: String,
}

impl std::error::Error for ParseError {}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Parse error: {}", self.msg)
    }
}

type ParseResult<T> = Result<ParsedCommand<T>, ParseError>;

impl<T: Clone> CommandNode<T> {
    /// Attempts to parse the command.
    /// On success, returns the list of parsed arguments.
    pub fn parse(&self, message: &str) -> ParseResult<T> {
        self.parse_impl(message, Vec::new())
    }

    fn parse_impl(&self, message: &str, mut arguments: Vec<String>) -> ParseResult<T> {
        match self {
            CommandNode::Literal {
                literals,
                child_nodes,
            } => literals
                .iter()
                .find(|&literal| message.starts_with(literal))
                .ok_or_else(|| ParseError {
                    parsed: arguments.clone(),
                    msg: {
                        let mut msg = format!("Expected one of: ");
                        let mut literals = literals.iter();
                        if let Some(literal) = literals.next() {
                            msg += literal;
                        }
                        for literal in literals {
                            msg += ", ";
                            msg += literal;
                        }
                        msg += ". Found: \"";
                        msg += message;
                        msg += "\"";
                        msg
                    },
                })
                .and_then(|literal| {
                    let message = message[literal.len()..].trim();
                    child_nodes!(child_nodes, message, arguments);
                    // TODO: better error
                    Err(ParseError {
                        parsed: arguments,
                        msg: "No inner nodes matched".to_string(),
                    })
                }),

            CommandNode::Argument {
                argument_type,
                child_nodes,
            } => match argument_type {
                ArgumentType::Word => message.split_whitespace().next(),
                ArgumentType::Line => message.lines().next(),
                ArgumentType::Tail => {
                    if message.trim().is_empty() {
                        None
                    } else {
                        Some(message)
                    }
                }
            }
            .ok_or_else(|| ParseError {
                parsed: arguments.clone(),
                msg: format!("Expected a {} argument", argument_type),
            })
            .and_then(|argument| {
                let message = message[argument.len()..].trim();
                arguments.push(argument.to_owned());
                child_nodes!(child_nodes, message, arguments);
                // TODO: better error
                Err(ParseError {
                    parsed: arguments,
                    msg: "No inner nodes matched".to_string(),
                })
            }),

            CommandNode::ArgumentChoice {
                choices,
                child_nodes,
            } => choices
                .iter()
                .find(|choice| message.starts_with(*choice))
                .ok_or_else(|| ParseError {
                    parsed: arguments.clone(),
                    msg: {
                        let mut msg = format!("Expected one of: ");
                        let mut literals = choices.iter();
                        if let Some(literal) = literals.next() {
                            msg += literal;
                        }
                        for literal in literals {
                            msg += ", ";
                            msg += literal;
                        }
                        msg += ". Found: \"";
                        msg += message;
                        msg += "\"";
                        msg
                    },
                })
                .and_then(|choice| {
                    let message = message[choice.len()..].trim();
                    arguments.push(choice.to_owned());
                    child_nodes!(child_nodes, message, arguments);
                    // TODO: better error
                    Err(ParseError {
                        parsed: arguments,
                        msg: "No inner nodes matched".to_string(),
                    })
                }),

            CommandNode::Final {
                expects_empty_message,
                value,
            } => {
                if *expects_empty_message && !message.trim().is_empty() {
                    Err(ParseError {
                        parsed: arguments,
                        msg: format!("Did not expect any more arguments, found: {:?}", message),
                    })
                } else {
                    Ok(ParsedCommand {
                        value: value.clone(),
                        arguments,
                    })
                }
            }
        }
    }

    pub fn children_mut(&mut self) -> Option<&mut Vec<CommandNode<T>>> {
        match self {
            Self::Literal { child_nodes, .. } => Some(child_nodes),
            Self::Argument { child_nodes, .. } => Some(child_nodes),
            Self::ArgumentChoice { child_nodes, .. } => Some(child_nodes),
            Self::Final { .. } => None,
        }
    }
}
