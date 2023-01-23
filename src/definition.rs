use std::{collections::HashMap, fmt::Display};

use crate::expression::{ExpressionSyntaxTree, VariableSuperTrait};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EDefinition;

#[derive(Clone, Debug)]
pub struct Definition<T: VariableSuperTrait> {
    pub fun_map: HashMap<(T, usize), (Vec<T>, ExpressionSyntaxTree<T>)>,
    pub const_map: HashMap<T, ExpressionSyntaxTree<T>>,
}

impl Definition<String> {
    pub fn new(
        fun_map: HashMap<(String, usize), (Vec<String>, ExpressionSyntaxTree<String>)>,
        const_map: HashMap<String, ExpressionSyntaxTree<String>>,
    ) -> Self {
        Self { fun_map, const_map }
    }
}

impl Display for Definition<String> {
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
