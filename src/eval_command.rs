use std::collections::HashMap;

use crate::{
    command::Command,
    command_options::{DisplayOption, OutputOptions},
    display::{
        AnsiRenderer, AsciiRenderer, CsvWriter, DisplayRenderer, LatexWriter, OutputWriter,
        PpmWriter, RegisRenderer, RegisWriter, SixelRenderer, SixelWriter, SvgWriter,
    },
    eval::{Eval, EvaluationError},
    eval_expression,
    eval_range::DummyRange,
    range::Range,
    values::ExpressionRange1dResult,
};

#[derive(Clone, Debug)]
pub struct CommandEvaluator;

impl CommandEvaluator {
    pub fn new() -> Self {
        Self
    }

    fn get_range_variable(&self, range: &Range) -> String {
        match range {
            Range::Numeric(node) => node.value.0.clone(),
            Range::NumericStep(node) => node.value.0.clone(),
            Range::FileBare(node) => node.value.0.clone(),
            Range::FileCol(node) => node.value.0.clone(),
        }
    }

    fn evaluate_expression(
        &self,
        command: &Command,
    ) -> Result<(ExpressionRange1dResult, ExpressionRange1dResult), EvaluationError> {
        let range_result = DummyRange::eval(&command.range, &command.definitions)?;
        let range_var = self.get_range_variable(&command.range);

        let mut env = HashMap::new();
        env.insert(range_var, range_result.clone());

        match eval_expression::eval_with_hashmap(&command.expr, &env, &command.definitions) {
            Some(y_result) => Ok((range_result, y_result)),
            None => Err(EvaluationError::GenericWithString(
                Default::default(),
                Default::default(),
                "Expression evaluation failed".into(),
            )),
        }
    }

    fn handle_display(
        &self,
        command: &Command,
        x_result: &ExpressionRange1dResult,
        y_result: &ExpressionRange1dResult,
    ) {
        if command.options.display.is_empty() {
            let output = AsciiRenderer.render(y_result, 80, 24, x_result);
            println!("{}", output);
        } else {
            for display_option in &command.options.display {
                let output = match display_option {
                    DisplayOption::REGIS(_) => RegisRenderer.render(y_result, 800, 600, x_result),
                    DisplayOption::ASCII(_) => AsciiRenderer.render(y_result, 80, 24, x_result),
                    DisplayOption::ANSI(_) => AnsiRenderer.render(y_result, 80, 24, x_result),
                    DisplayOption::SIXEL(_) => SixelRenderer.render(y_result, 400, 300, x_result),
                };
                println!("{}", output);
            }
        }
    }

    fn handle_output(
        &self,
        command: &Command,
        x_result: &ExpressionRange1dResult,
        y_result: &ExpressionRange1dResult,
    ) {
        for output_option in &command.options.output {
            match output_option {
                OutputOptions::CSV(node) => {
                    if let Err(e) = CsvWriter.write(&node.value, x_result, y_result, 0, 0) {
                        eprintln!("Error saving CSV: {}", e);
                    } else {
                        println!("CSV output saved to {}", node.value);
                    }
                }
                OutputOptions::PPM(node) => {
                    let geom = &node.value.1;
                    if let Err(e) =
                        PpmWriter.write(&node.value.0, x_result, y_result, geom.width, geom.height)
                    {
                        eprintln!("Error saving PPM: {}", e);
                    } else {
                        println!("PPM output saved to {}", node.value.0);
                    }
                }
                OutputOptions::SVG(node) => {
                    let geom = &node.value.1;
                    if let Err(e) =
                        SvgWriter.write(&node.value.0, x_result, y_result, geom.width, geom.height)
                    {
                        eprintln!("Error saving SVG: {}", e);
                    } else {
                        println!("SVG output saved to {}", node.value.0);
                    }
                }
                OutputOptions::LaTeX(node) => {
                    let geom = &node.value.1;
                    if let Err(e) = LatexWriter.write(
                        &node.value.0,
                        x_result,
                        y_result,
                        geom.width,
                        geom.height,
                    ) {
                        eprintln!("Error saving LaTeX: {}", e);
                    } else {
                        println!("LaTeX output saved to {}", node.value.0);
                    }
                }
                OutputOptions::Sixel(node) => {
                    let geom = &node.value.1;
                    if let Err(e) = SixelWriter.write(
                        &node.value.0,
                        x_result,
                        y_result,
                        geom.width,
                        geom.height,
                    ) {
                        eprintln!("Error saving Sixel: {}", e);
                    } else {
                        println!("Sixel output saved to {}", node.value.0);
                    }
                }
                OutputOptions::Regis(node) => {
                    if let Err(e) = RegisWriter.write(&node.value.0, x_result, y_result, 800, 800) {
                        eprintln!("Error saving REGIS: {}", e);
                    } else {
                        println!("REGIS output saved to {}", node.value.0);
                    }
                }
            }
        }
    }
}

impl Eval<Command, (), ()> for CommandEvaluator {
    fn eval(command: &Command, _context: &()) -> Result<(), EvaluationError> {
        let evaluator = CommandEvaluator::new();

        let (x_result, y_result) = evaluator.evaluate_expression(command)?;

        println!("✓ Evaluation successful, {} data points", y_result.0.len());

        evaluator.handle_display(command, &x_result, &y_result);
        evaluator.handle_output(command, &x_result, &y_result);

        Ok(())
    }
}

pub fn evaluate_command(command: &Command) -> Result<(), EvaluationError> {
    CommandEvaluator::eval(command, &())
}
