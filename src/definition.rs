use std::{collections::HashMap, fmt::Display};

use crate::expression::{ExpressionSyntaxTree, HasSameShape, VariableSuperTrait};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EDefinition;

#[derive(Clone, Debug)]
pub struct Definition<T: VariableSuperTrait> {
    pub fun_map: HashMap<(String, usize), (Vec<String>, ExpressionSyntaxTree<T>)>,
    pub const_map: HashMap<String, ExpressionSyntaxTree<T>>,
}

impl <T : std::fmt::Debug+Display+Clone+PartialEq+HasSameShape> Definition<T> {
    pub fn new(
        fun_map: HashMap<(String, usize), (Vec<String>, ExpressionSyntaxTree<T>)>,
        const_map: HashMap<String, ExpressionSyntaxTree<T>>,
    ) -> Self {
        Self { fun_map, const_map }
    }
}

impl <T> Display for Definition<T> where T: PartialEq + Display + Clone +std::fmt::Debug+  HasSameShape {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Functions :\n{}\nConstants :\n{}",
            self.fun_map
                .iter()
                .map(|(k, v)| format!(
                    "             {}    : {}  of arity {}   Vars : {}\n",
                    k.0,
                    v.1,
                    k.1,
                    v.0.clone().join(" ")
                ))
                .reduce(|x, y| format!("{x}{y}"))
                .unwrap(),
            self.const_map
                .iter()
                .map(|(k, v)| format!("             {k}    : {v}\n"))
                .reduce(|x, y| format!("{x}{y}"))
                .unwrap(),
        )
    }
}
