#![feature(float_erf)]
#![feature(float_gamma)]
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(elided_named_lifetimes)]

use std::{collections::HashMap, env, io::stdin};

use parser_combinator::Parse;

use crate::{
    command::ECommand,
    definition::Definition,
    eval::Eval,
    eval_command::evaluate_command,
    eval_expression::DummyExpr,
    expression::ExpressionSyntaxTree,
    parser_common::State,
    values::ExpressionRange1dResult,
};

mod command;
mod command_options;
mod command_options_parser;
mod command_parser;
mod context;
mod definition;
mod definition_parser;
mod display;
mod eval;
mod eval_command;
mod eval_expression;
mod eval_range;
mod expression;
mod expression_parser;
mod parametric2d;
mod parser_common;
mod range;
mod range_parser;
mod values;

fn main() {
    let args: Vec<String> = env::args().collect();

    match args.len() {
        1 => interactive_mode(),
        2 => {
            if args[1] == "--help" || args[1] == "-h" {
                print_help();
            } else {
                execute_expression(&args[1]);
            }
        }
        _ => {
            let expression = args[1..].join(" ");
            execute_expression(&expression);
        }
    }
}

fn print_help() {
    println!("Mathematical Expression Parser and Plotter");
    println!();
    println!("USAGE:");
    println!(
        "  {} [EXPRESSION]",
        env::args().next().unwrap_or_else(|| "program".to_string())
    );
    println!(
        "  {} --help",
        env::args().next().unwrap_or_else(|| "program".to_string())
    );
    println!();
    println!("MODES:");
    println!("  Interactive mode: Run without arguments to enter expressions interactively");
    println!("  Command mode:     Provide expression as command line argument");
    println!();
    println!("EXPRESSION SYNTAX:");
    println!("  Variables:        x, y, a, b, etc.");
    println!("  Numbers:          1, 2.5, -3.14");
    println!("  Operations:       +, -, *, /, ^");
    println!("  Functions:        abs(x), max(x,y), clamp(x,min,max)");
    println!("  Parentheses:      (expression)");
    println!();
    println!("COMMAND SYNTAX:");
    println!("  [definitions] expression for var in range [with options]");
    println!();
    println!("DEFINITIONS:");
    println!("  constant = value;");
    println!("  function(args) = expression;");
    println!();
    println!("RANGES:");
    println!("  for x in start:end           - Numeric range");
    println!("  for x in start:end:step      - Numeric range with step");
    println!("  for x in filename            - Read from file");
    println!("  for x in filename:column     - Read specific column from file");
    println!();
    println!("DISPLAY OPTIONS:");
    println!("  display=regis     - REGIS graphics (default)");
    println!("  display=sixel     - Sixel graphics");
    println!("  display=ascii     - ASCII art");
    println!("  display=ansi      - ANSI graphics");
    println!();
    println!("OUTPUT OPTIONS:");
    println!("  png=filename      - PNG output");
    println!("  jpg=filename      - JPEG output");
    println!("  csv=filename      - CSV data output");
    println!("  latex=filename    - LaTeX output");
    println!();
    println!("EXAMPLES:");
    println!("  x^2                                    - Simple quadratic");
    println!("  x^2 for x in -10:10                   - Plot x² from -10 to 10");
    println!("  x for x in 0:10 with display=ascii    - ASCII plot");
    println!("  x^2 for x in -5:5 with display=ansi   - Colored plot");
    println!("  a=2; a*x^2 + 1 for x in -5:5          - With constant definition");
    println!();
    println!("DEBUGGING COMMANDS:");
    println!("  test                                   - Run built-in test");
    println!("  debug                                  - Show debug information");
}

fn interactive_mode() {
    println!("Mathematical Expression Parser and Plotter");
    println!("Type expressions or 'help' for usage, 'quit' to exit");
    println!();

    loop {
        print!("> ");
        use std::io::Write;
        std::io::stdout().flush().unwrap();

        let mut input = String::new();
        match stdin().read_line(&mut input) {
            Ok(_) => {
                let input = input.trim();

                if input.is_empty() {
                    continue;
                }

                match input {
                    "quit" | "exit" | "q" => {
                        println!("Goodbye!");
                        break;
                    }
                    "help" | "h" => {
                        print_help();
                        continue;
                    }

                    _ => {
                        execute_command(input);
                    }
                }
            }
            Err(error) => {
                eprintln!("Error reading input: {error}");
                break;
            }
        }
    }
}

fn execute_expression(expression: &str) {
    if !expression.contains("for") && !expression.contains("in") {
        if let Ok(_result) = parse_and_evaluate_simple_expression(expression) {
            //println!("Result: {result}");
            return;
        }
    }

    execute_command(expression);
}

fn execute_command(input: &str) {
    let state = State::new();

    match ECommand.parse(input.chars(), state) {
        Ok((command, _, _)) => {
            //println!("✓ Parsed command successfully");
            if let Err(error) = evaluate_command(&command) {
                eprintln!("✗ Evaluation error: {error:?}");
            }
        }
        Err(error) => {
            eprintln!("✗ Parse error: {error:?}");

            if input.contains("for") && input.contains("in") {
                eprintln!("Hint: Command syntax is: [definitions] expression for var in range [with options]");
            } else if input.chars().any(|c| c.is_alphabetic()) {
                eprintln!("Hint: For simple expressions, try: expression for x in start:end");
                eprintln!("      Or define the variable first: var=value; expression");
            }
        }
    }
}

fn parse_and_evaluate_simple_expression(
    expression: &str,
) -> Result<ExpressionRange1dResult, String> {
    use crate::expression::EExpression;

    let state = State::new();
    let (expr, _, _) = EExpression
        .parse(expression.chars(), state)
        .map_err(|e| format!("Parse error: {e:?}"))?;

    let mut const_map = HashMap::new();
    const_map.insert(
        "x".to_string(),
        ExpressionSyntaxTree::number(
            crate::parser_common::Localization::default(),
            crate::parser_common::Localization::default(),
            1.0,
        ),
    );
    const_map.insert(
        "pi".to_string(),
        ExpressionSyntaxTree::number(
            crate::parser_common::Localization::default(),
            crate::parser_common::Localization::default(),
            std::f64::consts::PI,
        ),
    );
    const_map.insert(
        "e".to_string(),
        ExpressionSyntaxTree::number(
            crate::parser_common::Localization::default(),
            crate::parser_common::Localization::default(),
            std::f64::consts::E,
        ),
    );

    let context = Definition::new(HashMap::new(), const_map);

    DummyExpr::eval(&expr, &context).map_err(|e| format!("Evaluation error: {e:?}"))
}
