use std::{collections::HashSet, marker::PhantomData};

use crate::{
    expression::HasSameShape,
    parser_common::{Localization, Node},
};
use std::hash::Hash;

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct EDisplayRegis;
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct EDisplaySixel;
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct EDisplayAnsi;
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct EDisplayAscii;
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct EDisplay;

#[derive(Clone, Debug, PartialEq, Hash, Eq)]
pub enum DisplayOption {
    Regis(Node<EDisplayRegis, ()>),
    Sixel(Node<EDisplaySixel, ()>),
    Ansi(Node<EDisplayAnsi, ()>),
    Ascii(Node<EDisplayAscii, ()>),
}

impl DisplayOption {
    pub fn regis(starts: Localization, end: Localization) -> Self {
        Self::Regis(Node::new(starts, end, ()))
    }
    pub fn sixel(starts: Localization, end: Localization) -> Self {
        Self::Sixel(Node::new(starts, end, ()))
    }
    pub fn ascii(starts: Localization, end: Localization) -> Self {
        Self::Ascii(Node::new(starts, end, ()))
    }
    pub fn ansi(starts: Localization, end: Localization) -> Self {
        Self::Ansi(Node::new(starts, end, ()))
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct EOutputPPM;
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct EOutputSVG;
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct EOutputLaTeX;
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct EOutputSixel;
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct EOutputRegis;
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct EOutputCSV;

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct EOutput;

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Geometry<T> {
    pub width: usize,
    pub height: usize,
    phantom: PhantomData<T>,
}

impl<Dummy: Hash, T: Hash + HasSameShape> Hash for Node<Dummy, T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.value.hash(state);
        self.location.hash(state);
        self.phantom.hash(state);
    }
}

impl<T> Geometry<T> {
    fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            phantom: Default::default(),
        }
    }
}
impl<T> HasSameShape for Geometry<T> {
    fn has_same_shape(&self, other: &Self) -> bool {
        self.width == other.width && self.height == other.height
    }
}

impl Default for Geometry<EOutputPPM> {
    fn default() -> Self {
        Self {
            width: 800,
            height: 800,
            phantom: Default::default(),
        }
    }
}

impl Default for Geometry<EOutputSVG> {
    fn default() -> Self {
        Self {
            width: 800,
            height: 800,
            phantom: Default::default(),
        }
    }
}

impl Default for Geometry<EOutputLaTeX> {
    fn default() -> Self {
        Self {
            width: 800,
            height: 800,
            phantom: Default::default(),
        }
    }
}

impl Default for Geometry<EOutputSixel> {
    fn default() -> Self {
        Self {
            width: 800,
            height: 800,
            phantom: Default::default(),
        }
    }
}

impl Default for Geometry<EOutputRegis> {
    fn default() -> Self {
        Self {
            width: 800,
            height: 800,
            phantom: Default::default(),
        }
    }
}

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub enum OutputOptions {
    Ppm(Node<EOutputPPM, (String, Geometry<EOutputPPM>)>),
    Svg(Node<EOutputSVG, (String, Geometry<EOutputSVG>)>),
    LaTeX(Node<EOutputLaTeX, (String, Geometry<EOutputLaTeX>)>),
    Sixel(Node<EOutputSixel, (String, Geometry<EOutputSixel>)>),
    Regis(Node<EOutputRegis, (String, Geometry<EOutputRegis>)>),
    Csv(Node<EOutputCSV, String>),
}

impl OutputOptions {
    pub fn ppm(start: Localization, end: Localization, var: String) -> Self {
        Self::Ppm(Node::new(start, end, (var, Geometry::default())))
    }
    pub fn ppm_geom(
        start: Localization,
        end: Localization,
        var: String,
        width: usize,
        height: usize,
    ) -> Self {
        Self::Ppm(Node::new(start, end, (var, Geometry::new(width, height))))
    }
    pub fn svg(start: Localization, end: Localization, var: String) -> Self {
        Self::Svg(Node::new(start, end, (var, Geometry::default())))
    }
    pub fn svg_geom(
        start: Localization,
        end: Localization,
        var: String,
        width: usize,
        height: usize,
    ) -> Self {
        Self::Svg(Node::new(start, end, (var, Geometry::new(width, height))))
    }

    pub fn latex(start: Localization, end: Localization, var: String) -> Self {
        Self::LaTeX(Node::new(start, end, (var, Geometry::default())))
    }
    pub fn latex_geom(
        start: Localization,
        end: Localization,
        var: String,
        width: usize,
        height: usize,
    ) -> Self {
        Self::LaTeX(Node::new(start, end, (var, Geometry::new(width, height))))
    }

    pub fn sixel(start: Localization, end: Localization, var: String) -> Self {
        Self::Sixel(Node::new(start, end, (var, Geometry::default())))
    }

    pub fn sixel_geom(
        start: Localization,
        end: Localization,
        var: String,
        width: usize,
        height: usize,
    ) -> Self {
        Self::Sixel(Node::new(start, end, (var, Geometry::new(width, height))))
    }

    pub fn regis(start: Localization, end: Localization, var: String) -> Self {
        Self::Regis(Node::new(start, end, (var, Geometry::default())))
    }

    pub fn regis_geom(
        start: Localization,
        end: Localization,
        var: String,
        width: usize,
        height: usize,
    ) -> Self {
        Self::Regis(Node::new(start, end, (var, Geometry::new(width, height))))
    }

    pub fn csv(start: Localization, end: Localization, var: String) -> Self {
        Self::Csv(Node::new(start, end, var))
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ECommandOption;

#[derive(Clone, Debug)]
#[derive(Default)]
pub struct CommandOptions {
    pub display: HashSet<DisplayOption>,
    pub output: HashSet<OutputOptions>,
}

impl CommandOptions {
    pub fn new(output: HashSet<OutputOptions>, display: HashSet<DisplayOption>) -> Self {
        Self { display, output }
    }
}

