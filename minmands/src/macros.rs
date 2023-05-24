#[macro_export]
macro_rules! command {
    // Final
    ($empty:expr, $value:expr) => {{
        $crate::CommandNode::final_node($empty, $value)
    }};
    // Literal
    ($($literals:literal),+; $($tail:tt)*) => {{
        let children = vec![$crate::command!($($tail)*)];
        $crate::CommandNode::literal([$($literals),+], children)
    }};
    // Choice
    ($($choices:literal)|+; $($tail:tt)*) => {{
        let children = vec![$crate::command!($($tail)*)];
        $crate::CommandNode::argument_choice([$($choices),+], children)
    }};
    // Argument word
    (word; $($tail:tt)*) => {{
        let children = vec![$crate::command!($($tail)*)];
        $crate::CommandNode::argument($crate::ArgumentType::Word, children)
    }};
    // Argument line
    (line; $($tail:tt)*) => {{
        let children = vec![$crate::command!($($tail)*)];
        $crate::CommandNode::argument($crate::ArgumentType::Line, children)
    }};
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_command_macro() {
        // `CommandNode::final_node(...)`
        command!(true, ());

        // `CommandBuilder::new().literal(["!help"]).finalize(...)`
        command!(
            "!help";
            true, ()
        );

        // `CommandBuilder::new().literal(["!backup"]).choice(["create", "load"]).finalize(...)`
        command!(
            "!backup";
            "create" | "load";
            true, ()
        );

        // `CommandBuilder::new().literal(["!hello"]).word().finalize(...)`
        command!(
            "!hello";
            word;
            true, ()
        );

        // `CommandBuilder::new().literal(["!echo"]).line().finalize(...)`
        command!(
            "!echo";
            line;
            true, ()
        );
    }
}
