#![feature(box_patterns)]

use std::collections::HashMap;
use std::collections::HashSet;
use std::env;
use std::fmt::format;
use std::fmt::Debug;
use std::fmt::Display;
use std::fmt::Formatter;
use std::hash::Hash;
use std::io::stdin;
use std::marker::PhantomData;
use std::ops::Add;
use std::ops::Div;
use std::ops::Mul;
use std::ops::Neg;
use std::ops::Sub;

use std::str::Chars;
use std::usize;

use definition::Definition;
use definition::EDefinition;
use definition_parser::DefinitionParseResult;
use expression::EExpression;
use expression::ExpressionSyntaxTree;
use expression::HasSameShape;
use expression::VariableSuperTrait;
use parser_combinator::Parse;
use parser_combinator::ParseResult;
use parser_common::ParseErrors;
//use parser_common::Parsable;

use crate::parser_common::State;


mod parser_common;
mod definition;
mod definition_parser;
mod expression;
mod expression_parser;
mod range;
mod range_parser;
mod command_options;
mod command_options_parser;

mod command;
mod command_parser;

mod values;
mod context;

mod eval;
mod eval_expression;
mod eval_range;
mod eval_command;

fn uniq_var_count<T: VariableSuperTrait + Hash + Eq>(
    e: &ExpressionSyntaxTree<T>,
    s: &mut HashSet<T>,
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
            .unwrap(),
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




/*
fn  replace_var_with_number<T:VariableSuperTrait>(e: ExpressionSyntaxTree<T>, env: &HashMap<T, f32>) -> ExpressionSyntaxTree<T> {
    match e {
        ExpressionSyntaxTree::Variable(x) =>
            match env.get(&x.value) {
            Some(n) => {
                ExpressionSyntaxTree::number(x.location.0, x.location.1, *n)},
            None => ExpressionSyntaxTree::variable(x.location.0, x.location.1, x.value),
        },
        x @ ExpressionSyntaxTree::Number(_) => x,
        ExpressionSyntaxTree::Fun(x) => ExpressionSyntaxTree::fun(
            x.location.0,
            x.location.1,
            x.value.0,
            x.value
                .1
                .into_iter()
                .map(move |t| replace_var(t, env))
                .collect(),
        ),
        ExpressionSyntaxTree::Sum(x) => ExpressionSyntaxTree::add(
            x.location.0,
            x.location.1,
            replace_var(x.value.0, env),
            replace_var(x.value.1, env),
        ),
        ExpressionSyntaxTree::Product(x) => ExpressionSyntaxTree::mul(
            x.location.0,
            x.location.1,
            replace_var(x.value.0, env),
            replace_var(x.value.1, env),
        ),
        ExpressionSyntaxTree::Exponent(x) => ExpressionSyntaxTree::exp(
            x.location.0,
            x.location.1,
            replace_var(x.value.0, env),
            replace_var(x.value.1, env),
        ),
        ExpressionSyntaxTree::Subtraction(x) => ExpressionSyntaxTree::sub(
            x.location.0,
            x.location.1,
            replace_var(x.value.0, env),
            replace_var(x.value.1, env),
        ),
        ExpressionSyntaxTree::Division(x) => ExpressionSyntaxTree::div(
            x.location.0,
            x.location.1,
            replace_var(x.value.0, env),
            replace_var(x.value.1, env),
        ),
        ExpressionSyntaxTree::Negation(x) => {
            ExpressionSyntaxTree::neg(x.location.0, x.location.1, replace_var(x.value, env))
        }
    }
}


/*

*/

trait Pow<T> {
    type Output;
    fn pow(self, rhs: T) -> Self::Output;
}

fn eval_aux<T1, T2>(e: &ExpressionSyntaxTree<T2>, env: &HashMap<T1, T2>) -> Option<T2>
where
    T1: VariableSuperTrait + Hash + Eq,
    T2: VariableSuperTrait
        + Add<T2, Output = T2>
        + Mul<T2, Output = T2>
        + Div<T2, Output = T2>
        + Sub<T2, Output = T2>
        + Pow<T2, Output = T2>
        + Neg<Output = T2>
        + From<f64>,
{
    match e {
        ExpressionSyntaxTree::Variable(x) => Some(x.value.to_owned()),
        ExpressionSyntaxTree::Number(x) => Some(x.value.into()),
        ExpressionSyntaxTree::Fun(_) => todo!(),
        ExpressionSyntaxTree::Sum(x) => {
            let (l, r) = x.value.to_owned();
            Some(eval_aux(&l, env)? + eval_aux(&r, env)?)
        }
        ExpressionSyntaxTree::Product(x) => {
            let (l, r) = x.value.to_owned();
            Some(eval_aux(&l, env)? * eval_aux(&r, env)?)
        }
        ExpressionSyntaxTree::Exponent(x) => {
            let (l, r) = x.value.to_owned();
            Some(eval_aux(&l, env)?.pow(eval_aux(&r, env)?))
        },
        ExpressionSyntaxTree::Subtraction(x) => {
            let (l, r) = x.value.to_owned();
            Some(eval_aux(&l, env)? - eval_aux(&r, env)?)
        }
        ExpressionSyntaxTree::Division(x) => {
            let (l, r) = x.value.to_owned();
            Some(eval_aux(&l, env)? / eval_aux(&r, env)?)
        }
        ExpressionSyntaxTree::Negation(x) => {
            let l = x.value.to_owned();
            Some(-eval_aux(&l, env)?)
        }
    }
}

fn eval<T1: VariableSuperTrait + Hash + Eq, T2>(
    e: &ExpressionSyntaxTree<T1>,
    env: &HashMap<T1, T2>,
) -> Option<T2>
where
    T2: VariableSuperTrait
        + Add<T2, Output = T2>
        + Mul<T2, Output = T2>
        + Div<T2, Output = T2>
        + Sub<T2, Output = T2>
        + Pow<T2, Output = T2>
        + Neg<Output = T2>
        + From<f64>,
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
        | x @ ExpressionSyntaxTree::Negation(_) => eval_aux(&x, env),
    }
}

fn replace_var_kind<T: VariableSuperTrait + Hash + Eq, T2: VariableSuperTrait>(
    e: &ExpressionSyntaxTree<T>,
    env: &HashMap<T, T2>,
) -> Option<ExpressionSyntaxTree<T2>> {
    let res = match e {
        ExpressionSyntaxTree::Variable(x) => ExpressionSyntaxTree::variable(
            x.location.0,
            x.location.1,
            env.get(&x.value)?.to_owned(),
        ),

        ExpressionSyntaxTree::Number(x) => {
            ExpressionSyntaxTree::number(x.location.0, x.location.1, x.value)
        }
        ExpressionSyntaxTree::Fun(x) => {
            let vals = x.value.1.iter().map(move |t| replace_var_kind(&t, env));
            let fails = vals.clone().filter(Option::is_none).count();

            if fails == 0 {
                ExpressionSyntaxTree::fun(
                    x.location.0,
                    x.location.1,
                    x.value.0.clone(),
                    vals.map(Option::unwrap).map(|x| x.to_owned()).collect(),
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
    return Some(res);
}

//TODO replace_function

fn replace_expr<T: VariableSuperTrait>(
    source: &ExpressionSyntaxTree<T>,
    sum2find: &ExpressionSyntaxTree<T>,
    expr2paste: &ExpressionSyntaxTree<T>,
) -> Option<ExpressionSyntaxTree<T>> {
    if source.has_same_shape(sum2find) {
        return Some(expr2paste.to_owned());
    }
    match source {
        x @ ExpressionSyntaxTree::Variable(_) => Some(x.to_owned()),
        x @ExpressionSyntaxTree::Number(_) => Some(x.to_owned()),
        ExpressionSyntaxTree::Fun(_) => todo!(),
        ExpressionSyntaxTree::Sum(x) => Some(ExpressionSyntaxTree::add(
            x.location.0,
            x.location.1,
            replace_expr(&x.value.0, sum2find, expr2paste)?,
            replace_expr(&x.value.1, sum2find, expr2paste)?,
        )),
        ExpressionSyntaxTree::Product(x) => Some(ExpressionSyntaxTree::mul(
            x.location.0,
            x.location.1,
            replace_expr(&x.value.0, sum2find, expr2paste)?,
            replace_expr(&x.value.1, sum2find, expr2paste)?,
        )),
        ExpressionSyntaxTree::Exponent(x) => Some(ExpressionSyntaxTree::exp(
            x.location.0,
            x.location.1,
            replace_expr(&x.value.0, sum2find, expr2paste)?,
            replace_expr(&x.value.1, sum2find, expr2paste)?,
        )),
        ExpressionSyntaxTree::Subtraction(x) => Some(ExpressionSyntaxTree::sub(
            x.location.0,
            x.location.1,
            replace_expr(&x.value.0, sum2find, expr2paste)?,
            replace_expr(&x.value.1, sum2find, expr2paste)?,
        )),
        ExpressionSyntaxTree::Division(x) => Some(ExpressionSyntaxTree::div(
            x.location.0,
            x.location.1,
            replace_expr(&x.value.0, sum2find, expr2paste)?,
            replace_expr(&x.value.1, sum2find, expr2paste)?,
        )),
        ExpressionSyntaxTree::Negation(x) => Some(ExpressionSyntaxTree::neg(
            x.location.0,
            x.location.1,
            replace_expr(&x.value, sum2find, expr2paste)?,
        )),
    }
}

fn replace_var<T: VariableSuperTrait + Hash + Eq>(
    e: &ExpressionSyntaxTree<T>,
    env: &HashMap<T, ExpressionSyntaxTree<T>>,
) -> Option<ExpressionSyntaxTree<T>> {
    let res = match e {
        ExpressionSyntaxTree::Variable(x) => env.get(&x.value)?.to_owned(),

        ExpressionSyntaxTree::Number(x) => {
            ExpressionSyntaxTree::number(x.location.0, x.location.1, x.value)
        }
        ExpressionSyntaxTree::Fun(x) => {
            let vals = x.value.1.iter().map(move |t| replace_var(&t, env));
            let fails = vals.clone().filter(Option::is_none).count();

            if fails == 0 {
                ExpressionSyntaxTree::fun(
                    x.location.0,
                    x.location.1,
                    x.value.0.clone(),
                    vals.map(Option::unwrap).map(|x| x.to_owned()).collect(),
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
    return Some(res);
}




#[derive(Clone, Debug, PartialEq)]
struct ExpressionRange1dResult(/*TODO Box<[f64]>*/ Vec<f64>);

impl Display for ExpressionRange1dResult {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}
impl ExpressionRange1dResult {
    fn new(s: f64, e: f64, st: f64) -> Self {
        let g = (st+1.0)*1_000f64;
        let mut delta = (((e - s) * g) / (st+1.0)) as i64;
        if delta == 0 {
            delta = 1;
        }
        let start = (s * g) as i64;
        let end = (e * g) as i64;
        let ret = (start..end)
            .step_by(delta as usize)
            .map(|x| (x as f64) / g)
            .collect();
        Self(ret)
    }

    fn max(&self) -> f64 {
        self.clone().0.into_iter().reduce(f64::max).unwrap()
    }

    fn min(&self) -> f64 {
        self.clone().0.into_iter().reduce(f64::min).unwrap()
    }
}

impl From<f64> for ExpressionRange1dResult {
    fn from(value: f64) -> Self {
        ExpressionRange1dResult(vec![value])
    }
}

impl From<Vec<f64>> for ExpressionRange1dResult {
    fn from(value: Vec<f64>) -> Self {
        ExpressionRange1dResult(value)
    }
}

macro_rules! ops_definer {
    ($type:ident,$optype:ident,$opname:ident,$oprepr:tt) => {
        impl $optype for $type{
            type Output = Self;

            fn $opname(self, rhs: Self) -> Self::Output {
                match (self.0.len(),rhs.0.len(),self,rhs){
                    (_,1,y,x) => y.0.iter().map(|var| var $oprepr x.0.get(0).unwrap()).collect::<Vec<f64>>().into(),
                    (1,_,y,x) => x.0.iter().map(|var| y.0.get(0).unwrap() $oprepr var ).collect::<Vec<f64>>().into(),
                    (a,b,x,y) if a == b => y.0.iter().zip(x.0.iter()).map(|(right,left)| left $oprepr right  ).collect::<Vec<f64>>().into(),
                    (_,_,_,_) => panic!("lengths are not same ")
                }
            }
        }
    };
}

impl HasSameShape for ExpressionRange1dResult {
    fn has_same_shape(&self, other: &Self) -> bool {
        self == other
    }
}

ops_definer!(ExpressionRange1dResult,Mul,mul,*);
ops_definer!(ExpressionRange1dResult,Div,div,/);
ops_definer!(ExpressionRange1dResult,Add,add,+);
ops_definer!(ExpressionRange1dResult,Sub,sub,-);

impl Neg for ExpressionRange1dResult {
    type Output = Self;
    fn neg(self) -> Self::Output {
        self.0.iter().map(Neg::neg).collect::<Vec<f64>>().into()
    }
}

impl Pow<ExpressionRange1dResult> for ExpressionRange1dResult {
    type Output = Self;

    fn pow(self, rhs: Self) -> Self::Output {
        match (self.0.len(), rhs.0.len(), self, rhs) {
            (_, 1, x, y) => {
                x.0.iter()
                    .map(|var| var.powf(*y.0.get(0).unwrap()))
                    .collect::<Vec<f64>>()
                    .into()
            }
            (1, _, x, y) => {
                y.0.iter()
                    .map(|var| (*x.0.get(0).unwrap()).powf(*var))
                    .collect::<Vec<f64>>()
                    .into()
            }
            (a, b, x, y) if a == b => {
                x.0.iter()
                    .zip(y.0.iter())
                    .map(|(left, right)| left.powf(*right))
                    .collect::<Vec<f64>>()
                    .into()
            }
            (_, _, _, _) => panic!("lengths are not same "),
        }
    }
}



fn regis_prolog(x: usize, y: usize) -> String {
    format!("\x1bP0p\nS(A[0,0][{x},{y}])\nS(E)(C1)")
}

fn regis_print_grid(x: usize, y: usize, x_spacing: usize, y_spacing: usize) -> String {
    let mut ret = "".to_string();
    ret.extend("W(P1)\n".chars());

    let middle = (x + y) / 2;
    ret.extend(format!("P[{},0]\nV(B),[+0,+{y}],(E)\n", x / 2).chars());
    ret.extend(format!("P[0,{}]\nV(B),[+{x},+0],(E)\n", y / 2).chars());

    ret.extend(
        (0..x / 2)
            .step_by(x_spacing)
            .map(|s| format!("P[{},{}]\nT(S0)\n'{s}'", x / 2 + s, y / 2)),
    );
    ret.extend(
        (0..y / 2)
            .step_by(y_spacing)
            .map(|s| format!("P[{},{}]\nT(S0)\n'{s}'", x / 2, y / 2 - s)),
    );
    ret.extend("W(P3)\n".chars());

    ret.extend((0..x / 2).step_by(x_spacing).map(|s| {
        format!(
            "P[{},0]\nV(B),[+0,+{y}],(E)\nP[{},0]\nV(B),[+0,+{y}],(E)",
            x / 2 + s,
            x / 2 - s
        )
    }));
    ret.extend((0..y / 2).step_by(y_spacing).map(|s| {
        format!(
            "P[0,{}]\nV(B),[+{x},+0],(E)\nP[0,{}]\nV(B),[+{x},+0],(E)",
            s + y / 2,
            y / 2 - s
        )
    }));

    return ret;
}

fn regis_plot_graph_2(result: &ExpressionRange1dResult, x: usize, y: usize) -> String {
    let max = result.min().abs().max(result.max());

    let interp = |input,output_start,output_end,input_start,input_end| { return output_start + ((output_end - output_start) / (input_end - input_start)) * (input - input_start)};

    //assert!(x == result.0.len()); //FIX THIS TODO
    let mut res = "W(P1)\nC(S)".to_string();
    res.extend(
        (0..result.0.len() + 1)
            .zip(result.0.iter())
            .map(|(x_raw, y_raw)| format!("[{},{}],",  interp(x_raw as f64,0.0,x as f64,0.0,result.0.len() as f64),interp(*y_raw,0.0,y as f64,result.min(),result.max()) as u64))
    );
    res.extend(format!("(E)\n").chars());
    res
}

fn regis_plot_graph(result: &ExpressionRange1dResult, x: usize, y: usize) -> String {
    let max = result.min().abs().max(result.max());

    let interp = |input,output_start,output_end,input_start,input_end| { return output_start + ((output_end - output_start) / (input_end - input_start)) * (input - input_start)};

    //assert!(x == result.0.len()); //FIX THIS TODO
    let mut res = "W(P1)\n".to_string();
    res.extend(
        (0..result.0.len() + 1)
            .zip(result.0.iter())
            .map(|(x_raw, y_raw)| format!("P[{},{}]\nV[]\n",  x_raw,
                                          y as isize - interp(*y_raw,0.0,(y as f64),result.min(),result.max()) as isize))
    );
    //res.extend(format!("(E)\n").chars());
    res
}

fn regis_epilog() -> String {
    format!("\x1b\\")
}

fn regis(
    result: &ExpressionRange1dResult,
    x: usize,
    y: usize,
    x_spacing: usize,
    y_spacing: usize,
) -> String {
    let plog = regis_prolog(x, y);
    let graph = regis_plot_graph(result, x, y);
    let grid = regis_print_grid(x, y, x_spacing, y_spacing);
    let elog = regis_epilog();
    format!("{plog}\n{grid}{graph}\n{elog}")
}
#[test]
fn pof_range() {
    let init_vals = ExpressionRange1dResult::new(-1.0, 1.0, 20_000.0);
    println!(" len = {} \n init = {init_vals}",init_vals.0.len());
    assert!(true)
}

#[test]
fn pof_regis() {
    let state = State::new();
    let state2 = State::new();
    let (a, _, _) = "1/a ".chars().parse(state).unwrap();
    let (b, _, _) = "2* (a + 3 ) - a /2 ".chars().parse(state2).unwrap();
    //println!("{a}");
    let mut env = HashMap::new();
    let init_vals = ExpressionRange1dResult::new(-1.0, 1.0, 10.0);
    println!("init = {init_vals}");
    env.insert("a".to_string(), init_vals.clone());
    match eval(&a, &env) {
        Some(result) => {
            println!("{result}");
            let r = regis(&result, result.0.len(), 800, 50, 25);
            println!("{r}");
            assert!(true)
        }
        None => assert!(false),
    }

    assert!(true)
}

#[test]
fn pof() {
    let state = State::new();
    let state2 = State::new();
    let (a, _, _) = "a*a  ".chars().parse(state).unwrap();
    let (b, _, _) = "2* (a + 3 ) - a /2 ".chars().parse(state2).unwrap();
    println!("{a}");
    let mut env = HashMap::new();
    let init_vals = ExpressionRange1dResult::new(0.0, 1.0, 10.0);
    println!("init = {init_vals}");
    env.insert("a".to_string(), init_vals.clone());
    match eval(&a, &env) {
        Some(result) => {
            println!("{result}");

            assert!(true)
        }
        None => assert!(false),
    }

    assert!(true)
}



#[test]
fn hehe2() {
    let state = State::new();
    let state2 = State::new();
    let state3 = State::new();
    let (a, _, _) = "a + a ".chars().parse(state).unwrap();
    let (b, _, _) = "b*b  ".chars().parse(state2).unwrap();
    let (c, _, _) = " a + a + 2 ^ ( a + a + (a + a)) ".chars().parse(state3).unwrap();
    
    let d = replace_expr(&c,&a, &b).unwrap();
    println!("{}",d);

    assert!(true)
}
/*struct FRange1D {
    start: u64,
    end: u64,
    step: u64,
    gran: f64,
}

impl FRange1D {
    fn new(start: f64, end: f64, step: f64) -> Self {
        let g = 1_000f64;
        let delta = (((start - end) / step) * g) as u64;
        Self {
            start: (start * g) as u64,
            end: (end * g) as u64,
            step: delta,
            gran: g,
        }
    }
}

impl Iterator for FRange1D {
    type Item = f64;

    fn next(&mut self) -> Option<Self::Item> {
        if (self.start + self.step) < self.end {
            let ret = self.start as f64 / self.gran;
            self.start += self.step;
            return Option::Some(ret);
        } else {
            return None;
        }
    }
}
*/

/*

#[test]
fn pof() {
let state = State::new();
let (a, _, _) = "1-a-b-c-d".chars().parse(state).unwrap();
println!("{a}");

let state = State::new();
let (a, _, _) = "1^a^b^c^1".chars().parse(state).unwrap();
println!("{a}");

assert!(true)
}


#[test]
fn pof2() {
    let state = State::new();
    let (a, _, _) = "1.23 - a-b-c-d+a+b".chars().parse(state).unwrap();

    let mut hset = HashSet::new();
    let var_cnr = uniq_var_count(a.clone(), &mut hset);
    println!("{a}   has {var_cnr} variables");
    let state = State::new();
    let (a, _, _) = "1^a^b^c^1".chars().parse(state).unwrap();
    let mut hset = HashSet::new();
    let var_cnr = uniq_var_count(a.clone(), &mut hset);
    println!("{a}   has {var_cnr} variables");

    assert!(true)
}


#[test]
fn pof3() {
    let state = State::new();
    let (a, _, _) = "1.23 - a-b-c-d+a+b".chars().parse(state).unwrap();

    let mut hset = HashSet::new();
    let mut hmap = HashMap::new();
    hmap.insert("a".to_string(), 1000.0);
    let b = replace_var(a, &hmap);
    let var_cnr = uniq_var_count(b.clone(), &mut hset);
    println!("{b}   has {var_cnr} variables");
    let state = State::new();
    let (a, _, _) = "1^a^b^c^1".chars().parse(state).unwrap();
    let mut hset = HashSet::new();
    let var_cnr = uniq_var_count(a.clone(), &mut hset);
    println!("{a}   has {var_cnr} variables");

    assert!(true)
}
*/

fn main() {
    /*let stdin = stdin();
    for line in stdin.lines() {

        let state = State::new();
        let (a, _, _) = line.unwrap().chars().parse(state).unwrap();
        //println!("{a}");
        let mut env = HashMap::new();
        let init_vals = ExpressionRange1dResult::new(-1.0, 1.0, 1000.0);
        //println!("init = {init_vals}");
        env.insert("a".to_string(), init_vals.clone());
        match eval(&a, &env) {
            Some(result) => {
                //println!("{result}");
                let r = regis(&result, result.0.len(), 800, 50, 25);
                println!("{r}");
                
            }
            None => println!("wrong hehee"),
        }

        
    }*/
    let mut args = env::args();
    args.next();
    let state = State::new();
    let line = args.collect::<String>();
    match line.clone().chars().parse(state){
        Ok((tree,_,_)) => {
            println!(":{line} succeed  :{}",tree.clone());
            println!(":{line} succeed  :{:?}",tree.clone());
            let mut env = HashMap::new();
            let init_vals = ExpressionRange1dResult::new(-1.0, 1.0, 1000.0);
            //println!("init = {init_vals}");
            env.insert("a".to_string(), init_vals.clone());
            match eval(&tree, &env) {
                Some(result) => {
                    //println!("{result}");
                    let r = regis(&result, result.0.len(), 800, 50, 25);
                    println!("{r}");
                    
                }
                None => println!("wrong hehee"),
            }
        },
        Err(err) => println!(":{line} failed  :{:?}",err),
    }

}
*/

trait Parsable<'a, T, I: Display>
where
    T: Parse<'a, Chars<'a>, State, ExpressionSyntaxTree<String>, ParseErrors>,
{
    fn parse(self, s: State) -> expression_parser::ExprParseResult<'a>;
}

trait Parsable2<'a, T :  Parse<'a, Self, State, O, ParseErrors>,O>
where

    Self:Sized + Iterator + Clone + 'a,
<Self as Iterator>::Item : Eq, O: 'a
    
{
    fn get_parser() -> T;
    fn parse(self, s: State) -> Result<(O, State, Self), ParseErrors>{
        Self::get_parser().parse(self, s)
    }
}




impl<'a> Parsable2<'a,EExpression,  ExpressionSyntaxTree<String>> for Chars<'a> {
    fn get_parser() -> EExpression {
        EExpression
    }
}

impl<'a> Parsable2<'a,EDefinition,  Definition<String>> for Chars<'a> {
    fn get_parser() -> EDefinition {
        EDefinition
    }
}
#[test]
fn hmm(){
    let r1 : DefinitionParseResult = "a = 4;hello = 2; h(y)=y;f(x) = x*x; f(x,y) = 5 ;g(x,y) = 6".chars().parse(State::new());
    println!("{}",r1.unwrap().0);
    assert!(true);
}

fn main() {
    
    /*let stdin = stdin();
    for line in stdin.lines() {

        let state = State::new();
        let (a, _, _) = line.unwrap().chars().parse(state).unwrap();
        //println!("{a}");
        let mut env = HashMap::new();
        let init_vals = ExpressionRange1dResult::new(-1.0, 1.0, 1000.0);
        //println!("init = {init_vals}");
        env.insert("a".to_string(), init_vals.clone());
        match eval(&a, &env) {
            Some(result) => {
                //println!("{result}");
                let r = regis(&result, result.0.len(), 800, 50, 25);
                println!("{r}");
                
            }
            None => println!("wrong hehee"),
        }

        
    }*/
    let mut args = env::args();
    args.next();
    let state = State::new();
    let line = args.collect::<String>();
    match EExpression.parse(line.chars(), state){
        Ok((tree,_,_)) => {
            println!(":{line} succeed  :{}",tree.clone());
            println!(":{line} succeed  :{:?}",tree.clone());
        }
            /*let mut env = HashMap::new();
            let init_vals = ExpressionRange1dResult::new(-1.0, 1.0, 1000.0);
            //println!("init = {init_vals}");
            env.insert("a".to_string(), init_vals.clone());
            match eval(&tree, &env) {
                Some(result) => {
                    //println!("{result}");
                    let r = regis(&result, result.0.len(), 800, 50, 25);
                    println!("{r}");
                    
                }
                None => println!("wrong hehee"),
            }
        },*/
        Err(err) => println!(":{line} failed  :{:?}",err),
    }
}
