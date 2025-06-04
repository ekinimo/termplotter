use crate::{
    command_options::CommandOptions, definition::Definition, expression::ExpressionSyntaxTree,
    parametric2d::Parametric2D, range::Range,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ECommand;

#[derive(Clone, Debug)]
pub enum PlotType {
    Expression(ExpressionSyntaxTree<String>),
    Parametric(Parametric2D<String>),
}

#[derive(Clone, Debug)]
pub struct Command {
    pub definitions: Definition<String>,
    pub plot: PlotType,
    pub range: Range,
    pub options: CommandOptions,
}

impl Command {
    pub fn new_expression(
        definitions: Definition<String>,
        expr: ExpressionSyntaxTree<String>,
        range: Range,
        options: CommandOptions,
    ) -> Self {
        Self {
            definitions,
            plot: PlotType::Expression(expr),
            range,
            options,
        }
    }

    pub fn new_parametric(
        definitions: Definition<String>,
        parametric: Parametric2D<String>,
        range: Range,
        options: CommandOptions,
    ) -> Self {
        Self {
            definitions,
            plot: PlotType::Parametric(parametric),
            range,
            options,
        }
    }
}
