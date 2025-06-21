use std::str::Chars;

use parser_combinator::Parse;

use crate::{
    command::{Command, ECommand},
    command_options::{CommandOptions, ECommandOption},
    definition::EDefinition,
    expression::EExpression,
    parametric2d::EParametric2D,
    parser_common::{identity, ParseErrors, State, For, In, LowerCaseName, Colon, DoubleToken},
    range::{ERange, Range},
};

pub type CommandParseResult<'a> = Result<(Command, State, Chars<'a>), ParseErrors>;

impl<'a> Parse<'a, Chars<'a>, State, Command, ParseErrors> for ECommand {
    fn parse(&self, input: Chars<'a>, state: State) -> CommandParseResult<'a> {
        // First try parsing 3D surface with inline syntax: "expr for x in 1:10 for y in 1:10" or "expr for x in 1:10:100 for y in 1:10:100"
        let surface3d_inline = EExpression
            .pair(
                For.triple(LowerCaseName, In)
                    .second()
                    .pair(
                        // Try stepped range first: start:end:step
                        DoubleToken
                            .triple(Colon, DoubleToken)
                            .triple(Colon, DoubleToken)
                            .transform(|((s, _, e), _, st)| (s.parse::<f64>().unwrap(), e.parse::<f64>().unwrap(), Some(st.parse::<f64>().unwrap())))
                            .either(
                                // Fall back to simple range: start:end
                                DoubleToken
                                    .triple(Colon, DoubleToken)
                                    .transform(|(s, _, e)| (s.parse::<f64>().unwrap(), e.parse::<f64>().unwrap(), None))
                            )
                            .fold(identity, identity)
                    )
            )
            .pair(
                For.triple(LowerCaseName, In)
                    .second()
                    .pair(
                        // Try stepped range first: start:end:step
                        DoubleToken
                            .triple(Colon, DoubleToken)
                            .triple(Colon, DoubleToken)
                            .transform(|((s, _, e), _, st)| (s.parse::<f64>().unwrap(), e.parse::<f64>().unwrap(), Some(st.parse::<f64>().unwrap())))
                            .either(
                                // Fall back to simple range: start:end
                                DoubleToken
                                    .triple(Colon, DoubleToken)
                                    .transform(|(s, _, e)| (s.parse::<f64>().unwrap(), e.parse::<f64>().unwrap(), None))
                            )
                            .fold(identity, identity)
                    )
            )
            .pair(ECommandOption)
            .transform_with_state(|(((expr, (x_var, (x_start, x_end, x_step))), (y_var, (y_start, y_end, y_step))), options), st| {
                let x_range = match x_step {
                    Some(step) => Range::numeric_step(st.start, st.end, x_var.clone(), x_start, x_end, step),
                    None => Range::numeric(st.start, st.end, x_var.clone(), x_start, x_end),
                };
                let y_range = match y_step {
                    Some(step) => Range::numeric_step(st.start, st.end, y_var.clone(), y_start, y_end, step),
                    None => Range::numeric(st.start, st.end, y_var.clone(), y_start, y_end),
                };
                Command::new_surface3d(
                    crate::definition::Definition::new(
                        std::collections::HashMap::new(),
                        std::collections::HashMap::new(),
                    ),
                    expr,
                    x_var,
                    y_var,
                    x_range,
                    y_range,
                    options,
                )
            });
            
        // Also try 3D surface without command options (use defaults)
        let simple_surface3d_inline = EExpression
            .pair(
                For.triple(LowerCaseName, In)
                    .second()
                    .pair(
                        // Try stepped range first: start:end:step
                        DoubleToken
                            .triple(Colon, DoubleToken)
                            .triple(Colon, DoubleToken)
                            .transform(|((s, _, e), _, st)| (s.parse::<f64>().unwrap(), e.parse::<f64>().unwrap(), Some(st.parse::<f64>().unwrap())))
                            .either(
                                // Fall back to simple range: start:end
                                DoubleToken
                                    .triple(Colon, DoubleToken)
                                    .transform(|(s, _, e)| (s.parse::<f64>().unwrap(), e.parse::<f64>().unwrap(), None))
                            )
                            .fold(identity, identity)
                    )
            )
            .pair(
                For.triple(LowerCaseName, In)
                    .second()
                    .pair(
                        // Try stepped range first: start:end:step
                        DoubleToken
                            .triple(Colon, DoubleToken)
                            .triple(Colon, DoubleToken)
                            .transform(|((s, _, e), _, st)| (s.parse::<f64>().unwrap(), e.parse::<f64>().unwrap(), Some(st.parse::<f64>().unwrap())))
                            .either(
                                // Fall back to simple range: start:end
                                DoubleToken
                                    .triple(Colon, DoubleToken)
                                    .transform(|(s, _, e)| (s.parse::<f64>().unwrap(), e.parse::<f64>().unwrap(), None))
                            )
                            .fold(identity, identity)
                    )
            )
            .transform_with_state(|((expr, (x_var, (x_start, x_end, x_step))), (y_var, (y_start, y_end, y_step))), st| {
                let x_range = match x_step {
                    Some(step) => Range::numeric_step(st.start, st.end, x_var.clone(), x_start, x_end, step),
                    None => Range::numeric(st.start, st.end, x_var.clone(), x_start, x_end),
                };
                let y_range = match y_step {
                    Some(step) => Range::numeric_step(st.start, st.end, y_var.clone(), y_start, y_end, step),
                    None => Range::numeric(st.start, st.end, y_var.clone(), y_start, y_end),
                };
                Command::new_surface3d(
                    crate::definition::Definition::new(
                        std::collections::HashMap::new(),
                        std::collections::HashMap::new(),
                    ),
                    expr,
                    x_var,
                    y_var,
                    x_range,
                    y_range,
                    CommandOptions::default(),
                )
            });

        // Try 3D surface first, if it fails, try the rest
        if let Ok(result) = surface3d_inline.or_else(simple_surface3d_inline).parse(input.clone(), state.clone()) {
            return Ok(result);
        }

        // Try parsing the full command with all parts (expression)
        let full_command = EDefinition
            .pair(EExpression)
            .with_error(|err, _i| err.fold(identity, identity))
            .pair(ERange)
            .with_error(|err, _i| err.fold(identity, identity))
            .pair(ECommandOption)
            .with_error(|err, _i| err.fold(identity, identity))
            .transform(|(((def, expr), range), options)| Command::new_expression(def, expr, range, options));

        // Try parsing the full parametric command with all parts
        let full_parametric_command = EDefinition
            .pair(EParametric2D)
            .with_error(|err, _i| err.fold(identity, identity))
            .pair(ERange)
            .with_error(|err, _i| err.fold(identity, identity))
            .pair(ECommandOption)
            .with_error(|err, _i| err.fold(identity, identity))
            .transform(|(((def, parametric), range), options)| Command::new_parametric(def, parametric, range, options));

        // Try parsing without command options (use defaults)
        let simple_command = EDefinition
            .pair(EExpression)
            .with_error(|err, _i| err.fold(identity, identity))
            .pair(ERange)
            .with_error(|err, _i| err.fold(identity, identity))
            .transform(|((def, expr), range)| {
                Command::new_expression(def, expr, range, CommandOptions::default())
            });

        // Try parsing parametric without command options (use defaults)
        let simple_parametric_command = EDefinition
            .pair(EParametric2D)
            .with_error(|err, _i| err.fold(identity, identity))
            .pair(ERange)
            .with_error(|err, _i| err.fold(identity, identity))
            .transform(|((def, parametric), range)| {
                Command::new_parametric(def, parametric, range, CommandOptions::default())
            });

        // Try parsing expression with range only (no definitions)
        let expr_with_range = EExpression
            .pair(ERange)
            .with_error(|err, _i| err.fold(identity, identity))
            .pair(ECommandOption)
            .with_error(|err, _i| err.fold(identity, identity))
            .transform(|((expr, range), options)| {
                Command::new_expression(
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
                Command::new_expression(
                    crate::definition::Definition::new(
                        std::collections::HashMap::new(),
                        std::collections::HashMap::new(),
                    ),
                    expr,
                    range,
                    CommandOptions::default(),
                )
            });

        // Try parsing parametric with range only (no definitions)
        let parametric_with_range = EParametric2D
            .pair(ERange)
            .with_error(|err, _i| err.fold(identity, identity))
            .pair(ECommandOption)
            .with_error(|err, _i| err.fold(identity, identity))
            .transform(|((parametric, range), options)| {
                Command::new_parametric(
                    crate::definition::Definition::new(
                        std::collections::HashMap::new(),
                        std::collections::HashMap::new(),
                    ),
                    parametric,
                    range,
                    options,
                )
            });

        // Try parsing parametric with range only (no definitions, no options)
        let simple_parametric_with_range = EParametric2D
            .pair(ERange)
            .with_error(|err, _i| err.fold(identity, identity))
            .transform(|(parametric, range)| {
                Command::new_parametric(
                    crate::definition::Definition::new(
                        std::collections::HashMap::new(),
                        std::collections::HashMap::new(),
                    ),
                    parametric,
                    range,
                    CommandOptions::default(),
                )
            });

        // Try all variants in order of complexity
        full_command
            .or_else(full_parametric_command)
            .with_error_using_state(|err, st, _i| {
                ParseErrors::Both(st.start, st.end, Box::new(err))
            })
            .or_else(simple_command)
            .with_error_using_state(|err, st, _i| {
                ParseErrors::Both(st.start, st.end, Box::new(err))
            })
            .or_else(simple_parametric_command)
            .with_error_using_state(|err, st, _i| {
                ParseErrors::Both(st.start, st.end, Box::new(err))
            })
            .or_else(expr_with_range)
            .with_error_using_state(|err, st, _i| {
                ParseErrors::Both(st.start, st.end, Box::new(err))
            })
            .or_else(parametric_with_range)
            .with_error_using_state(|err, st, _i| {
                ParseErrors::Both(st.start, st.end, Box::new(err))
            })
            .or_else(simple_expr_with_range)
            .with_error_using_state(|err, st, _i| {
                ParseErrors::Both(st.start, st.end, Box::new(err))
            })
            .or_else(simple_parametric_with_range)
            .with_error_using_state(|err, st, _i| {
                ParseErrors::Both(st.start, st.end, Box::new(err))
            })
            .parse(input, state)
    }
}
