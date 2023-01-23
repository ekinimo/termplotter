use crate::parser_common::{Node, Localization};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ERangeNumeric;
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ERangeNumericStep;
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ERangeFile;
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ERangeFileCol;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ERange;

#[derive(Clone, Debug, PartialEq)]
pub enum Range {
    Numeric(Node<ERangeNumeric, (String, f64, f64)>),
    NumericStep(Node<ERangeNumericStep, (String, f64, f64, f64)>),
    FileBare(Node<ERangeFile, (String, String)>),
    FileCol(Node<ERangeFileCol, (String, String, usize)>),
}

impl Range {
    pub fn numeric(
        starts: Localization,
        end: Localization,
        var: String,
        starts_at: f64,
        ends_at: f64,
    ) -> Self {
        Self::Numeric(Node::new(starts, end, (var, starts_at, ends_at)))
    }
    pub fn numeric_step(
        starts: Localization,
        end: Localization,
        var: String,
        starts_at: f64,
        ends_at: f64,
        step: f64,
    ) -> Self {
        Self::NumericStep(Node::new(starts, end, (var, starts_at, ends_at, step)))
    }
    pub fn file(starts: Localization, end: Localization, var: String, name: String) -> Self {
        Self::FileBare(Node::new(starts, end, (var, name)))
    }
    pub fn file_col(
        starts: Localization,
        end: Localization,
        var: String,
        name: String,
        col: usize,
    ) -> Self {
        Self::FileCol(Node::new(starts, end, (var, name, col)))
    }
}
