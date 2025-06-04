use crate::{
    eval::{Eval, EvaluationError},
    range::Range,
    values::ExpressionRange1dResult,
};
use std::marker::PhantomData;

pub struct DummyRange<T> {
    data: PhantomData<T>,
}

impl<T> DummyRange<T> {
    pub fn new() -> Self {
        Self { data: PhantomData }
    }
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
