use crate::{ArgumentType, CommandNode};

/// Helps to easier create trees of [CommandNode]. Makes construction of
/// deep command trees more concise. Conversely, constructing
/// wide trees takes about as much words as with using
/// constructors of [CommandNode]. Most helper functions in this struct
/// allow constructing only 1-wide trees (effectively lists),
/// but when you need to split a node, you can use the `split` function.
pub struct CommandBuilder<T: Clone> {
    nodes: Vec<CommandNode<T>>,
}

impl<T: Clone> CommandBuilder<T> {
    /// Initializes a builder
    pub fn new() -> Self {
        Self { nodes: Vec::new() }
    }

    /// Adds a literal node that accepts specific literals
    pub fn literal(mut self, literals: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.nodes.push(CommandNode::literal(literals, vec![]));
        self
    }

    /// Adds a literal node that expects an word argument
    pub fn word(mut self) -> Self {
        self.nodes
            .push(CommandNode::argument(ArgumentType::Word, vec![]));
        self
    }

    /// Adds a literal node that takes a line as an argument
    pub fn line(mut self) -> Self {
        self.nodes
            .push(CommandNode::argument(ArgumentType::Line, vec![]));
        self
    }

    /// Adds a literal node that accepts only certain literals
    /// and forwards the chosen one as an argument
    pub fn choice(mut self, choices: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.nodes
            .push(CommandNode::argument_choice(choices, vec![]));
        self
    }

    /// Finalizes the branch by adding a final node
    pub fn finalize(self, expects_empty_message: bool, value: T) -> CommandNode<T> {
        let final_node = CommandNode::final_node(expects_empty_message, value);
        fold_nodes(final_node, self.nodes.into_iter().rev())
    }

    /// Split the branch into several. Useful when several commands
    /// start from the same pattern.
    /// Assumes that at least one node has been inserted before
    pub fn split(self, children: impl IntoIterator<Item = CommandNode<T>>) -> CommandNode<T> {
        assert!(
            !self.nodes.is_empty(),
            "Expected at least one node in the builder"
        );

        let mut nodes = self.nodes.into_iter().rev();
        let mut final_node = nodes.next().unwrap();
        let child_nodes = final_node
            .children_mut()
            .expect("A final node cannot be in the middle");
        child_nodes.extend(children);
        fold_nodes(final_node, nodes)
    }
}

impl<T: Clone> Default for CommandBuilder<T> {
    fn default() -> Self {
        Self::new()
    }
}

fn fold_nodes<T: Clone>(
    final_node: CommandNode<T>,
    nodes: impl IntoIterator<Item = CommandNode<T>>,
) -> CommandNode<T> {
    (std::iter::once(final_node))
        .chain(nodes.into_iter())
        .reduce(|child, mut parent| {
            let children = parent
                .children_mut()
                .expect("A final node cannot be in the middle");
            children.push(child);
            parent
        })
        .unwrap()
}
