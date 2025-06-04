use std::str::Chars;

use parser_combinator::Parse;

use crate::{
    parser_common::{
        identity, AsciiAnythingUpToSpace, Colon, DoubleToken, For, In, IntegerToken, LowerCaseName,
        ParseErrors, State,
    },
    range::{ERange, ERangeFile, ERangeFileCol, ERangeNumeric, ERangeNumericStep, Range},
};

type RangeParseResult<'a> = Result<(Range, State, Chars<'a>), ParseErrors>;

impl<'a> Parse<'a, Chars<'a>, State, Range, ParseErrors> for ERangeNumeric {
    fn parse(&self, input: Chars<'a>, state: State) -> RangeParseResult<'a> {
        For.triple(LowerCaseName, In)
            .second()
            .with_error_using_state(|x, s, i| ParseErrors::Generic(s.start, s.end))
            .pair(
                DoubleToken
                    .triple(Colon, DoubleToken)
                    .transform(|(s, _, e)| (s.parse::<f64>().unwrap(), e.parse::<f64>().unwrap()))
                    .with_error(|x, _| x.fold(identity, |_| "error".into(), identity)),
            )
            .transform_with_state(|(var, (starts_at, ends_at)), st| {
                Range::numeric(st.start, st.end, var, starts_at, ends_at)
            })
            .with_error_using_state(|_, s, _| ParseErrors::Generic(s.start, s.end))
            .parse(input, state)
    }
}
impl<'a> Parse<'a, Chars<'a>, State, Range, ParseErrors> for ERangeNumericStep {
    fn parse(&self, input: Chars<'a>, state: State) -> RangeParseResult<'a> {
        For.triple(LowerCaseName, In)
            .second()
            .with_error_using_state(|x, s, i| ParseErrors::Generic(s.start, s.end))
            .pair(
                DoubleToken
                    .triple(Colon, DoubleToken)
                    .transform(|(s, _, e)| (s.parse::<f64>().unwrap(), e.parse::<f64>().unwrap()))
                    .with_error(|x, _| x.fold(identity, |_| "error".into(), identity))
                    .triple(Colon, DoubleToken)
                    .transform(|((s, e), _, st)| (s, e, st.parse::<f64>().unwrap()))
                    .with_error(|x, _| x.fold(identity, |_| "error".into(), identity)),
            )
            .transform_with_state(|(var, (starts_at, ends_at, step)), st| {
                Range::numeric_step(st.start, st.end, var, starts_at, ends_at, step)
            })
            .with_error_using_state(|_, s, _| ParseErrors::Generic(s.start, s.end))
            .parse(input, state)
    }
}
impl<'a> Parse<'a, Chars<'a>, State, Range, ParseErrors> for ERangeFile {
    fn parse(&self, input: Chars<'a>, state: State) -> RangeParseResult<'a> {
        For.triple(LowerCaseName, In)
            .second()
            .with_error_using_state(|x, s, i| ParseErrors::Generic(s.start, s.end))
            .pair(AsciiAnythingUpToSpace)
            .transform_with_state(|(var, filename), st| {
                Range::file(st.start, st.end, var, filename)
            })
            .with_error_using_state(|_, s, _| ParseErrors::Generic(s.start, s.end))
            .parse(input, state)
    }
}

impl<'a> Parse<'a, Chars<'a>, State, Range, ParseErrors> for ERangeFileCol {
    fn parse(&self, input: Chars<'a>, state: State) -> RangeParseResult<'a> {
        For.triple(LowerCaseName, In)
            .second()
            .with_error_using_state(|x, s, i| ParseErrors::Generic(s.start, s.end))
            .pair(AsciiAnythingUpToSpace)
            .with_error_using_state(|_, s, _| ParseErrors::Generic(s.start, s.end))
            .pair(
                Colon
                    .pair(IntegerToken)
                    .second()
                    .transform(|x| x.parse::<usize>().unwrap())
                    .with_error_using_state(|_, s, _| ParseErrors::Generic(s.start, s.end)),
            )
            .transform_with_state(|((var, name), col), s| {
                Range::file_col(s.start, s.end, var, name, col)
            })
            .with_error_using_state(|_, s, _| ParseErrors::Generic(s.start, s.end))
            .parse(input, state)
    }
}

impl<'a> Parse<'a, Chars<'a>, State, Range, ParseErrors> for ERange {
    fn parse(&self, input: Chars<'a>, state: State) -> RangeParseResult<'a> {
        ERangeNumericStep
            .or_else(ERangeNumeric)
            .or_else(ERangeFileCol)
            .or_else(ERangeFile)
            .with_error(|(a, b), i| b)
            .parse(input, state)
    }
}
