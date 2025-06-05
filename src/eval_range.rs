use crate::{
    eval::{Eval, EvaluationError},
    range::Range,
    values::{ExpressionRange1dResult, ExpressionRange2d},
};
use std::marker::PhantomData;

pub struct DummyRange<T> {
    data: PhantomData<T>,
}


impl<Context> Eval<Range, Context, ExpressionRange1dResult>
    for DummyRange<ExpressionRange1dResult>
{
    fn eval(tree: &Range, _context: &Context) -> Result<ExpressionRange1dResult, EvaluationError> {
        match tree {
            Range::Numeric(x) => {
                ExpressionRange1dResult::create_with_step(x.value.1, x.value.2, 1000.0)
            }
            Range::NumericStep(x) => {
                ExpressionRange1dResult::create_with_step(x.value.1, x.value.2, x.value.3)
            }
            Range::FileBare(x) => ExpressionRange1dResult::create_from_file(x.clone().value.1)
                .map_err(|a| a(x.location.0, x.location.1)),
            Range::FileCol(x) => {
                ExpressionRange1dResult::create_from_file_col(x.clone().value.1, x.clone().value.2)
                    .map_err(|a| a(x.location.0, x.location.1))
            }
        }
    }
}

// Structure to hold 2D range data for 3D surface evaluation
pub struct Range2D {
    pub x_range: Range,
    pub y_range: Range,
}


impl<Context> Eval<Range2D, Context, ExpressionRange2d> for DummyRange<ExpressionRange2d> {
    fn eval(tree: &Range2D, context: &Context) -> Result<ExpressionRange2d, EvaluationError> {
        let x_result = DummyRange::<ExpressionRange1dResult>::eval(&tree.x_range, context)?;
        let y_result = DummyRange::<ExpressionRange1dResult>::eval(&tree.y_range, context)?;
        
        // Create a flattened 2D range: [x1,x2,...,xn, y1,y2,...,ym, x1,x2,...,xn, y1,y2,...,ym, ...]
        // This represents a meshgrid where each (x,y) pair can be evaluated
        let mut data = Vec::new();
        for &y_val in &y_result.0 {
            for &x_val in &x_result.0 {
                data.push(x_val);
                data.push(y_val);
            }
        }
        
        Ok(ExpressionRange2d(data, x_result.0.len(), y_result.0.len()))
    }
}
