use parser_combinator::either::Either;

use parser_combinator::either::EitherParser;

use parser_combinator::parser::{match_anything, match_literal, Parser};
use parser_combinator::triple::Triple;
use parser_combinator::*;

use std::{marker::PhantomData, fmt::Display, str::Chars};

use parser_combinator::Parse;

use crate::expression::HasSameShape;

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Hash)]
pub struct Localization {
    pub line: usize,
    pub column: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Node<Dummy, T>
where
    T: HasSameShape,
{
    pub value: T,
    pub location: (Localization, Localization),
    pub phantom: PhantomData<Dummy>,
}


impl<Dummy, T: HasSameShape> Node<Dummy, T> {
    pub fn new(starts: Localization, ends: Localization, tree: T) -> Self {
        Self {
            value: tree,
            location: (starts, ends),
            phantom: PhantomData,
        }
    }
    /* fn starts_at(&mut self, pos: Localization) {
    self.location.0 = pos
}
    fn ends_at(&mut self, pos: Localization) {
    self.location.1 = pos
}

    fn at(&mut self, start: Localization, end: Localization) {
    self.location = (start, end)
}*/
}

impl Localization {
    pub fn new() -> Self {
        Self { line: 0, column: 0 }
    }

    pub fn at(line: usize, column: usize) -> Localization {
        Localization { line, column }
    }
}

impl<Dummy, T: HasSameShape> HasSameShape for Node<Dummy, T> {
    fn has_same_shape(&self, other: &Self) -> bool {
        self.value.has_same_shape(&other.value)
    }
}






#[derive(Clone, Debug)]
pub struct State {
    pub start: Localization,
    pub end: Localization,
}

impl State {
    pub fn new() -> State {
        State {
            start: Localization::new(),
            end: Localization::new(),
        }
    }

    pub fn transit_generator<'a>(n: usize, m: usize) -> impl Fn(State) -> State {
        move |state: State| State {
            start: state.end,
            end: Localization {
                line: state.end.line + n,
                column: state.end.column + m,
            },
        }
    }
}

pub fn state_trans(x: State) -> State {
    x
}

#[derive(Clone, Debug)]
pub enum ParseErrors {
    ExpectedButGot(Localization, Localization, String, String),
    Both(Localization, Localization, Box<(ParseErrors, ParseErrors)>),
    WhiteSpace(Localization, Localization),
    Variable(Localization, Localization),
    Sign(Localization, Localization),
    Generic(Localization, Localization),
}

macro_rules! token_implementer {
    ($type:ident,$repr:literal) => {
        #[derive(Clone, Debug, Eq, PartialEq)]
        pub struct $type;

        impl<'a> Parse<'a, Chars<'a>, State, State, ParseErrors> for $type {
            fn parse(
                &self,
                input: Chars<'a>,
                state: State,
            ) -> Result<(State, State, Chars<'a>), ParseErrors> {
                match_literal($repr.chars(), State::transit_generator($repr.len(), 0))
                    .transform_with_state(|_, state| state)
                    .with_error_using_state(|_, state, input| {
                        ParseErrors::ExpectedButGot(
                            state.start,
                            state.end,
                            $repr.to_string(),
                            input.as_str().to_string(),
                        )
                    })
                    .skip(whitespace)
                    .parse(input, state)
            }
        }
    };
}

token_implementer!(Plus, "+");
token_implementer!(Minus, "-");
token_implementer!(Star, "*");
token_implementer!(Slash, "/");
token_implementer!(Carrot, "^");
token_implementer!(LParen, "(");
token_implementer!(RParen, ")");
token_implementer!(Dot, ".");
token_implementer!(Comma, ",");
token_implementer!(Equal, "=");
token_implementer!(SemiColon, ";");

token_implementer!(Colon, ":");
token_implementer!(For, "for");
token_implementer!(In, "in");
token_implementer!(With, "with");

//Display tokens
token_implementer!(RegisToken, "display=regis");
token_implementer!(SixelToken, "display=sixel");
token_implementer!(AsciiToken, "display=ascii");
token_implementer!(AnciToken, "display=ansi");

//Output tokens
token_implementer!(PngToken, "png=");
token_implementer!(JpgToken, "jpg=");
token_implementer!(LatexToken, "latex=");
token_implementer!(SixelDToken, "sixel=");
token_implementer!(RegisDToken, "regis=");
token_implementer!(CsvToken, "csv=");

//Geometry tokens
token_implementer!(GeometryToken, "geometry=");
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IntegerToken;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DoubleToken;
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LowerCaseName;
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AsciiAnythingUpToSpace;

impl<'a> Parse<'a, Chars<'a>, State, String, String> for AsciiAnythingUpToSpace {
    fn parse(&self, input: Chars<'a>, state: State) -> Result<(String, State, Chars<'a>), String> {
        match_anything(State::transit_generator(1, 0))
            .validate(
                |character: &char| {
                    character.is_ascii() && !character.is_whitespace() && character != &'\n'
                },
                "alphabetic character".to_string(),
            )
            .one_or_more()
            .transform(|x| x.into_iter().collect::<String>())
            .parse(input, state)
    }
}

impl<'a> Parse<'a, Chars<'a>, State, String, String> for LowerCaseName {
    fn parse(&self, input: Chars<'a>, state: State) -> Result<(String, State, Chars<'a>), String> {
        match_anything(State::transit_generator(1, 0))
            .validate(
                |character: &char| character.is_alphabetic() && character.is_ascii_lowercase(),
                "alphabetic character".to_string(),
            )
            .one_or_more()
            .transform(|x| x.into_iter().collect::<String>())
            .parse(input, state)
    }
}

impl<'a> Parse<'a, Chars<'a>, State, String, String> for IntegerToken {
    fn parse(&self, input: Chars<'a>, state: State) -> Result<(String, State, Chars<'a>), String> {
        let to_string = |characters: Vec<char>| {
            characters
                .iter()
                .fold(String::new(), |mut result, character| {
                    result.push(*character);
                    result
                })
        };

        let int_parser = match_anything(State::transit_generator(1, 0))
            .validate(
                |character: &char| character.is_numeric(),
                "numeric character".to_string(),
            )
            .one_or_more()
            .with_error(|_, i: Chars<'a>| format!("integer parsing failed got {}", i.as_str()))
            .transform(to_string);
        int_parser.parse(input, state)
    }
}

impl<'a> Parse<'a, Chars<'a>, State, String, String> for DoubleToken {
    fn parse(&self, input: Chars<'a>, state: State) -> Result<(String, State, Chars<'a>), String> {
        IntegerToken
            .triple(Dot, IntegerToken)
            .transform(|(a, _, c)| format!("{a}.{c}"))
            .with_error(|_, i: Chars<'a>| format!("float parsing failed got {}", i.as_str()))
            .either(IntegerToken)
            .fold(identity, identity)
            .with_error(|_, i: Chars<'a>| format!("float parsing failed got {}", i.as_str()))
            .parse(input, state)
    }
}

pub fn identity<T>(x: T) -> T {
    x
}

pub fn whitespace<'a>(
    input: Chars<'a>,
    state: State,
) -> parser_combinator::ParseResult<Chars<'a>, State, char, ParseErrors> {
    let space = match_anything(State::transit_generator(1, 0)).validate(
        |character: &char| character == &' ',
        "alphabetic character".to_string(),
    );
    let newline = match_anything(State::transit_generator(0, 1)).validate(
        |character: &char| character == &'\n',
        "alphabetic character".to_string(),
    );

    space
        .or_else(newline)
        .with_error_using_state(|(_a, _b), state, _| {
            ParseErrors::WhiteSpace(state.start, state.end)
        })
        .parse(input, state)
}
