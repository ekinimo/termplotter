use std::hash::Hash;

use crate::{
    definition::Definition,
    expression::{ExpressionSyntaxTree, VariableSuperTrait},
    values::{
        make_primitive_binary, make_primitive_ternary, make_primitive_unary, PrimitiveBinary,
        PrimitiveTernary, PrimitiveUnary, ExpressionRange1dResult,
    },
};

pub trait Context<T: VariableSuperTrait> {
    fn get_variable(&self, var: impl AsRef<str>) -> Option<ExpressionSyntaxTree<T>>;
    fn get_function_expr(
        &self,
        name: impl AsRef<str>,
        arity: usize,
    ) -> Option<ExpressionSyntaxTree<T>>;
    fn get_function_vars(&self, name: impl AsRef<str>, arity: usize) -> Option<Vec<&str>>;
    fn get_primitive_unary_function(
        &self,
        name: impl AsRef<str>,
    ) -> Option<Box<dyn PrimitiveUnary<ExpressionRange1dResult>>>;
    fn get_primitive_binary_function(
        &self,
        name: impl AsRef<str>,
    ) -> Option<Box<dyn PrimitiveBinary<ExpressionRange1dResult>>>;
    fn get_primitive_ternary_function(
        &self,
        name: impl AsRef<str>,
    ) -> Option<Box<dyn PrimitiveTernary<ExpressionRange1dResult>>>;
}

impl<T: VariableSuperTrait + Eq + Hash> Context<T> for Definition<T> {
    fn get_variable(&self, var: impl AsRef<str>) -> Option<ExpressionSyntaxTree<T>> {
        self.const_map.get(var.as_ref()).cloned()
    }

    fn get_function_expr(
        &self,
        name: impl AsRef<str>,
        arity: usize,
    ) -> Option<ExpressionSyntaxTree<T>> {
        self.fun_map
            .get(&(name.as_ref().to_string(), arity))
            .map(|x| x.1.clone())
    }

    fn get_function_vars(&self, name: impl AsRef<str>, arity: usize) -> Option<Vec<&str>> {
        self.fun_map
            .get(&(name.as_ref().to_string(), arity))
            .map(|x| x.0.iter().map(|a| a.as_str()).collect())
    }

    fn get_primitive_unary_function(
        &self,
        name: impl AsRef<str>,
    ) -> Option<Box<dyn PrimitiveUnary<ExpressionRange1dResult>>> {
        match name.as_ref() {
            "abs" => Some(make_primitive_unary(f64::abs)),

            "sin" => Some(make_primitive_unary(f64::sin)),
            "asin" => Some(make_primitive_unary(f64::asin)),
            "cos" => Some(make_primitive_unary(f64::cos)),
            "acos" => Some(make_primitive_unary(f64::acos)),
            "tan" => Some(make_primitive_unary(f64::tan)),
            "tanh" => Some(make_primitive_unary(f64::tanh)),
            "atan" => Some(make_primitive_unary(f64::atan)),
            "sinh" => Some(make_primitive_unary(f64::sinh)),
            "asinh" => Some(make_primitive_unary(f64::asinh)),
            "cosh" => Some(make_primitive_unary(f64::cosh)),
            "acosh" => Some(make_primitive_unary(f64::acosh)),
            "atanh" => Some(make_primitive_unary(f64::atanh)),

            "ln" => Some(make_primitive_unary(f64::ln)),
            "log10" => Some(make_primitive_unary(f64::log10)),
            "log2" => Some(make_primitive_unary(f64::log2)),
            "sqrt" => Some(make_primitive_unary(f64::sqrt)),
            "cbrt" => Some(make_primitive_unary(f64::cbrt)),
            "exp" => Some(make_primitive_unary(f64::exp)),
            "erf" => Some(make_primitive_unary(f64::erf)),
            "erfc" => Some(make_primitive_unary(f64::erfc)),
            "gamma" => Some(make_primitive_unary(f64::gamma)),

            "fract" => Some(make_primitive_unary(f64::fract)),
            "floor" => Some(make_primitive_unary(f64::floor)),
            "ceil" => Some(make_primitive_unary(f64::ceil)),
            "round" => Some(make_primitive_unary(f64::round)),

            "nextup" => Some(make_primitive_unary(f64::next_up)),
            "nextdown" => Some(make_primitive_unary(f64::next_down)),

            "recip" => Some(make_primitive_unary(f64::recip)),
            "todegrees" => Some(make_primitive_unary(f64::to_degrees)),
            "toradians" => Some(make_primitive_unary(f64::to_radians)),
            "signum" => Some(make_primitive_unary(f64::signum)),
            _ => None,
        }
    }

    fn get_primitive_binary_function(
        &self,
        name: impl AsRef<str>,
    ) -> Option<Box<dyn PrimitiveBinary<ExpressionRange1dResult>>> {
        match name.as_ref() {
            "log" => Some(make_primitive_binary(f64::log)),
            "hypot" => Some(make_primitive_binary(f64::hypot)),
            "max" => Some(make_primitive_binary(f64::max)),
            "min" => Some(make_primitive_binary(f64::min)),
            "pow" => Some(make_primitive_binary(f64::powf)),
            "atan2" => Some(make_primitive_binary(f64::atan2)),
            "midpoint" => Some(make_primitive_binary(f64::midpoint)),
            _ => None,
        }
    }

    fn get_primitive_ternary_function(
        &self,
        name: impl AsRef<str>,
    ) -> Option<Box<dyn PrimitiveTernary<ExpressionRange1dResult>>> {
        match name.as_ref() {
            "clamp" => Some(make_primitive_ternary(f64::clamp)),
            _ => None,
        }
    }
}
