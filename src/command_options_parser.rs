use std::{str::Chars, collections::HashSet};

use parser_combinator::{Parse, either::Either};

use crate::{command_options::*, parser_common::{State, ParseErrors, GeometryToken, IntegerToken, Comma, AsciiAnythingUpToSpace, With, SixelToken, AsciiToken, AnciToken, RegisToken}};

pub type DisplayParseResult<'a> = Result<(DisplayOption, State, Chars<'a>), ParseErrors>;

impl<'a> Parse<'a, Chars<'a>, State, DisplayOption, ParseErrors> for EDisplayRegis {
    fn parse(&self, input: Chars<'a>, state: State) -> DisplayParseResult<'a> {
        RegisToken
            .transform_with_state(|_, s| DisplayOption::regis(s.start, s.end))
            .with_error_using_state(|_, s, _| ParseErrors::Generic(s.start, s.end))
            .parse(input, state)
    }
}

impl<'a> Parse<'a, Chars<'a>, State, DisplayOption, ParseErrors> for EDisplaySixel {
    fn parse(&self, input: Chars<'a>, state: State) -> DisplayParseResult<'a> {
        SixelToken
            .transform_with_state(|_, s| DisplayOption::sixel(s.start, s.end))
            .with_error_using_state(|_, s, _| ParseErrors::Generic(s.start, s.end))
            .parse(input, state)
    }
}

impl<'a> Parse<'a, Chars<'a>, State, DisplayOption, ParseErrors> for EDisplayAscii {
    fn parse(&self, input: Chars<'a>, state: State) -> DisplayParseResult<'a> {
        AsciiToken
            .transform_with_state(|_, s| DisplayOption::ascii(s.start, s.end))
            .with_error_using_state(|_, s, _| ParseErrors::Generic(s.start, s.end))
            .parse(input, state)
    }
}

impl<'a> Parse<'a, Chars<'a>, State, DisplayOption, ParseErrors> for EDisplayAnsi {
    fn parse(&self, input: Chars<'a>, state: State) -> DisplayParseResult<'a> {
        AnciToken
            .transform_with_state(|_, s| DisplayOption::ansi(s.start, s.end))
            .with_error_using_state(|_, s, _| ParseErrors::Generic(s.start, s.end))
            .parse(input, state)
    }
}

impl<'a> Parse<'a, Chars<'a>, State, DisplayOption, ParseErrors> for EDisplay {
    fn parse(&self, input: Chars<'a>, state: State) -> DisplayParseResult<'a> {
        EDisplayAnsi
            .or_else(EDisplayAscii)
            .or_else(EDisplaySixel)
            .or_else(EDisplayRegis)
            .with_error_using_state(|_, s, _| ParseErrors::Generic(s.start, s.end))
            .parse(input, state)
    }
}

pub type OutputParseResult<'a> = Result<(OutputOptions, State, Chars<'a>), ParseErrors>;
impl<'a> Parse<'a, Chars<'a>, State, OutputOptions, ParseErrors> for EOutputPNG {
    fn parse(&self, input: Chars<'a>, state: State) -> OutputParseResult<'a> {
        EOutputPNG
            .pair(AsciiAnythingUpToSpace)
            .transform_with_state(|(_, var), s| OutputOptions::png(s.start, s.end, var))
            .with_error_using_state(|_, s, _| ParseErrors::Generic(s.start, s.end))
            .parse(input, state)
    }
}

impl<'a> Parse<'a, Chars<'a>, State, OutputOptions, ParseErrors> for EOutputJPG {
    fn parse(&self, input: Chars<'a>, state: State) -> OutputParseResult<'a> {
        EOutputJPG
            .pair(AsciiAnythingUpToSpace)
            .transform_with_state(|(_, var), s| OutputOptions::jpg(s.start, s.end, var))
            .with_error_using_state(|_, s, _| ParseErrors::Generic(s.start, s.end))
            .parse(input, state)
    }
}

impl<'a> Parse<'a, Chars<'a>, State, OutputOptions, ParseErrors> for EOutputLaTeX {
    fn parse(&self, input: Chars<'a>, state: State) -> OutputParseResult<'a> {
        EOutputLaTeX
            .pair(AsciiAnythingUpToSpace)
            .transform_with_state(|(_, var), s| OutputOptions::latex(s.start, s.end, var))
            .with_error_using_state(|_, s, _| ParseErrors::Generic(s.start, s.end))
            .parse(input, state)
    }
}

impl<'a> Parse<'a, Chars<'a>, State, OutputOptions, ParseErrors> for EOutputSixel {
    fn parse(&self, input: Chars<'a>, state: State) -> OutputParseResult<'a> {
        EOutputSixel
            .pair(AsciiAnythingUpToSpace)
            //.or_else(EOutputSixel
            //         .pair(AsciiAnythingUpToSpace)
            //)
            .transform_with_state(|(_, var), s| OutputOptions::sixel(s.start, s.end, var))
            .with_error_using_state(|_, s, _| ParseErrors::Generic(s.start, s.end))
            .parse(input, state)
    }
}

impl<'a> Parse<'a, Chars<'a>, State, OutputOptions, ParseErrors> for EOutputRegis {
    fn parse(&self, input: Chars<'a>, state: State) -> OutputParseResult<'a> {
        EOutputRegis
            .pair(AsciiAnythingUpToSpace)
            .transform_with_state(|(_, var), s| OutputOptions::regis(s.start, s.end, var))
            .with_error_using_state(|_, s, _| ParseErrors::Generic(s.start, s.end))
            .parse(input, state)
    }
}

impl<'a> Parse<'a, Chars<'a>, State, OutputOptions, ParseErrors> for EOutputCSV {
    fn parse(&self, input: Chars<'a>, state: State) -> OutputParseResult<'a> {
        EOutputCSV
            .pair(AsciiAnythingUpToSpace)
            .transform_with_state(|(_, var), s| OutputOptions::csv(s.start, s.end, var))
            .with_error_using_state(|_, s, _| ParseErrors::Generic(s.start, s.end))
            .parse(input, state)
    }
}

struct EOutputPNGwithGeometry;
impl<'a> Parse<'a, Chars<'a>, State, OutputOptions, ParseErrors> for EOutputPNGwithGeometry {
    fn parse(&self, input: Chars<'a>, state: State) -> OutputParseResult<'a> {
        EOutputPNG
            .pair(AsciiAnythingUpToSpace)
            .second()
            .with_error_using_state(|_, s, _| ParseErrors::Generic(s.start, s.end))
            .triple(
                GeometryToken,
                IntegerToken
                    .triple(Comma, IntegerToken)
                    .transform(|(a, _, b)| {
                        (a.parse::<usize>().unwrap(), b.parse::<usize>().unwrap())
                    })
                    .with_error_using_state(|_, s, _| ParseErrors::Generic(s.start, s.end)),
            )
            .transform_with_state(|(var, _, (a, b)), s| {
                OutputOptions::png_geom(s.start, s.end, var, a, b)
            })
            .with_error_using_state(|_, s, _| ParseErrors::Generic(s.start, s.end))
            .parse(input, state)
    }
}

struct EOutputJPGwithGeometry;
impl<'a> Parse<'a, Chars<'a>, State, OutputOptions, ParseErrors> for EOutputJPGwithGeometry {
    fn parse(&self, input: Chars<'a>, state: State) -> OutputParseResult<'a> {
        EOutputJPG
            .pair(AsciiAnythingUpToSpace)
            .second()
            .with_error_using_state(|_, s, _| ParseErrors::Generic(s.start, s.end))
            .triple(
                GeometryToken,
                IntegerToken
                    .triple(Comma, IntegerToken)
                    .transform(|(a, _, b)| {
                        (a.parse::<usize>().unwrap(), b.parse::<usize>().unwrap())
                    })
                    .with_error_using_state(|_, s, _| ParseErrors::Generic(s.start, s.end)),
            )
            .transform_with_state(|(var, _, (a, b)), s| {
                OutputOptions::jpg_geom(s.start, s.end, var, a, b)
            })
            .with_error_using_state(|_, s, _| ParseErrors::Generic(s.start, s.end))
            .parse(input, state)
    }
}

struct EOutputLatexWithGeometry;
impl<'a> Parse<'a, Chars<'a>, State, OutputOptions, ParseErrors> for EOutputLatexWithGeometry {
    fn parse(&self, input: Chars<'a>, state: State) -> OutputParseResult<'a> {
        EOutputJPG
            .pair(AsciiAnythingUpToSpace)
            .second()
            .with_error_using_state(|_, s, _| ParseErrors::Generic(s.start, s.end))
            .triple(
                GeometryToken,
                IntegerToken
                    .triple(Comma, IntegerToken)
                    .transform(|(a, _, b)| {
                        (a.parse::<usize>().unwrap(), b.parse::<usize>().unwrap())
                    })
                    .with_error_using_state(|_, s, _| ParseErrors::Generic(s.start, s.end)),
            )
            .transform_with_state(|(var, _, (a, b)), s| {
                OutputOptions::latex_geom(s.start, s.end, var, a, b)
            })
            .with_error_using_state(|_, s, _| ParseErrors::Generic(s.start, s.end))
            .parse(input, state)
    }
}

struct EOutputSixelWithGeometry;
impl<'a> Parse<'a, Chars<'a>, State, OutputOptions, ParseErrors> for EOutputSixelWithGeometry {
    fn parse(&self, input: Chars<'a>, state: State) -> OutputParseResult<'a> {
        EOutputJPG
            .pair(AsciiAnythingUpToSpace)
            .second()
            .with_error_using_state(|_, s, _| ParseErrors::Generic(s.start, s.end))
            .triple(
                GeometryToken,
                IntegerToken
                    .triple(Comma, IntegerToken)
                    .transform(|(a, _, b)| {
                        (a.parse::<usize>().unwrap(), b.parse::<usize>().unwrap())
                    })
                    .with_error_using_state(|_, s, _| ParseErrors::Generic(s.start, s.end)),
            )
            .transform_with_state(|(var, _, (a, b)), s| {
                OutputOptions::sixel_geom(s.start, s.end, var, a, b)
            })
            .with_error_using_state(|_, s, _| ParseErrors::Generic(s.start, s.end))
            .parse(input, state)
    }
}

struct EOutputRegisWithGeometry;
impl<'a> Parse<'a, Chars<'a>, State, OutputOptions, ParseErrors> for EOutputRegisWithGeometry {
    fn parse(&self, input: Chars<'a>, state: State) -> OutputParseResult<'a> {
        EOutputJPG
            .pair(AsciiAnythingUpToSpace)
            .second()
            .with_error_using_state(|_, s, _| ParseErrors::Generic(s.start, s.end))
            .triple(
                GeometryToken,
                IntegerToken
                    .triple(Comma, IntegerToken)
                    .transform(|(a, _, b)| {
                        (a.parse::<usize>().unwrap(), b.parse::<usize>().unwrap())
                    })
                    .with_error_using_state(|_, s, _| ParseErrors::Generic(s.start, s.end)),
            )
            .transform_with_state(|(var, _, (a, b)), s| {
                OutputOptions::regis_geom(s.start, s.end, var, a, b)
            })
            .with_error_using_state(|_, s, _| ParseErrors::Generic(s.start, s.end))
            .parse(input, state)
    }
}

impl<'a> Parse<'a, Chars<'a>, State, OutputOptions, ParseErrors> for EOutput {
    fn parse(&self, input: Chars<'a>, state: State) -> OutputParseResult<'a> {
        EOutputCSV
            .or_else(EOutputLatexWithGeometry.or_else(EOutputLaTeX))
            .or_else(EOutputJPGwithGeometry.or_else(EOutputJPG))
            .or_else(EOutputPNGwithGeometry.or_else(EOutputPNG))
            .or_else(EOutputRegisWithGeometry.or_else(EOutputRegis))
            .or_else(EOutputSixelWithGeometry.or_else(EOutputSixel))
            .with_error_using_state(|_, s, _| ParseErrors::Generic(s.start, s.end))
            .parse(input, state)
    }
}

type CommandOptionParseResult<'a> = Result<(CommandOptions, State, Chars<'a>), ParseErrors>;

impl<'a> Parse<'a, Chars<'a>, State, CommandOptions, ParseErrors> for ECommandOption {
    fn parse(&self, input: Chars<'a>, state: State) -> CommandOptionParseResult<'a> {
        With.pair(EOutput.either(EDisplay).zero_or_more())
            .second()
            .transform(|x| {
                let mut map1 = HashSet::new();
                let mut map2 = HashSet::new();
                for i in x.into_iter() {
                    match i {
                        Either::Left(output) => map1.insert(output),
                        Either::Right(display) => map2.insert(display),
                    };
                }

                CommandOptions::new(map1, map2)
            })
            .with_error_using_state(|_, s, _| ParseErrors::Generic(s.start, s.end))
            .parse(input, state)
    }
}
