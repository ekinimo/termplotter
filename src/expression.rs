use std::{
    fmt::Debug,
    fmt::{Display, Formatter},
    usize,
};

use crate::{ parser_common::{Node, Localization}};

pub trait VariableSuperTrait: Display + Clone + PartialEq + Debug + HasSameShape {}
pub trait HasSameShape {
    fn has_same_shape(&self, other: &Self) -> bool;
}

pub trait LangElement {}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EVar;
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ENum;
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EFun;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ESum;
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EMul;
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ESub;
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EExp;
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EDiv;
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ENeg;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EExpression;


#[derive(Clone, Debug, PartialEq)]
pub enum ExpressionSyntaxTree<T: VariableSuperTrait> {
    Variable(Node<EVar, T>),
    Number(Node<ENum, f64>),
    Fun(Node<EFun, (String, Vec<ExpressionSyntaxTree<T>>)>),
    Sum(Box<Node<ESum, (ExpressionSyntaxTree<T>, ExpressionSyntaxTree<T>)>>),
    Product(Box<Node<EMul, (ExpressionSyntaxTree<T>, ExpressionSyntaxTree<T>)>>),

    Exponent(Box<Node<EExp, (ExpressionSyntaxTree<T>, ExpressionSyntaxTree<T>)>>),

    Subtraction(Box<Node<ESub, (ExpressionSyntaxTree<T>, ExpressionSyntaxTree<T>)>>),
    Division(Box<Node<EDiv, (ExpressionSyntaxTree<T>, ExpressionSyntaxTree<T>)>>),
    Negation(Box<Node<ENeg, ExpressionSyntaxTree<T>>>),
}




impl<T: VariableSuperTrait> ExpressionSyntaxTree<T> {
    pub fn number(starts: Localization, ends: Localization, num: f64) -> Self {
        ExpressionSyntaxTree::Number(Node::new(starts, ends, num))
    }

    pub fn variable(starts: Localization, ends: Localization, name: T) -> Self {
        ExpressionSyntaxTree::Variable(Node::new(starts, ends, name))
    }

    pub fn add(starts: Localization, ends: Localization, left: Self, right: Self) -> Self {
        ExpressionSyntaxTree::Sum(Box::new(Node::new(starts, ends, (left, right))))
    }

    pub fn mul(starts: Localization, ends: Localization, left: Self, right: Self) -> Self {
        ExpressionSyntaxTree::Product(Box::new(Node::new(starts, ends, (left, right))))
    }

    pub fn sub(starts: Localization, ends: Localization, left: Self, right: Self) -> Self {
        ExpressionSyntaxTree::Subtraction(Box::new(Node::new(starts, ends, (left, right))))
    }

    pub fn div(starts: Localization, ends: Localization, left: Self, right: Self) -> Self {
        ExpressionSyntaxTree::Division(Box::new(Node::new(starts, ends, (left, right))))
    }

    pub fn exp(starts: Localization, ends: Localization, left: Self, right: Self) -> Self {
        ExpressionSyntaxTree::Exponent(Box::new(Node::new(starts, ends, (left, right))))
    }

    pub fn neg(starts: Localization, ends: Localization, value: Self) -> Self {
        ExpressionSyntaxTree::Negation(Box::new(Node::new(starts, ends, value)))
    }

    pub fn fun(starts: Localization, ends: Localization, name: String, vec: Vec<Self>) -> Self {
        ExpressionSyntaxTree::Fun(Node::new(starts, ends, (name, vec)))
    }
}


impl<T: Debug + Clone + Display + PartialEq + HasSameShape> VariableSuperTrait for T {}

impl LangElement for EVar {}
impl LangElement for ENum {}
impl LangElement for EFun {}
impl LangElement for EExp {}
impl LangElement for EMul {}
impl LangElement for EDiv {}
impl LangElement for ENeg {}

impl HasSameShape for String {
    fn has_same_shape(&self, other: &Self) -> bool {
        self == other
    }
}

impl HasSameShape for f64 {
    fn has_same_shape(&self, other: &Self) -> bool {
        self == other
    }
}

impl HasSameShape for usize {
    fn has_same_shape(&self, other: &Self) -> bool {
        self == other
    }
}

impl<T: HasSameShape> HasSameShape for Vec<T> {
    fn has_same_shape(&self, other: &Self) -> bool {
        self.iter()
            .zip(other.iter())
            .all(|(x, y)| x.has_same_shape(y))
    }
}

impl HasSameShape for () {
    fn has_same_shape(&self, other: &Self) -> bool {
        self == other
    }
}

impl<T1: HasSameShape, T2: HasSameShape> HasSameShape for (T1, T2) {
    fn has_same_shape(&self, other: &Self) -> bool {
        let (a, b) = self;
        let (a1, b1) = other;
        a.has_same_shape(a1) && b.has_same_shape(b1)
    }
}

impl<T1: HasSameShape, T2: HasSameShape, T3: HasSameShape> HasSameShape for (T1, T2, T3) {
    fn has_same_shape(&self, other: &Self) -> bool {
        let (a, b, c) = self;
        let (a1, b1, c1) = other;
        a.has_same_shape(a1) && b.has_same_shape(b1) && c.has_same_shape(c1)
    }
}

impl<T1: HasSameShape, T2: HasSameShape, T3: HasSameShape, T4: HasSameShape> HasSameShape
    for (T1, T2, T3, T4)
{
    fn has_same_shape(&self, other: &Self) -> bool {
        let (a, b, c, d) = self;
        let (a1, b1, c1, d1) = other;
        a.has_same_shape(a1) && b.has_same_shape(b1) && c.has_same_shape(c1) && d.has_same_shape(d1)
    }
}

impl<T: VariableSuperTrait> HasSameShape for ExpressionSyntaxTree<T> {
    fn has_same_shape(&self, other: &Self) -> bool {
        match (self, other) {
            (ExpressionSyntaxTree::Variable(x), ExpressionSyntaxTree::Variable(y)) => {
                x.has_same_shape(y)
            }
            (ExpressionSyntaxTree::Number(x), ExpressionSyntaxTree::Number(y)) => {
                x.has_same_shape(y)
            }
            (ExpressionSyntaxTree::Fun(x), ExpressionSyntaxTree::Fun(y)) => x.has_same_shape(y),
            (ExpressionSyntaxTree::Sum(x), ExpressionSyntaxTree::Sum(y)) => x.has_same_shape(y),
            (ExpressionSyntaxTree::Product(x), ExpressionSyntaxTree::Product(y)) => {
                x.has_same_shape(y)
            }
            (ExpressionSyntaxTree::Exponent(x), ExpressionSyntaxTree::Exponent(y)) => {
                x.has_same_shape(y)
            }
            (ExpressionSyntaxTree::Subtraction(x), ExpressionSyntaxTree::Subtraction(y)) => {
                x.has_same_shape(y)
            }
            (ExpressionSyntaxTree::Division(x), ExpressionSyntaxTree::Division(y)) => {
                x.has_same_shape(y)
            }
            (ExpressionSyntaxTree::Negation(x), ExpressionSyntaxTree::Negation(y)) => {
                x.has_same_shape(y)
            }
            (_, _) => false,
        }
    }
}

impl<T: VariableSuperTrait> Display for ExpressionSyntaxTree<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ExpressionSyntaxTree::Variable(Node {
                value,
                location: _,
                phantom: _,
            }) => write!(f, "{value}"),

            ExpressionSyntaxTree::Fun(Node {
                value: (name, value),
                location: _,
                phantom: _,
            }) => write!(
                f,
                "{name}({})",
                value
                    .iter()
                    .map(|x| format!("{x}"))
                    .collect::<Vec<String>>()
                    .join(",")
            ),

            ExpressionSyntaxTree::Number(Node {
                value,
                location: _,
                phantom: _,
            }) => write!(f, "{value}"),
            ExpressionSyntaxTree::Sum(box Node {
                value: (left, right),
                location: _,
                phantom: _,
            }) => write!(f, "({left} + {right})"),
            ExpressionSyntaxTree::Product(box Node {
                value: (left, right),
                location: _,
                phantom: _,
            }) => write!(f, "({left} * {right})"),
            ExpressionSyntaxTree::Exponent(box Node {
                value: (left, right),
                location: _,
                phantom: _,
            }) => write!(f, "({left} ^ {right})"),
            ExpressionSyntaxTree::Subtraction(box Node {
                value: (left, right),
                location: _,
                phantom: _,
            }) => write!(f, "({left} - {right})"),
            ExpressionSyntaxTree::Division(box Node {
                value: (left, right),
                location: _,
                phantom: _,
            }) => write!(f, "({left} / {right})"),
            ExpressionSyntaxTree::Negation(box Node {
                value,
                location: _,
                phantom: _,
            }) => write!(f, "-{value}"),
        }
    }
}
