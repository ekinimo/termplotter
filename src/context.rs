use std::{marker::PhantomData, hash::Hash};

use crate::{expression::{ExpressionSyntaxTree, VariableSuperTrait}, values::{Value, make_primitive_ternary, make_primitive_binary, make_primitive_unary, PrimitiveUnary, PrimitiveBinary, PrimitiveTernary, ExpressionRange1dResult}, definition::Definition};

pub trait Context<T:VariableSuperTrait>{
    fn get_variable(&self,var:&T) ->  Option<ExpressionSyntaxTree<T>>;
     fn get_function_expr(&self,name:&T,arity:usize) -> Option<ExpressionSyntaxTree<T>>;
     fn get_function_vars(&self,name:&T,arity:usize) -> Option<Vec<T>>;
     fn get_primitive_unary_function(&self,name:impl Into<String>) -> Option<&dyn PrimitiveUnary>{

        
        match name.into().as_str(){
            "abs" => Some(make_primitive_unary(&f64::abs)),
            
            _ => None
        }
        

    }
    fn get_primitive_binary_function(&self,name:impl Into<String>) -> Option<&dyn PrimitiveBinary>{
        match name.into().as_str(){
            "max" => Some(make_primitive_binary(&f64::max)),
            
            _ => None
        }
        
    }

    fn get_primitive_ternary_function(&self,name:impl Into<String>) -> Option<&dyn PrimitiveTernary>{
        match name.into().as_str(){
            "clamp" => Some(make_primitive_ternary(&f64::clamp)),
            _ => None
        }

    }
}



impl<T:VariableSuperTrait+Eq+Hash> Context<T> for Definition<T>{
    fn get_variable(&self,var:&T) -> Option< ExpressionSyntaxTree<T>> {
        self.const_map.get(var).cloned()
    }

    fn get_function_expr(&self,name:&T,arity:usize) -> Option<ExpressionSyntaxTree<T>> {
        self.fun_map.get(&(name.to_owned(),arity)).map(|x| x.1)
    }

    fn get_function_vars(&self,name:&T,arity:usize) -> Option<Vec<T>> {
        self.fun_map.get(&(name.to_owned(),arity)).map(|x| x.0)
    }
}
