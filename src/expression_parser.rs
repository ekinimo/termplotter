use parser_combinator::either::Either;

use parser_combinator::either::EitherParser;

use parser_combinator::parser::{match_anything, match_literal, Parser};
use parser_combinator::triple::Triple;
use parser_combinator::*;
use std::str::Chars;

use crate::expression::*;
use crate::parser_common::{*};


pub type ExprParseResult<'a> = Result<(ExpressionSyntaxTree<String>, State, Chars<'a>), ParseErrors>;

/*
macro_rules! ASTBuilder {
    ($name:ident => $production:expr  ; $transformation:expr ; $error:expr) => {
        impl<'a> Parse<'a, Chars<'a>, State, ExpressionSyntaxTree<String>, ParseErrors> for $name {
            fn parse(&self, input: Chars<'a>, state: State) -> ParseResult<'a> {
                $production
                    .transform($transformation)
                    .with_error_using_state($error)
                    .parse(input, state)
            }
        }
    };
}

#[derive(Clone, Debug)]
struct A;

macro_rules! token_combiner {
    ($a:ident | $b:ident) => {
        EitherParser::new($a, $b).with_error_using_state(|err, state, _i| {
            ParseErrors::Both(state.start, state.end, Box::new(err))
        })
    };
}

ASTBuilder!(A => Self.right_assoc(EMulOrDiv, token_combiner!(Plus | Minus) ) ;
            binop_transform(ExpressionSyntaxTree::add, ExpressionSyntaxTree::sub) ;
            |x, _state, _i| match x {
    Either::Left(a) | Either::Right(Either::Right(a)) | Either::Right(Either::Left(a)) => a,
}  );
*/



#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct ESumOrSub;
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EMulOrDiv;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EAtom;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EBracketedExpression;

fn binop_transform<T: VariableSuperTrait>(
    left_fun: impl Fn(
        Localization,
        Localization,
        ExpressionSyntaxTree<T>,
        ExpressionSyntaxTree<T>,
    ) -> ExpressionSyntaxTree<T>,
    right_fun: impl Fn(
        Localization,
        Localization,
        ExpressionSyntaxTree<T>,
        ExpressionSyntaxTree<T>,
    ) -> ExpressionSyntaxTree<T>,
) -> impl Fn(
    (
        ExpressionSyntaxTree<T>,
        Vec<(Either<State, State>, ExpressionSyntaxTree<T>)>,
    ),
) -> ExpressionSyntaxTree<T> {
    move |(mut left, y)| {
        if y.is_empty() {
            return left;
        }
        for (eith, right) in y.into_iter() {
            match eith {
                Either::Left(state) => {
                    left = left_fun(state.start, state.end, left, right);
                }
                Either::Right(state) => {
                    left = right_fun(state.start, state.end, left, right);
                }
            }
        }
        left
    }
}

impl<'a> Parse<'a, Chars<'a>, State, ExpressionSyntaxTree<String>, ParseErrors> for ESumOrSub {
    fn parse(&self, input: Chars<'a>, state: State) -> ExprParseResult<'a> {
        let a = EitherParser::new(Plus, Minus).with_error_using_state(|err, state, _i| {
            ParseErrors::Both(state.start, state.end, Box::new(err))
        });
        let b = Self.right_assoc(EMulOrDiv, a);
        b.transform(binop_transform(
            ExpressionSyntaxTree::add,
            ExpressionSyntaxTree::sub,
        ))
        .with_error_using_state(|x, _state, _i| match x {
            Either::Left(a) | Either::Right(Either::Right(a)) | Either::Right(Either::Left(a)) => a,
        })
        .parse(input, state)
    }
}

impl<'a> Parse<'a, Chars<'a>, State, ExpressionSyntaxTree<String>, ParseErrors> for EMulOrDiv {
    fn parse(&self, input: Chars<'a>, state: State) -> ExprParseResult<'a> {
        let a = EitherParser::new(Star, Slash).with_error_using_state(|err, state, _i| {
            ParseErrors::Both(state.start, state.end, Box::new(err))
        });
        let b = Self.right_assoc(EExp, a);
        b.transform(binop_transform(
            ExpressionSyntaxTree::mul,
            ExpressionSyntaxTree::div,
        ))
        .with_error_using_state(|x, _state, _i| match x {
            Either::Left(a) | Either::Right(Either::Right(a)) | Either::Right(Either::Left(a)) => a,
        })
        .parse(input, state)
    }
}

impl<'a> Parse<'a, Chars<'a>, State, ExpressionSyntaxTree<String>, ParseErrors> for EExp {
    fn parse(&self, input: Chars<'a>, state: State) -> ExprParseResult<'a> {
        let a = Carrot.transform_with_state(|_, s| s);
        Self.left_assoc(a, EAtom.or_else(ENeg))
            .transform(|x| match x {
                Either::Left((x, s, y)) => ExpressionSyntaxTree::exp(s.start, s.end, x, y),
                Either::Right(x) => x,
            })
            .with_error_using_state(|x, state, _i| {
                ParseErrors::Both(state.start, state.end, Box::new(x.1))
            })
            .parse(input, state)
    }
}

impl<'a> Parse<'a, Chars<'a>, State, ExpressionSyntaxTree<String>, ParseErrors> for ENeg {
    fn parse(&self, input: Chars<'a>, state: State) -> ExprParseResult<'a> {
        Minus
            .pair(EExp)
            .transform_with_state(move |(_op, x), state| {
                ExpressionSyntaxTree::neg(state.start, state.end, x)
            })
            .with_error_using_state(|_err, state: State, _input| {
                ParseErrors::Sign(state.start, state.end)
            })
            .skip(whitespace)
            .parse(input, state)
    }
}

impl<'a> Parse<'a, Chars<'a>, State, ExpressionSyntaxTree<String>, ParseErrors>
    for EBracketedExpression
{
    fn parse(&self, input: Chars<'a>, state: State) -> ExprParseResult<'a> {
        Triple::new(LParen, EExpression, RParen)
            .second()
            .with_error_using_state(|_x, state, input| {
                ParseErrors::ExpectedButGot(
                    state.start,
                    state.end,
                    "( or )".to_string(),
                    input.as_str().to_string(),
                )
            })
            .parse(input, state)
    }
}

impl<'a> Parse<'a, Chars<'a>, State, ExpressionSyntaxTree<String>, ParseErrors> for EAtom {
    fn parse(&self, input: Chars<'a>, state: State) -> ExprParseResult<'a> {
        EFun.or_else(ENum)
            .or_else(EVar)
            .or_else(EBracketedExpression)
            .with_error_using_state(|((_f_err, _n_err), _var_err), s, i| {
                ParseErrors::ExpectedButGot(
                    s.start,
                    s.end,
                    "funcall or number or variable".into(),
                    i.collect(),
                )
            })
            .skip(whitespace)
            .parse(input, state)
    }
}

impl<'b> Parse<'b, Chars<'b>, State, ExpressionSyntaxTree<String>, ParseErrors> for EVar {
    fn parse(&self, input: Chars<'b>, state: State) -> ExprParseResult<'b> {
        LowerCaseName
            .transform_with_state(move |x, s| ExpressionSyntaxTree::variable(state.end, s.end, x))
            .with_error_using_state(|_err, state, _input| {
                ParseErrors::Variable(state.start, state.end)
            })
            .parse(input, state)
    }
}

impl<'a> Parse<'a, Chars<'a>, State, ExpressionSyntaxTree<String>, ParseErrors> for EExpression {
    fn parse(&self, input: Chars<'a>, state: State) -> ExprParseResult<'a> {
        if !input.clone().any(|char| !char.is_whitespace()) {
            return Err(ParseErrors::ExpectedButGot(
                state.start,
                state.end,
                "something".into(),
                "nothing".into(),
            ));
        }
        ESumOrSub.parse(input, state)
    }
}

impl<'a> Parse<'a, Chars<'a>, State, ExpressionSyntaxTree<String>, ParseErrors> for ENum {
    fn parse(&self, input: Chars<'a>, state: State) -> ExprParseResult<'a> {
        DoubleToken
            .transform_with_state(move |num, curr_state| {
                ExpressionSyntaxTree::number(state.end, curr_state.end, num.parse::<f64>().unwrap())
            })
            .with_error_using_state(|err, state: State, _input| {
                ParseErrors::ExpectedButGot(state.start, state.end, "variable".into(), err)
            })
            .parse(input, state)
    }
}



impl<'a> Parse<'a, Chars<'a>, State, ExpressionSyntaxTree<String>, ParseErrors> for EFun {
    fn parse(&self, input: Chars<'a>, state: State) -> ExprParseResult<'a> {
        let tuple = Triple::new(
            LParen,
            EExpression
                .separated_by(match_literal(",".chars(), state_trans))
                .with_error_using_state(|_, state, input| {
                    ParseErrors::ExpectedButGot(
                        state.start,
                        state.end,
                        ",".to_string(),
                        input.as_str().to_string(),
                    )
                })
                .transform(|(x, y)| {
                    let mut ret = vec![x];
                    ret.extend(y.iter().map(|(_a, b)| b.to_owned()));
                    ret
                }),
            RParen,
        )
        .second()
        .with_error(|err, _input| err.fold(identity, identity, identity));

        match_anything(State::transit_generator(1, 0))
            .validate(
                |character: &char| character.is_alphabetic() && character.is_lowercase(),
                "alphabetic character".to_string(),
            )
            .with_error_using_state(|_, state, input: Chars| {
                ParseErrors::ExpectedButGot(
                    state.start,
                    state.end,
                    "alphabetic character".to_string(),
                    input.as_str().to_string(),
                )
            })
            .one_or_more()
            .pair(tuple)
            .transform_with_state(move |(x, y), curr_state| {
                ExpressionSyntaxTree::fun(state.end, curr_state.end, x.into_iter().collect(), y)
            })
            .with_error(|err, _input| err.fold(identity, identity))
            .parse(input, state)
    }
}



