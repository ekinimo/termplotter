use std::marker::PhantomData;

use crate::{
    context::{Context},
    eval::{self, Eval, EvaluationError, Pow},
    expression::{EExpression, ExpressionSyntaxTree, VariableSuperTrait},
    
    values::ExpressionRange1dResult,
};

struct DummyExpr<T> {
    data: PhantomData<T>,
}

impl<T,  ContextV> Eval<ExpressionSyntaxTree<T>, ContextV, ExpressionRange1dResult>
    for DummyExpr<ExpressionRange1dResult>
where
    T: VariableSuperTrait,
    ContextV: Context<T>,
{
    fn eval(
        tree: &ExpressionSyntaxTree<T>,
        context: &ContextV,
    ) -> Result<ExpressionRange1dResult, eval::EvaluationError> {
        let local_eval = |tr: &ExpressionSyntaxTree<T>| Self::eval(tr, context);

        match tree {
            ExpressionSyntaxTree::Variable(x) =>

                 local_eval(&context.get_variable(&x.value).ok_or(
                   EvaluationError::GenericWithString(
                    x.location.0,
                    x.location.1,
                    "Variable is not defined".into(),
                )
                
            )?),
            ExpressionSyntaxTree::Number(x) => Ok(x.value.into()),
            ExpressionSyntaxTree::Fun(x) => {
                todo!()
            }
            ExpressionSyntaxTree::Sum(x) => Ok(local_eval(&x.value.0)? + local_eval(&x.value.1)?),
            ExpressionSyntaxTree::Product(x) => {
                Ok(local_eval(&x.value.0)? * local_eval(&x.value.1)?)
            }
            ExpressionSyntaxTree::Exponent(x) => {
                Ok(local_eval(&x.value.0)?.pow(local_eval(&x.value.1)?))
            }
            ExpressionSyntaxTree::Subtraction(x) => {
                Ok(local_eval(&x.value.0)? - local_eval(&x.value.1)?)
            }
            ExpressionSyntaxTree::Division(x) => {
                Ok(local_eval(&x.value.0)? / local_eval(&x.value.1)?)
            }
            ExpressionSyntaxTree::Negation(x) => Ok(-local_eval(&x.value)?),
        }
    }
}
