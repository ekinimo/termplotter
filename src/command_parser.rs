use std::str::Chars;

use parser_combinator::Parse;

use crate::{
    command::{Command, ECommand},
    command_options::{CommandOptions, ECommandOption},
    definition::EDefinition,
    expression::EExpression,
    expression_parser::*,
    parser_common::{identity, ParseErrors, State},
    range::ERange,
};

pub type CommandParseResult<'a> = Result<(Command, State, Chars<'a>), ParseErrors>;

impl<'a> Parse<'a, Chars<'a>, State, Command, ParseErrors> for ECommand {
    fn parse(&self, input: Chars<'a>, state: State) -> CommandParseResult<'a> {
        // Try parsing the full command with all parts
        let full_command = EDefinition
            .pair(EExpression)
            .with_error(|err, _i| err.fold(identity, identity))
            .pair(ERange)
            .with_error(|err, _i| err.fold(identity, identity))
            .pair(ECommandOption)
            .with_error(|err, _i| err.fold(identity, identity))
            .transform(|(((def, expr), range), options)| Command::new(def, expr, range, options));

        // Try parsing without command options (use defaults)
        let simple_command = EDefinition
            .pair(EExpression)
            .with_error(|err, _i| err.fold(identity, identity))
            .pair(ERange)
            .with_error(|err, _i| err.fold(identity, identity))
            .transform(|((def, expr), range)| {
                Command::new(def, expr, range, CommandOptions::default())
            });

        // Try parsing expression with range only (no definitions)
        let expr_with_range = EExpression
            .pair(ERange)
            .with_error(|err, _i| err.fold(identity, identity))
            .pair(ECommandOption)
            .with_error(|err, _i| err.fold(identity, identity))
            .transform(|((expr, range), options)| {
                Command::new(
                    crate::definition::Definition::new(
                        std::collections::HashMap::new(),
                        std::collections::HashMap::new(),
                    ),
                    expr,
                    range,
                    options,
                )
            });

        // Try parsing expression with range only (no definitions, no options)
        let simple_expr_with_range = EExpression
            .pair(ERange)
            .with_error(|err, _i| err.fold(identity, identity))
            .transform(|(expr, range)| {
                Command::new(
                    crate::definition::Definition::new(
                        std::collections::HashMap::new(),
                        std::collections::HashMap::new(),
                    ),
                    expr,
                    range,
                    CommandOptions::default(),
                )
            });

        // Try all variants in order of complexity
        full_command
            .or_else(simple_command)
            .with_error_using_state(|err, st, _i| {
                ParseErrors::Both(st.start, st.end, Box::new(err))
            })
            .or_else(expr_with_range)
            .with_error_using_state(|err, st, _i| {
                ParseErrors::Both(st.start, st.end, Box::new(err))
            })
            .or_else(simple_expr_with_range)
            .with_error_using_state(|err, st, _i| {
                ParseErrors::Both(st.start, st.end, Box::new(err))
            })
            .parse(input, state)
    }
}
