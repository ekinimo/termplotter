use crate::{definition::Definition, expression::ExpressionSyntaxTree, range::Range, command_options::CommandOptions};


#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ECommand;

#[derive(Clone, Debug)]
pub struct Command {
    pub definitions: Definition<String>,
    pub expr: ExpressionSyntaxTree<String>,
    pub range: Range,
    pub options: CommandOptions,
}

impl Command {
    pub fn new(
        definitions: Definition<String>,
        expr: ExpressionSyntaxTree<String>,
        range: Range,
        options: CommandOptions    ) -> Self {
        Self {
            definitions,
            expr,
            range,
            options,
        }
    }
}
