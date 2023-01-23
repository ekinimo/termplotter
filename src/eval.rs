use crate::parser_common::Localization;

pub trait Pow<T> {
    type Output;
    fn pow(self, rhs: T) -> Self::Output;
}


pub enum EvaluationError{
    GenericWithString(Localization,Localization,String)
}
pub trait Eval<Tree,Context,Output>{
    fn eval(tree:&Tree,context:&Context) -> Result<Output,EvaluationError>;
}


