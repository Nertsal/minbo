use crate::{ArgumentType, CommandNode};

impl<T: Clone> CommandNode<T> {
    pub fn literal(
        literals: impl IntoIterator<Item = impl Into<String>>,
        children: Vec<CommandNode<T>>,
    ) -> Self {
        Self::Literal {
            literals: literals.into_iter().map(|literal| literal.into()).collect(),
            child_nodes: children,
        }
    }

    pub fn argument(argument_type: ArgumentType, children: Vec<CommandNode<T>>) -> Self {
        Self::Argument {
            argument_type,
            child_nodes: children,
        }
    }

    pub fn argument_choice(
        choices: impl IntoIterator<Item = impl Into<String>>,
        children: Vec<CommandNode<T>>,
    ) -> Self {
        Self::ArgumentChoice {
            choices: choices.into_iter().map(|choice| choice.into()).collect(),
            child_nodes: children,
        }
    }

    pub fn final_node(expects_empty_message: bool, value: T) -> Self {
        Self::Final {
            expects_empty_message,
            value,
        }
    }
}
