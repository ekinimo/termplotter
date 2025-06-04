use std::{collections::HashMap, hash::Hash, marker::PhantomData};
//TODO this needs a major refactoring to make eval generic...

use crate::{
    context::Context,
    eval::{Eval, EvaluationError, Pow},
    expression::{ExpressionSyntaxTree, HasSameShape, VariableSuperTrait},
    parser_common::Localization,
    values::{ExpressionRange1dResult, PrimitiveBinary, PrimitiveTernary, PrimitiveUnary},
};

pub struct DummyExpr<T> {
    data: PhantomData<T>,
}

impl<T> DummyExpr<T> {
    pub fn new() -> Self {
        Self { data: PhantomData }
    }
}

impl<T, ContextV> Eval<ExpressionSyntaxTree<T>, ContextV, ExpressionRange1dResult>
    for DummyExpr<ExpressionRange1dResult>
where
    T: VariableSuperTrait + Hash + Eq + AsRef<str>,
    ContextV: Context<T>,
{
    fn eval(
        tree: &ExpressionSyntaxTree<T>,
        context: &ContextV,
    ) -> Result<ExpressionRange1dResult, EvaluationError> {
        evaluate_expression(tree, context)
    }
}

fn uniq_var_count<T: VariableSuperTrait + Hash + Eq>(
    e: &ExpressionSyntaxTree<T>,
    s: &mut std::collections::HashSet<T>,
) -> usize {
    match e {
        ExpressionSyntaxTree::Variable(x) => {
            if s.contains(&x.value) {
                0
            } else {
                s.insert(x.value.clone());
                1
            }
        }
        ExpressionSyntaxTree::Number(_) => 0,
        ExpressionSyntaxTree::Fun(x) => x
            .value
            .1
            .iter()
            .map(|t| uniq_var_count(&t, s))
            .reduce(|x, y| x + y)
            .unwrap_or(0),
        ExpressionSyntaxTree::Sum(x) => {
            uniq_var_count(&x.value.0, s) + uniq_var_count(&x.value.1, s)
        }
        ExpressionSyntaxTree::Product(x) => {
            uniq_var_count(&x.value.0, s) + uniq_var_count(&x.value.1, s)
        }
        ExpressionSyntaxTree::Exponent(x) => {
            uniq_var_count(&x.value.0, s) + uniq_var_count(&x.value.1, s)
        }
        ExpressionSyntaxTree::Division(x) => {
            uniq_var_count(&x.value.0, s) + uniq_var_count(&x.value.1, s)
        }
        ExpressionSyntaxTree::Subtraction(x) => {
            uniq_var_count(&x.value.0, s) + uniq_var_count(&x.value.1, s)
        }
        ExpressionSyntaxTree::Negation(x) => uniq_var_count(&x.value, s),
    }
}

fn eval_aux<T, ContextV>(
    e: &ExpressionSyntaxTree<ExpressionRange1dResult>,
    env: &HashMap<T, ExpressionRange1dResult>,
    context: &ContextV,
) -> Option<ExpressionRange1dResult>
where
    T: VariableSuperTrait + Hash + Eq + AsRef<str>,
    ContextV: Context<T>,
{
    match e {
        ExpressionSyntaxTree::Variable(x) => Some(x.value.clone()),
        ExpressionSyntaxTree::Number(x) => Some(x.value.into()),
        ExpressionSyntaxTree::Fun(x) => {
            let function_name = &x.value.0;
            let args = &x.value.1;
            let arity = args.len();

            let evaluated_args: Vec<_> = args
                .iter()
                .map(|arg| eval_aux(arg, env, context))
                .collect::<Option<Vec<_>>>()?;

            // Try primitive functions first
            match arity {
                1 => {
                    if let Some(func) = context.get_primitive_unary_function(function_name) {
                        return func.apply(evaluated_args[0].clone()).ok();
                    }
                }
                2 => {
                    if let Some(func) = context.get_primitive_binary_function(function_name) {
                        return func.apply(evaluated_args[0].clone(), evaluated_args[1].clone()).ok();
                    }
                }
                3 => {
                    if let Some(func) = context.get_primitive_ternary_function(function_name) {
                        return func.apply(
                            evaluated_args[0].clone(),
                            evaluated_args[1].clone(),
                            evaluated_args[2].clone(),
                        ).ok();
                    }
                }
                _ => {}
            }

            // Try user-defined functions
            if let (Some(function_expr), Some(param_names)) = (
                context.get_function_expr(function_name, arity),
                context.get_function_vars(function_name, arity),
            ) {
                if param_names.len() == evaluated_args.len() {
                    let mut parameter_env = HashMap::new();
                    for (param_name, arg_value) in param_names.iter().zip(evaluated_args.iter()) {
                        parameter_env.insert(*param_name, arg_value.clone());
                    }
                    return eval_with_param_substitution(&function_expr, &parameter_env, context);
                }
            }

            None
        }
        ExpressionSyntaxTree::Sum(x) => {
            let (l, r) = &x.value;
            Some(eval_aux(l, env, context)? + eval_aux(r, env, context)?)
        }
        ExpressionSyntaxTree::Product(x) => {
            let (l, r) = &x.value;
            Some(eval_aux(l, env, context)? * eval_aux(r, env, context)?)
        }
        ExpressionSyntaxTree::Exponent(x) => {
            let (l, r) = &x.value;
            Some(eval_aux(l, env, context)?.pow(eval_aux(r, env, context)?))
        }
        ExpressionSyntaxTree::Subtraction(x) => {
            let (l, r) = &x.value;
            Some(eval_aux(l, env, context)? - eval_aux(r, env, context)?)
        }
        ExpressionSyntaxTree::Division(x) => {
            let (l, r) = &x.value;
            let left = eval_aux(l, env, context)?;
            let right = eval_aux(r, env, context)?;
            Some(left / right)
        }
        ExpressionSyntaxTree::Negation(x) => Some(-eval_aux(&x.value, env, context)?),
    }
}

fn eval<T, ContextV>(
    e: &ExpressionSyntaxTree<T>,
    env: &HashMap<T, ExpressionRange1dResult>,
    context: &ContextV,
) -> Option<ExpressionRange1dResult>
where
    T: VariableSuperTrait + Hash + Eq + AsRef<str>,
    ContextV: Context<T>,
{
    match replace_var_kind(e, env)? {
        ExpressionSyntaxTree::Variable(x) => Some(x.value),
        ExpressionSyntaxTree::Number(x) => Some(x.value.into()),
        x @ ExpressionSyntaxTree::Fun(_)
        | x @ ExpressionSyntaxTree::Sum(_)
        | x @ ExpressionSyntaxTree::Product(_)
        | x @ ExpressionSyntaxTree::Exponent(_)
        | x @ ExpressionSyntaxTree::Subtraction(_)
        | x @ ExpressionSyntaxTree::Division(_)
        | x @ ExpressionSyntaxTree::Negation(_) => eval_aux(&x, env, context),
    }
}

fn replace_var_kind<T: VariableSuperTrait + Hash + Eq, T2: VariableSuperTrait>(
    e: &ExpressionSyntaxTree<T>,
    env: &HashMap<T, T2>,
) -> Option<ExpressionSyntaxTree<T2>> {
    let res = match e {
        ExpressionSyntaxTree::Variable(x) => {
            ExpressionSyntaxTree::variable(x.location.0, x.location.1, env.get(&x.value)?.clone())
        }

        ExpressionSyntaxTree::Number(x) => {
            ExpressionSyntaxTree::number(x.location.0, x.location.1, x.value)
        }
        ExpressionSyntaxTree::Fun(x) => {
            let vals: Vec<_> = x.value.1.iter().map(|t| replace_var_kind(t, env)).collect();
            let fails = vals.iter().filter(|x| x.is_none()).count();

            if fails == 0 {
                ExpressionSyntaxTree::fun(
                    x.location.0,
                    x.location.1,
                    x.value.0.clone(),
                    vals.into_iter().map(Option::unwrap).collect(),
                )
            } else {
                return None;
            }
        }
        ExpressionSyntaxTree::Sum(x) => ExpressionSyntaxTree::add(
            x.location.0,
            x.location.1,
            replace_var_kind(&x.value.0, env)?,
            replace_var_kind(&x.value.1, env)?,
        ),
        ExpressionSyntaxTree::Product(x) => ExpressionSyntaxTree::mul(
            x.location.0,
            x.location.1,
            replace_var_kind(&x.value.0, env)?,
            replace_var_kind(&x.value.1, env)?,
        ),
        ExpressionSyntaxTree::Exponent(x) => ExpressionSyntaxTree::exp(
            x.location.0,
            x.location.1,
            replace_var_kind(&x.value.0, env)?,
            replace_var_kind(&x.value.1, env)?,
        ),
        ExpressionSyntaxTree::Subtraction(x) => ExpressionSyntaxTree::sub(
            x.location.0,
            x.location.1,
            replace_var_kind(&x.value.0, env)?,
            replace_var_kind(&x.value.1, env)?,
        ),
        ExpressionSyntaxTree::Division(x) => ExpressionSyntaxTree::div(
            x.location.0,
            x.location.1,
            replace_var_kind(&x.value.0, env)?,
            replace_var_kind(&x.value.1, env)?,
        ),
        ExpressionSyntaxTree::Negation(x) => {
            ExpressionSyntaxTree::neg(x.location.0, x.location.1, replace_var_kind(&x.value, env)?)
        }
    };
    Some(res)
}

fn replace_expr<T: VariableSuperTrait>(
    source: &ExpressionSyntaxTree<T>,
    expr_to_find: &ExpressionSyntaxTree<T>,
    expr_to_paste: &ExpressionSyntaxTree<T>,
) -> Option<ExpressionSyntaxTree<T>> {
    if source.has_same_shape(expr_to_find) {
        return Some(expr_to_paste.clone());
    }
    match source {
        x @ ExpressionSyntaxTree::Variable(_) => Some(x.clone()),
        x @ ExpressionSyntaxTree::Number(_) => Some(x.clone()),
        ExpressionSyntaxTree::Fun(x) => {
            let vals: Vec<_> = x
                .value
                .1
                .iter()
                .map(|t| replace_expr(t, expr_to_find, expr_to_paste))
                .collect();
            let fails = vals.iter().filter(|x| x.is_none()).count();

            if fails == 0 {
                Some(ExpressionSyntaxTree::fun(
                    x.location.0,
                    x.location.1,
                    x.value.0.clone(),
                    vals.into_iter().map(Option::unwrap).collect(),
                ))
            } else {
                None
            }
        }
        ExpressionSyntaxTree::Sum(x) => Some(ExpressionSyntaxTree::add(
            x.location.0,
            x.location.1,
            replace_expr(&x.value.0, expr_to_find, expr_to_paste)?,
            replace_expr(&x.value.1, expr_to_find, expr_to_paste)?,
        )),
        ExpressionSyntaxTree::Product(x) => Some(ExpressionSyntaxTree::mul(
            x.location.0,
            x.location.1,
            replace_expr(&x.value.0, expr_to_find, expr_to_paste)?,
            replace_expr(&x.value.1, expr_to_find, expr_to_paste)?,
        )),
        ExpressionSyntaxTree::Exponent(x) => Some(ExpressionSyntaxTree::exp(
            x.location.0,
            x.location.1,
            replace_expr(&x.value.0, expr_to_find, expr_to_paste)?,
            replace_expr(&x.value.1, expr_to_find, expr_to_paste)?,
        )),
        ExpressionSyntaxTree::Subtraction(x) => Some(ExpressionSyntaxTree::sub(
            x.location.0,
            x.location.1,
            replace_expr(&x.value.0, expr_to_find, expr_to_paste)?,
            replace_expr(&x.value.1, expr_to_find, expr_to_paste)?,
        )),
        ExpressionSyntaxTree::Division(x) => Some(ExpressionSyntaxTree::div(
            x.location.0,
            x.location.1,
            replace_expr(&x.value.0, expr_to_find, expr_to_paste)?,
            replace_expr(&x.value.1, expr_to_find, expr_to_paste)?,
        )),
        ExpressionSyntaxTree::Negation(x) => Some(ExpressionSyntaxTree::neg(
            x.location.0,
            x.location.1,
            replace_expr(&x.value, expr_to_find, expr_to_paste)?,
        )),
    }
}

fn replace_var<T: VariableSuperTrait + Hash + Eq>(
    e: &ExpressionSyntaxTree<T>,
    env: &HashMap<T, ExpressionSyntaxTree<T>>,
) -> Option<ExpressionSyntaxTree<T>> {
    let res = match e {
        ExpressionSyntaxTree::Variable(x) => env.get(&x.value)?.clone(),

        ExpressionSyntaxTree::Number(x) => {
            ExpressionSyntaxTree::number(x.location.0, x.location.1, x.value)
        }
        ExpressionSyntaxTree::Fun(x) => {
            let vals: Vec<_> = x.value.1.iter().map(|t| replace_var(t, env)).collect();
            let fails = vals.iter().filter(|x| x.is_none()).count();

            if fails == 0 {
                ExpressionSyntaxTree::fun(
                    x.location.0,
                    x.location.1,
                    x.value.0.clone(),
                    vals.into_iter().map(Option::unwrap).collect(),
                )
            } else {
                return None;
            }
        }
        ExpressionSyntaxTree::Sum(x) => ExpressionSyntaxTree::add(
            x.location.0,
            x.location.1,
            replace_var(&x.value.0, env)?,
            replace_var(&x.value.1, env)?,
        ),
        ExpressionSyntaxTree::Product(x) => ExpressionSyntaxTree::mul(
            x.location.0,
            x.location.1,
            replace_var(&x.value.0, env)?,
            replace_var(&x.value.1, env)?,
        ),
        ExpressionSyntaxTree::Exponent(x) => ExpressionSyntaxTree::exp(
            x.location.0,
            x.location.1,
            replace_var(&x.value.0, env)?,
            replace_var(&x.value.1, env)?,
        ),
        ExpressionSyntaxTree::Subtraction(x) => ExpressionSyntaxTree::sub(
            x.location.0,
            x.location.1,
            replace_var(&x.value.0, env)?,
            replace_var(&x.value.1, env)?,
        ),
        ExpressionSyntaxTree::Division(x) => ExpressionSyntaxTree::div(
            x.location.0,
            x.location.1,
            replace_var(&x.value.0, env)?,
            replace_var(&x.value.1, env)?,
        ),
        ExpressionSyntaxTree::Negation(x) => {
            ExpressionSyntaxTree::neg(x.location.0, x.location.1, replace_var(&x.value, env)?)
        }
    };
    Some(res)
}

fn evaluate_expression<T, ContextV>(
    e: &ExpressionSyntaxTree<T>,
    context: &ContextV,
) -> Result<ExpressionRange1dResult, EvaluationError>
where
    T: VariableSuperTrait + Hash + Eq + AsRef<str>,
    ContextV: Context<T>,
{
    handle_with_context(e, context)
}

fn handle_with_context<T, ContextV>(
    e: &ExpressionSyntaxTree<T>,
    context: &ContextV,
) -> Result<ExpressionRange1dResult, EvaluationError>
where
    T: VariableSuperTrait + Hash + Eq + AsRef<str>,
    ContextV: Context<T>,
{
    match e {
        ExpressionSyntaxTree::Variable(x) => match context.get_variable(&x.value) {
            Some(var_expr) => handle_with_context(&var_expr, context),
            None => Err(EvaluationError::GenericWithString(
                x.location.0,
                x.location.1,
                format!("Variable '{}' is not defined", x.value),
            )),
        },

        ExpressionSyntaxTree::Number(x) => Ok(ExpressionRange1dResult::from(x.value)),

        ExpressionSyntaxTree::Fun(x) => {
            let function_name = &x.value.0;
            let args = &x.value.1;
            let arity = args.len();

            let evaluated_args: Result<Vec<_>, _> = args
                .iter()
                .map(|arg| handle_with_context(arg, context))
                .collect();
            let evaluated_args = evaluated_args?;

            match arity {
                1 => {
                    if let Some(func) = context.get_primitive_unary_function(function_name) {
                        func.apply(evaluated_args[0].clone()).map_err(|msg| {
                            EvaluationError::GenericWithString(x.location.0, x.location.1, msg)
                        })
                    } else {
                        evaluate_user_defined_function(
                            function_name,
                            evaluated_args,
                            arity,
                            context,
                            x.location,
                        )
                    }
                }
                2 => {
                    if let Some(func) = context.get_primitive_binary_function(function_name) {
                        func.apply(evaluated_args[0].clone(), evaluated_args[1].clone())
                            .map_err(|msg| {
                                EvaluationError::GenericWithString(x.location.0, x.location.1, msg)
                            })
                    } else {
                        evaluate_user_defined_function(
                            function_name,
                            evaluated_args,
                            arity,
                            context,
                            x.location,
                        )
                    }
                }
                3 => {
                    if let Some(func) = context.get_primitive_ternary_function(function_name) {
                        func.apply(
                            evaluated_args[0].clone(),
                            evaluated_args[1].clone(),
                            evaluated_args[2].clone(),
                        )
                        .map_err(|msg| {
                            EvaluationError::GenericWithString(x.location.0, x.location.1, msg)
                        })
                    } else {
                        evaluate_user_defined_function(
                            function_name,
                            evaluated_args,
                            arity,
                            context,
                            x.location,
                        )
                    }
                }
                _ => evaluate_user_defined_function(
                    function_name,
                    evaluated_args,
                    arity,
                    context,
                    x.location,
                ),
            }
        }

        ExpressionSyntaxTree::Sum(x) => {
            let left = handle_with_context(&x.value.0, context)?;
            let right = handle_with_context(&x.value.1, context)?;
            Ok(left + right)
        }
        ExpressionSyntaxTree::Product(x) => {
            let left = handle_with_context(&x.value.0, context)?;
            let right = handle_with_context(&x.value.1, context)?;
            Ok(left * right)
        }
        ExpressionSyntaxTree::Exponent(x) => {
            let left = handle_with_context(&x.value.0, context)?;
            let right = handle_with_context(&x.value.1, context)?;
            Ok(left.pow(right))
        }
        ExpressionSyntaxTree::Subtraction(x) => {
            let left = handle_with_context(&x.value.0, context)?;
            let right = handle_with_context(&x.value.1, context)?;
            Ok(left - right)
        }
        ExpressionSyntaxTree::Division(x) => {
            let left = handle_with_context(&x.value.0, context)?;
            let right = handle_with_context(&x.value.1, context)?;
            Ok(left / right)
        }
        ExpressionSyntaxTree::Negation(x) => {
            let operand = handle_with_context(&x.value, context)?;
            Ok(-operand)
        }
    }
}

fn eval_with_param_substitution<T, ContextV>(
    expr: &ExpressionSyntaxTree<T>,
    param_env: &HashMap<&str, ExpressionRange1dResult>,
    context: &ContextV,
) -> Option<ExpressionRange1dResult>
where
    T: VariableSuperTrait + Hash + Eq + AsRef<str>,
    ContextV: Context<T>,
{
    match expr {
        ExpressionSyntaxTree::Variable(x) => {
            if let Some(value) = param_env.get(x.value.as_ref()) {
                Some(value.clone())
            } else if let Some(var_expr) = context.get_variable(&x.value) {
                eval_with_param_substitution(&var_expr, param_env, context)
            } else {
                None
            }
        }
        ExpressionSyntaxTree::Number(x) => Some(x.value.into()),
        ExpressionSyntaxTree::Fun(x) => {
            let function_name = &x.value.0;
            let args = &x.value.1;
            let arity = args.len();

            let evaluated_args: Vec<_> = args
                .iter()
                .map(|arg| eval_with_param_substitution(arg, param_env, context))
                .collect::<Option<Vec<_>>>()?;

            match arity {
                1 => {
                    if let Some(func) = context.get_primitive_unary_function(function_name) {
                        func.apply(evaluated_args[0].clone()).ok()
                    } else if let (Some(function_expr), Some(nested_param_names)) = (
                        context.get_function_expr(function_name, arity),
                        context.get_function_vars(function_name, arity),
                    ) {
                        if nested_param_names.len() == evaluated_args.len() {
                            let mut nested_param_env = HashMap::new();
                            for (param_name, arg_value) in nested_param_names.iter().zip(evaluated_args.iter()) {
                                nested_param_env.insert(*param_name, arg_value.clone());
                            }
                            eval_with_param_substitution(&function_expr, &nested_param_env, context)
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                }
                2 => {
                    if let Some(func) = context.get_primitive_binary_function(function_name) {
                        func.apply(evaluated_args[0].clone(), evaluated_args[1].clone()).ok()
                    } else if let (Some(function_expr), Some(nested_param_names)) = (
                        context.get_function_expr(function_name, arity),
                        context.get_function_vars(function_name, arity),
                    ) {
                        if nested_param_names.len() == evaluated_args.len() {
                            let mut nested_param_env = HashMap::new();
                            for (param_name, arg_value) in nested_param_names.iter().zip(evaluated_args.iter()) {
                                nested_param_env.insert(*param_name, arg_value.clone());
                            }
                            eval_with_param_substitution(&function_expr, &nested_param_env, context)
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                }
                3 => {
                    if let Some(func) = context.get_primitive_ternary_function(function_name) {
                        func.apply(
                            evaluated_args[0].clone(),
                            evaluated_args[1].clone(),
                            evaluated_args[2].clone(),
                        ).ok()
                    } else if let (Some(function_expr), Some(nested_param_names)) = (
                        context.get_function_expr(function_name, arity),
                        context.get_function_vars(function_name, arity),
                    ) {
                        if nested_param_names.len() == evaluated_args.len() {
                            let mut nested_param_env = HashMap::new();
                            for (param_name, arg_value) in nested_param_names.iter().zip(evaluated_args.iter()) {
                                nested_param_env.insert(*param_name, arg_value.clone());
                            }
                            eval_with_param_substitution(&function_expr, &nested_param_env, context)
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                }
                _ => {
                    if let (Some(function_expr), Some(nested_param_names)) = (
                        context.get_function_expr(function_name, arity),
                        context.get_function_vars(function_name, arity),
                    ) {
                        if nested_param_names.len() == evaluated_args.len() {
                            let mut nested_param_env = HashMap::new();
                            for (param_name, arg_value) in nested_param_names.iter().zip(evaluated_args.iter()) {
                                nested_param_env.insert(*param_name, arg_value.clone());
                            }
                            eval_with_param_substitution(&function_expr, &nested_param_env, context)
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                }
            }
        }
        ExpressionSyntaxTree::Sum(x) => {
            let left = eval_with_param_substitution(&x.value.0, param_env, context)?;
            let right = eval_with_param_substitution(&x.value.1, param_env, context)?;
            Some(left + right)
        }
        ExpressionSyntaxTree::Product(x) => {
            let left = eval_with_param_substitution(&x.value.0, param_env, context)?;
            let right = eval_with_param_substitution(&x.value.1, param_env, context)?;
            Some(left * right)
        }
        ExpressionSyntaxTree::Exponent(x) => {
            let left = eval_with_param_substitution(&x.value.0, param_env, context)?;
            let right = eval_with_param_substitution(&x.value.1, param_env, context)?;
            Some(left.pow(right))
        }
        ExpressionSyntaxTree::Subtraction(x) => {
            let left = eval_with_param_substitution(&x.value.0, param_env, context)?;
            let right = eval_with_param_substitution(&x.value.1, param_env, context)?;
            Some(left - right)
        }
        ExpressionSyntaxTree::Division(x) => {
            let left = eval_with_param_substitution(&x.value.0, param_env, context)?;
            let right = eval_with_param_substitution(&x.value.1, param_env, context)?;
            Some(left / right)
        }
        ExpressionSyntaxTree::Negation(x) => {
            let operand = eval_with_param_substitution(&x.value, param_env, context)?;
            Some(-operand)
        }
    }
}

fn evaluate_user_defined_function<T, ContextV>(
    function_name: &str,
    evaluated_args: Vec<ExpressionRange1dResult>,
    arity: usize,
    context: &ContextV,
    location: (Localization, Localization),
) -> Result<ExpressionRange1dResult, EvaluationError>
where
    T: VariableSuperTrait + Hash + Eq + AsRef<str>,
    ContextV: Context<T>,
{
    if let (Some(function_expr), Some(param_names)) = (
        context.get_function_expr(function_name, arity),
        context.get_function_vars(function_name, arity),
    ) {
        if param_names.len() == evaluated_args.len() {
            let mut parameter_env = HashMap::new();
            for (param_name, arg_value) in param_names.iter().zip(evaluated_args.iter()) {
                parameter_env.insert(*param_name, arg_value.clone());
            }

            if let Some(result) = eval_with_param_substitution(&function_expr, &parameter_env, context) {
                Ok(result)
            } else {
                Err(EvaluationError::GenericWithString(
                    location.0,
                    location.1,
                    format!("Failed to evaluate user-defined function '{}'", function_name),
                ))
            }
        } else {
            Err(EvaluationError::GenericWithString(
                location.0,
                location.1,
                format!(
                    "Parameter count mismatch for function '{}': expected {}, got {}",
                    function_name,
                    param_names.len(),
                    evaluated_args.len()
                ),
            ))
        }
    } else {
        Err(EvaluationError::GenericWithString(
            location.0,
            location.1,
            format!("Function '{}' with arity {} is not defined", function_name, arity),
        ))
    }
}

pub fn eval_with_hashmap<T: VariableSuperTrait + Hash + Eq + AsRef<str>, ContextV: Context<T>>(
    e: &ExpressionSyntaxTree<T>,
    env: &HashMap<T, ExpressionRange1dResult>,
    context: &ContextV,
) -> Option<ExpressionRange1dResult> {
    eval(e, env, context)
}
