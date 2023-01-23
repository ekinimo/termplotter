use std::str::Chars;

use parser_combinator::Parse;

use crate::{command::{Command, ECommand}, parser_common::{State, ParseErrors, identity}, expression::EExpression, range::ERange, command_options::{ECommandOption, CommandOptions}, definition::EDefinition, expression_parser::{*}};


pub type CommandParseResult<'a> = Result<(Command, State, Chars<'a>), ParseErrors>;

impl<'a> Parse<'a, Chars<'a>, State, Command, ParseErrors> for ECommand {
    
    fn parse(&self, input: Chars<'a>, state: State) -> CommandParseResult<'a> {
         EDefinition
            .pair(EExpression)
            .with_error(|err, _i| err.fold(identity, identity))
            .pair(ERange)
            .with_error(|err, _i| err.fold(identity, identity))
            .pair(ECommandOption)
            .with_error(|err, _i| err.fold(identity, identity))
            .transform(|(((def, expr), range), options)| Command::new(def, expr, range, options))
            .either(
                EDefinition
                    .pair(EExpression)
                    .with_error(|err, _i| err.fold(identity, identity))
                    .pair(ERange)
                    .with_error(|err, _i| err.fold(identity, identity))
                    .transform(|((def, expr), range)| {
                        Command::new(def, expr, range, CommandOptions::default())
                    }),
            )
            .fold(identity, identity)
            .with_error_using_state(|err,st,_i| ParseErrors::Both(st.start, st.end, Box::new(err))).parse(input,state)
        
    }
}
