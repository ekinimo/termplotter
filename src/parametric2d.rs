use std::fmt::Display;
use std::hash::Hash;
use std::str::Chars;

use parser_combinator::Parse;
use crate::expression::{ExpressionSyntaxTree, HasSameShape, VariableSuperTrait};
use crate::expression_parser::ExprParseResult;
use crate::parser_common::{Localization, Node, State, ParseErrors};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EParametric2D;

#[derive(Clone, Debug, PartialEq)]
pub struct Parametric2D<T: VariableSuperTrait> {
    pub x_expr: ExpressionSyntaxTree<T>,
    pub y_expr: ExpressionSyntaxTree<T>,
}

impl<T: VariableSuperTrait> Parametric2D<T> {
    pub fn new(x_expr: ExpressionSyntaxTree<T>, y_expr: ExpressionSyntaxTree<T>) -> Self {
        Self { x_expr, y_expr }
    }
}

impl<T: VariableSuperTrait> HasSameShape for Parametric2D<T> {
    fn has_same_shape(&self, other: &Self) -> bool {
        self.x_expr.has_same_shape(&other.x_expr) && self.y_expr.has_same_shape(&other.y_expr)
    }
}

impl<T: VariableSuperTrait> Display for Parametric2D<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.x_expr, self.y_expr)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Parametric2DResult {
    pub x_values: Vec<f64>,
    pub y_values: Vec<f64>,
}

impl Parametric2DResult {
    pub fn new(x_values: Vec<f64>, y_values: Vec<f64>) -> Self {
        Self { x_values, y_values }
    }
    
    pub fn len(&self) -> usize {
        std::cmp::min(self.x_values.len(), self.y_values.len())
    }
    
    pub fn is_empty(&self) -> bool {
        self.x_values.is_empty() || self.y_values.is_empty()
    }
}

impl HasSameShape for Parametric2DResult {
    fn has_same_shape(&self, other: &Self) -> bool {
        self == other
    }
}

impl Display for Parametric2DResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let pairs: Vec<String> = self.x_values.iter()
            .zip(self.y_values.iter())
            .map(|(x, y)| format!("({}, {})", x, y))
            .collect();
        write!(f, "[{}]", pairs.join(", "))
    }
}

pub type Parametric2DParseResult<'a> = Result<(Parametric2D<String>, State, Chars<'a>), ParseErrors>;

impl<'a> Parse<'a, Chars<'a>, State, Parametric2D<String>, ParseErrors> for EParametric2D {
    fn parse(&self, input: Chars<'a>, state: State) -> Parametric2DParseResult<'a> {
        use crate::expression::EExpression;
        use crate::parser_common::{LParen, RParen, Comma};
        
        // Parse format: (x_expr, y_expr)
        LParen
            .pair(EExpression)
            .pair(Comma)
            .pair(EExpression)
            .pair(RParen)
            .transform(|((((_, x_expr), _), y_expr), _)| Parametric2D::new(x_expr, y_expr))
            .with_error_using_state(|err, state, _| ParseErrors::Generic(state.start, state.end))
            .parse(input, state)
    }
}

use crate::eval::{Eval, EvaluationError};
use crate::context::Context;
use crate::values::ExpressionRange1dResult;

pub struct DummyParametric2D<T> {
    data: std::marker::PhantomData<T>,
}

impl<T> DummyParametric2D<T> {
    pub fn new() -> Self {
        Self { data: std::marker::PhantomData }
    }
}

impl<T, ContextV> Eval<Parametric2D<T>, ContextV, Parametric2DResult> for DummyParametric2D<Parametric2DResult>
where
    T: VariableSuperTrait + Hash + Eq + AsRef<str>,
    ContextV: Context<T>,
{
    fn eval(parametric: &Parametric2D<T>, context: &ContextV) -> Result<Parametric2DResult, EvaluationError> {
        use crate::eval_expression::DummyExpr;
        
        let x_result = DummyExpr::<ExpressionRange1dResult>::eval(&parametric.x_expr, context)?;
        let y_result = DummyExpr::<ExpressionRange1dResult>::eval(&parametric.y_expr, context)?;
        
        Ok(Parametric2DResult::new(x_result.0, y_result.0))
    }
}