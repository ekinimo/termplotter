use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io::{self, BufRead};
use std::ops::{Add, Div, Mul, Neg, Sub};
use std::path::Path;

use crate::eval::{EvaluationError, Pow};
use crate::expression::{HasSameShape, VariableSuperTrait};
use crate::parser_common::Localization;

pub trait PrimitiveUnary<T> {
    fn apply(&self, arg: T) -> Result<T, String>;
}

pub trait PrimitiveBinary<T> {
    fn apply(&self, arg1: T, arg2: T) -> Result<T, String>;
}

pub trait PrimitiveTernary<T> {
    fn apply(&self, arg1: T, arg2: T, arg3: T) -> Result<T, String>;
}

pub struct UnaryFunction<F: Fn(f64) -> f64> {
    func: F,
}

pub struct BinaryFunction<F: Fn(f64, f64) -> f64> {
    func: F,
}

pub struct TernaryFunction<F: Fn(f64, f64, f64) -> f64> {
    func: F,
}

impl<F: Fn(f64) -> f64> UnaryFunction<F> {
    pub fn new(func: F) -> Self {
        Self { func }
    }
}

impl<F: Fn(f64, f64) -> f64> BinaryFunction<F> {
    pub fn new(func: F) -> Self {
        Self { func }
    }
}

impl<F: Fn(f64, f64, f64) -> f64> TernaryFunction<F> {
    pub fn new(func: F) -> Self {
        Self { func }
    }
}

impl<F: Fn(f64) -> f64> PrimitiveUnary<ExpressionRange1dResult> for UnaryFunction<F> {
    fn apply(&self, arg: ExpressionRange1dResult) -> Result<ExpressionRange1dResult, String> {
        let result = arg.0.into_iter().map(&self.func).collect::<Vec<f64>>();
        Ok(ExpressionRange1dResult::from(result))
    }
}

impl<F: Fn(f64, f64) -> f64> PrimitiveBinary<ExpressionRange1dResult> for BinaryFunction<F> {
    fn apply(
        &self,
        arg1: ExpressionRange1dResult,
        arg2: ExpressionRange1dResult,
    ) -> Result<ExpressionRange1dResult, String> {
        let result = match (arg1.0.len(), arg2.0.len()) {
            (1, _) => arg2
                .0
                .into_iter()
                .map(|val| (self.func)(arg1.0[0], val))
                .collect::<Vec<f64>>(),
            (_, 1) => arg1
                .0
                .into_iter()
                .map(|val| (self.func)(val, arg2.0[0]))
                .collect::<Vec<f64>>(),
            (a, b) if a == b => arg1
                .0
                .into_iter()
                .zip(arg2.0.into_iter())
                .map(|(a, b)| (self.func)(a, b))
                .collect::<Vec<f64>>(),
            _ => return Err("Mismatched array lengths in binary function".into()),
        };
        Ok(ExpressionRange1dResult::from(result))
    }
}

impl<F: Fn(f64, f64, f64) -> f64> PrimitiveTernary<ExpressionRange1dResult> for TernaryFunction<F> {
    fn apply(
        &self,
        arg1: ExpressionRange1dResult,
        arg2: ExpressionRange1dResult,
        arg3: ExpressionRange1dResult,
    ) -> Result<ExpressionRange1dResult, String> {
        let result = match (arg1.0.len(), arg2.0.len(), arg3.0.len()) {
            (_, 1, 1) => arg1
                .0
                .into_iter()
                .map(|val| (self.func)(val, arg2.0[0], arg3.0[0]))
                .collect::<Vec<f64>>(),
            (1, _, 1) => arg2
                .0
                .into_iter()
                .map(|val| (self.func)(arg1.0[0], val, arg3.0[0]))
                .collect::<Vec<f64>>(),
            (1, 1, _) => arg3
                .0
                .into_iter()
                .map(|val| (self.func)(arg1.0[0], arg2.0[0], val))
                .collect::<Vec<f64>>(),
            (a, b, c) if a == b && b == c => arg1
                .0
                .into_iter()
                .zip(arg2.0.into_iter())
                .zip(arg3.0.into_iter())
                .map(|((val, min), max)| (self.func)(val, min, max))
                .collect::<Vec<f64>>(),
            _ => return Err("Mismatched array lengths in ternary function".into()),
        };
        Ok(ExpressionRange1dResult::from(result))
    }
}

pub fn make_primitive_unary<T, F: Fn(f64) -> f64 + 'static>(f: F) -> Box<dyn PrimitiveUnary<T> + 'static>
where UnaryFunction<F>: PrimitiveUnary<T>
{
    Box::new(UnaryFunction::new(f))
}

pub fn make_primitive_binary<T, F: Fn(f64, f64) -> f64 + 'static>(
    f: F,
) -> Box<dyn PrimitiveBinary<T> + 'static>
    where BinaryFunction<F>: PrimitiveBinary<T>

{
    Box::new(BinaryFunction::new(f))
}

pub fn make_primitive_ternary<T, F: Fn(f64, f64, f64) -> f64 + 'static>(
    f: F,
) -> Box<dyn PrimitiveTernary<T>+ 'static> where TernaryFunction<F>: PrimitiveTernary<T> {
    Box::new(TernaryFunction::new(f))
}

pub trait Value<T>: Add + Mul + Neg + Div + Sub + Pow<T> + From<f64> {}

impl<T, A: Add + Mul + Neg + Div + Sub + Pow<T> + From<f64>> Value<T> for A {}

#[derive(Clone, Debug, PartialEq)]
pub struct ExpressionRange1dResult(/*TODO Box<[f64]>*/ pub Vec<f64>);

#[derive(Clone, Debug, PartialEq)]
pub struct ExpressionRange2d(pub Vec<f64>,pub usize,pub usize);

#[derive(Clone, Debug, PartialEq)]
pub struct Expression3dResult {
    pub data: Vec<Vec<f64>>, // data[y_index][x_index] = z_value
    pub x_values: Vec<f64>,
    pub y_values: Vec<f64>,
}

impl Expression3dResult {
    pub fn new(data: Vec<Vec<f64>>, x_values: Vec<f64>, y_values: Vec<f64>) -> Self {
        Self { data, x_values, y_values }
    }
    
    pub fn x_len(&self) -> usize {
        self.x_values.len()
    }
    
    pub fn y_len(&self) -> usize {
        self.y_values.len()
    }
    
    pub fn is_empty(&self) -> bool {
        self.data.is_empty() || self.x_values.is_empty() || self.y_values.is_empty()
    }
    
    pub fn get_z(&self, x_index: usize, y_index: usize) -> Option<f64> {
        self.data.get(y_index)?.get(x_index).copied()
    }
    
    pub fn z_min(&self) -> f64 {
        self.data.iter()
            .flatten()
            .fold(f64::INFINITY, |a, &b| a.min(b))
    }
    
    pub fn z_max(&self) -> f64 {
        self.data.iter()
            .flatten()
            .fold(f64::NEG_INFINITY, |a, &b| a.max(b))
    }
    
    pub fn x_min(&self) -> f64 {
        self.x_values.iter().fold(f64::INFINITY, |a, &b| a.min(b))
    }
    
    pub fn x_max(&self) -> f64 {
        self.x_values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b))
    }
    
    pub fn y_min(&self) -> f64 {
        self.y_values.iter().fold(f64::INFINITY, |a, &b| a.min(b))
    }
    
    pub fn y_max(&self) -> f64 {
        self.y_values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b))
    }
}

impl Display for ExpressionRange1dResult {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Ok(self.0.iter().for_each(|x| {
            _ = write!(f, "{x} ");
        }))
    }
}
impl Display for ExpressionRange2d {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Ok(self.0.iter().enumerate().for_each(|(i,x)| {

            _ = write!(f, "{x} ");
            if i == self.1{
                _ = write!(f, "\n");
            }
        }))
    }
}



impl Display for Expression3dResult {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "3D Surface[{}x{} points, X:[{:.2}, {:.2}], Y:[{:.2}, {:.2}], Z:[{:.2}, {:.2}]]", 
               self.x_len(), self.y_len(), 
               self.x_min(), self.x_max(), 
               self.y_min(), self.y_max(), 
               self.z_min(), self.z_max())
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
        let ret = (start..=end)
            .step_by(delta as usize)
            .map(|x| (x as f64) / g)
            .collect();
        Ok(Self(ret))
    }

    pub fn create_from_file(
        path: String,
    ) -> Result<Self, impl FnOnce(Localization, Localization) -> EvaluationError> {
        let er =
            Err(|a, b| EvaluationError::GenericWithString(a, b, "Couldnt Read Line".to_string()));
        if let Ok(lines) = read_lines(path) {
            let mut rey = vec![];
            for line in lines {
                if let Ok(ip) = line {
                    let ret: Vec<f64> = ip
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
            return er;
        }
    }

    pub fn create_from_file_col(
        path: String,
        col: usize,
    ) -> Result<Self, impl FnOnce(Localization, Localization) -> EvaluationError> {
        let er =
            Err(|a, b| EvaluationError::GenericWithString(a, b, "Couldnt Read Line".to_string()));
        if let Ok(lines) = read_lines(path) {
            let mut rey = vec![];
            for line in lines {
                if let Ok(ip) = line {
                    let ret: Vec<f64> = ip
                        .split_ascii_whitespace()
                        .filter_map(|x| x.parse::<f64>().ok())
                        .collect();
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

    pub fn max(&self) -> f64 {
        self.clone().0.into_iter().reduce(f64::max).unwrap()
    }

    pub fn min(&self) -> f64 {
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

impl HasSameShape for Expression3dResult {
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


//TODO
//ops_definer!(ExpressionRange2d,Mul,mul,*);
//ops_definer!(ExpressionRange2d,Div,div,/);
//ops_definer!(ExpressionRange2d,Add,add,+);
//ops_definer!(ExpressionRange2d,Sub,sub,-);
