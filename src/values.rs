use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io::{self, BufRead};
use std::ops::{Add, Div, Mul, Neg, Sub};
use std::path::Path;


use crate::eval::{EvaluationError, Pow};
use crate::expression::{HasSameShape, VariableSuperTrait};
use crate::parser_common::Localization;


pub trait PrimitiveUnary{
    
}

pub trait PrimitiveBinary{}

pub trait PrimitiveTernary{}

impl <T: Fn(f64) -> f64 > PrimitiveUnary for T {
    
}



impl <T: Fn(f64,f64) -> f64 > PrimitiveBinary for T {
    
}


impl <T: Fn(f64,f64,f64) -> f64 > PrimitiveTernary for T {
    
}




pub fn make_primitive_unary<'a,F:Fn(f64) -> f64 + 'a>( f:&'a F) -> &'a dyn PrimitiveUnary{
    return f;
}
 //TODO make Values Iterable so you can generalize this concept
pub fn make_primitive_unary_unary<'a,T:Fn(f64)->f64  >( f:&'a T) -> impl Fn(ExpressionRange1dResult) -> ExpressionRange1dResult + 'a {
     | x | {
         x.0.into_iter().map(
         |a| f(a)).collect::<Vec<f64>>().into()
    }
}




pub fn make_primitive_binary<'a,F>( f:&'a F) -> &'a dyn PrimitiveBinary where F: Fn(f64,f64) -> f64 + 'a {
    return f;
}

pub fn make_primitive_binary_binary<'a,T:Fn(f64,f64)->f64  >( f:&'a T) -> impl Fn(ExpressionRange1dResult,ExpressionRange1dResult) -> ExpressionRange1dResult + 'a {
    | x,y | {
        x.0.into_iter().zip(y.0.into_iter()).map(
            |(a,b)| f(a,b)).collect::<Vec<f64>>().into()
    }
}


pub fn make_primitive_ternary<'a,F>( f:&'a F) -> &'a dyn PrimitiveTernary where F: Fn(f64,f64,f64) -> f64 + 'a {
    return f;
}

pub fn make_primitive_ternary_ternary<'a,T:Fn(f64,f64,f64)->f64  >( f:&'a T) -> impl Fn(ExpressionRange1dResult,ExpressionRange1dResult,ExpressionRange1dResult) -> ExpressionRange1dResult + 'a {
    | x,y,z | {
        x.0.into_iter().zip(y.0.into_iter()).zip(z.0.into_iter()).map(
            |((a,b),c)| f(a,b,c)).collect::<Vec<f64>>().into()
    }
}



pub trait Value<T> : Add + Mul + Neg + Div + Sub + Pow<T> + From<f64> {}

impl <T,A : Add + Mul + Neg + Div + Sub + Pow<T> + From<f64>> Value<T> for A {
    
}


#[derive(Clone, Debug, PartialEq)]
pub struct ExpressionRange1dResult(/*TODO Box<[f64]>*/ Vec<f64>);

impl Display for ExpressionRange1dResult {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Ok(self.0.iter().for_each(|x| {
            write!(f, "{x} ");
        }))
    }
}

pub fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}
impl ExpressionRange1dResult {
    pub fn create_with_step(s: f64, e: f64, st: f64) -> Result<Self, EvaluationError> {
        let g = (st + 1.0) * 1_000f64;
        let mut delta = (((e - s) * g) / (st + 1.0)) as i64;
        if delta == 0 {
            delta = 1;
        }
        let start = (s * g) as i64;
        let end = (e * g) as i64;
        let ret = (start..end)
            .step_by(delta as usize)
            .map(|x| (x as f64) / g)
            .collect();
        Ok(Self(ret))
    }

    pub fn create_from_file(path: String) -> Result<Self, impl FnOnce(Localization,Localization) -> EvaluationError> {
        let er = Err( |a , b| EvaluationError::GenericWithString(a, b, "Couldnt Read Line".to_string()));
        if let Ok(lines) = read_lines(path) {
            let mut rey = vec![];
            for line in lines {
                if let Ok(ip) = line {
                    let ret : Vec<f64> = ip
                        .split_ascii_whitespace()
                        .into_iter()
                        .map(str::parse::<f64>)
                        .filter(Result::is_ok)
                        .map(Result::unwrap)
                        .collect();
                    rey.extend(ret);
                } else {
                    return er;
                }
            }
            return Ok(Self(rey));
        } else {
           return  er;
        }
    }

    pub fn create_from_file_col(path: String, col:usize) -> Result<Self, impl FnOnce(Localization,Localization) -> EvaluationError>  {
        let er = Err( |a , b| EvaluationError::GenericWithString(a, b, "Couldnt Read Line".to_string()));
        if let Ok(lines) = read_lines(path) {
            let mut rey = vec![];
            for line in lines {
                if let Ok(ip) = line {
                    let ret  : Vec<f64>= ip
                        .split_ascii_whitespace().filter_map(|x| x.parse::<f64>().ok()).collect();
                    rey.push(*ret.get(col).unwrap());
                } else {
                    return er;
                }
            }
            return Ok(Self(rey));
        } else {
            return er;
        }
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
