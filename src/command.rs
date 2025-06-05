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
    Surface3D(ExpressionSyntaxTree<String>, String, String), // (expression, x_var, y_var)
}

#[derive(Clone, Debug)]
pub struct Command {
    pub definitions: Definition<String>,
    pub plot: PlotType,
    pub range: Range,
    pub y_range: Option<Range>, // For 3D plotting
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
            y_range: None,
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
            y_range: None,
            options,
        }
    }

    pub fn new_surface3d(
        definitions: Definition<String>,
        expr: ExpressionSyntaxTree<String>,
        x_var: String,
        y_var: String,
        x_range: Range,
        y_range: Range,
        options: CommandOptions,
    ) -> Self {
        Self {
            definitions,
            plot: PlotType::Surface3D(expr, x_var, y_var),
            range: x_range,
            y_range: Some(y_range),
            options,
        }
    }
}
