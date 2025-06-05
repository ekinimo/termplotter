use std::collections::HashMap;
//TODO this needs a overhaul. constants fail atm i.e.
//
//❯ cargo run --release "pi=3.141528; sin(pi*x)^3 for x in 0:10 with display=ansi"
//    Finished `release` profile [optimized] target(s) in 0.02s
//     Running `target/release/termplot 'pi=3.141528; sin(pi*x)^3 for x in 0:10 with display=ansi'`
//✓ Parsed command successfully
//✗ Evaluation error: GenericWithString(Localization { line: 0, column: 0 }, Localization { line: 0, column: 0 }, "Expression evaluation failed")

//
//
use crate::{
    command::{Command, PlotType},
    command_options::{DisplayOption, OutputOptions},
    display::{
        AnsiRenderer, AsciiRenderer, CsvWriter, DisplayRenderer, LatexWriter, OutputWriter,
        PpmWriter, RegisRenderer, RegisWriter, SixelRenderer, SixelWriter, SvgWriter,
    },
    eval::{Eval, EvaluationError},
    eval_expression,
    eval_range::DummyRange,
    range::Range,
    values::{ExpressionRange1dResult, Expression3dResult},
    parametric2d::Parametric2DResult,
};

#[derive(Clone, Debug)]
pub enum PlotResult {
    Plot2D(ExpressionRange1dResult, ExpressionRange1dResult), // (x_data, y_data)
    Parametric2D(Parametric2DResult),
    Surface3D(Expression3dResult),
}

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
    ) -> Result<PlotResult, EvaluationError> {
        let range_result = DummyRange::eval(&command.range, &command.definitions)?;
        let range_var = self.get_range_variable(&command.range);

        let mut env = HashMap::new();
        env.insert(range_var, range_result.clone());

        match &command.plot {
            PlotType::Expression(expr) => {
                match eval_expression::eval_with_hashmap(expr, &env, &command.definitions) {
                    Some(y_result) => Ok(PlotResult::Plot2D(range_result, y_result)),
                    None => Err(EvaluationError::GenericWithString(
                        Default::default(),
                        Default::default(),
                        "Expression evaluation failed".into(),
                    )),
                }
            }
            PlotType::Parametric(parametric) => {
                // For parametric plots, we need to evaluate both x and y expressions with the range parameter
                let x_result = eval_expression::eval_with_hashmap(&parametric.x_expr, &env, &command.definitions);
                let y_result = eval_expression::eval_with_hashmap(&parametric.y_expr, &env, &command.definitions);
                
                match (x_result, y_result) {
                    (Some(x_vals), Some(y_vals)) => {
                        let parametric_result = Parametric2DResult::new(x_vals.0, y_vals.0);
                        Ok(PlotResult::Parametric2D(parametric_result))
                    }
                    _ => Err(EvaluationError::GenericWithString(
                        Default::default(),
                        Default::default(),
                        "Parametric expression evaluation failed".into(),
                    )),
                }
            }
            PlotType::Surface3D(expr, x_var, y_var) => {
                let y_range = if let Some(y_range) = &command.y_range {
                    y_range
                } else {
                    return Err(EvaluationError::GenericWithString(
                        Default::default(),
                        Default::default(),
                        "3D surface requires y_range".into(),
                    ));
                };

                let x_range_result = range_result;
                let y_range_result = DummyRange::eval(y_range, &command.definitions)?;
                
                // Evaluate the 3D surface by creating a meshgrid
                let mut z_data = Vec::new();
                for &y_val in &y_range_result.0 {
                    let mut z_row = Vec::new();
                    for &x_val in &x_range_result.0 {
                        let mut surf_env = HashMap::new();
                        surf_env.insert(x_var.clone(), ExpressionRange1dResult::from(x_val));
                        surf_env.insert(y_var.clone(), ExpressionRange1dResult::from(y_val));
                        
                        match eval_expression::eval_with_hashmap(expr, &surf_env, &command.definitions) {
                            Some(z_result) => {
                                if let Some(&z_val) = z_result.0.first() {
                                    z_row.push(z_val);
                                } else {
                                    return Err(EvaluationError::GenericWithString(
                                        Default::default(),
                                        Default::default(),
                                        "Expression evaluation returned empty result".into(),
                                    ));
                                }
                            }
                            None => {
                                return Err(EvaluationError::GenericWithString(
                                    Default::default(),
                                    Default::default(),
                                    "Expression evaluation failed".into(),
                                ));
                            }
                        }
                    }
                    z_data.push(z_row);
                }
                
                let surface3d_result = Expression3dResult::new(z_data, x_range_result.0, y_range_result.0);
                Ok(PlotResult::Surface3D(surface3d_result))
            }
        }
    }

    fn handle_display(
        &self,
        command: &Command,
        plot_result: &PlotResult,
    ) {
        match plot_result {
            PlotResult::Plot2D(x_result, y_result) => {
                if command.options.display.is_empty() {
                    let output = AsciiRenderer.render(y_result, 80, 24, x_result);
                    println!("{output}");
                } else {
                    for display_option in &command.options.display {
                        let output = match display_option {
                            DisplayOption::REGIS(_) => RegisRenderer.render(y_result, 800, 600, x_result),
                            DisplayOption::ASCII(_) => AsciiRenderer.render(y_result, 80, 24, x_result),
                            DisplayOption::ANSI(_) => AnsiRenderer.render(y_result, 80, 24, x_result),
                            DisplayOption::SIXEL(_) => SixelRenderer.render(y_result, 400, 300, x_result),
                        };
                        println!("{output}");
                    }
                }
            }
            PlotResult::Parametric2D(parametric_result) => {
                if command.options.display.is_empty() {
                    let output = AsciiRenderer.render_parametric(parametric_result, 80, 24);
                    println!("{output}");
                } else {
                    for display_option in &command.options.display {
                        let output = match display_option {
                            DisplayOption::REGIS(_) => RegisRenderer.render_parametric(parametric_result, 800, 600),
                            DisplayOption::ASCII(_) => AsciiRenderer.render_parametric(parametric_result, 80, 24),
                            DisplayOption::ANSI(_) => AnsiRenderer.render_parametric(parametric_result, 80, 24),
                            DisplayOption::SIXEL(_) => SixelRenderer.render_parametric(parametric_result, 400, 300),
                        };
                        println!("{output}");
                    }
                }
            }
            PlotResult::Surface3D(surface3d_result) => {
                if command.options.display.is_empty() {
                    let output = AsciiRenderer.render_surface3d(surface3d_result, 80, 24);
                    println!("{output}");
                } else {
                    for display_option in &command.options.display {
                        let output = match display_option {
                            DisplayOption::REGIS(_) => RegisRenderer.render_surface3d(surface3d_result, 800, 600),
                            DisplayOption::ASCII(_) => AsciiRenderer.render_surface3d(surface3d_result, 80, 24),
                            DisplayOption::ANSI(_) => AnsiRenderer.render_surface3d(surface3d_result, 80, 24),
                            DisplayOption::SIXEL(_) => SixelRenderer.render_surface3d(surface3d_result, 400, 300),
                        };
                        println!("{output}");
                    }
                }
            }
        }
    }

    fn handle_output(
        &self,
        command: &Command,
        plot_result: &PlotResult,
    ) {
        match plot_result {
            PlotResult::Plot2D(x_result, y_result) => {
                for output_option in &command.options.output {
                    match output_option {
                        OutputOptions::CSV(node) => {
                            if let Err(e) = CsvWriter.write(&node.value, x_result, y_result, 0, 0) {
                                eprintln!("Error saving CSV: {e}");
                            } else {
                                println!("CSV output saved to {}", node.value);
                            }
                        }
                        OutputOptions::PPM(node) => {
                            let geom = &node.value.1;
                            if let Err(e) =
                                PpmWriter.write(&node.value.0, x_result, y_result, geom.width, geom.height)
                            {
                                eprintln!("Error saving PPM: {e}");
                            } else {
                                println!("PPM output saved to {}", node.value.0);
                            }
                        }
                        OutputOptions::SVG(node) => {
                            let geom = &node.value.1;
                            if let Err(e) =
                                SvgWriter.write(&node.value.0, x_result, y_result, geom.width, geom.height)
                            {
                                eprintln!("Error saving SVG: {e}");
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
                                eprintln!("Error saving LaTeX: {e}");
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
                                eprintln!("Error saving Sixel: {e}");
                            } else {
                                println!("Sixel output saved to {}", node.value.0);
                            }
                        }
                        OutputOptions::Regis(node) => {
                            if let Err(e) = RegisWriter.write(&node.value.0, x_result, y_result, 800, 800) {
                                eprintln!("Error saving REGIS: {e}");
                            } else {
                                println!("REGIS output saved to {}", node.value.0);
                            }
                        }
                    }
                }
            }
            PlotResult::Parametric2D(parametric_result) => {
                
                for output_option in &command.options.output {
                    match output_option {
                        OutputOptions::CSV(node) => {
                            if let Err(e) = CsvWriter.write_parametric(&node.value, parametric_result, 0, 0) {
                                eprintln!("Error saving CSV: {e}");
                            } else {
                                println!("CSV output saved to {}", node.value);
                            }
                        }
                        OutputOptions::PPM(node) => {
                            let geom = &node.value.1;
                            if let Err(e) =
                                PpmWriter.write_parametric(&node.value.0, parametric_result, geom.width, geom.height)
                            {
                                eprintln!("Error saving PPM: {e}");
                            } else {
                                println!("PPM output saved to {}", node.value.0);
                            }
                        }
                        OutputOptions::SVG(node) => {
                            let geom = &node.value.1;
                            if let Err(e) =
                                SvgWriter.write_parametric(&node.value.0, parametric_result, geom.width, geom.height)
                            {
                                eprintln!("Error saving SVG: {e}");
                            } else {
                                println!("SVG output saved to {}", node.value.0);
                            }
                        }
                        OutputOptions::LaTeX(node) => {
                            let geom = &node.value.1;
                            if let Err(e) = LatexWriter.write_parametric(
                                &node.value.0,
                                parametric_result,
                                geom.width,
                                geom.height,
                            ) {
                                eprintln!("Error saving LaTeX: {e}");
                            } else {
                                println!("LaTeX output saved to {}", node.value.0);
                            }
                        }
                        OutputOptions::Sixel(node) => {
                            let geom = &node.value.1;
                            if let Err(e) = SixelWriter.write_parametric(
                                &node.value.0,
                                parametric_result,
                                geom.width,
                                geom.height,
                            ) {
                                eprintln!("Error saving Sixel: {e}");
                            } else {
                                println!("Sixel output saved to {}", node.value.0);
                            }
                        }
                        OutputOptions::Regis(node) => {
                            if let Err(e) = RegisWriter.write_parametric(&node.value.0, parametric_result, 800, 800) {
                                eprintln!("Error saving REGIS: {e}");
                            } else {
                                println!("REGIS output saved to {}", node.value.0);
                            }
                        }
                    }
                }
            }
            PlotResult::Surface3D(surface3d_result) => {
                for output_option in &command.options.output {
                    match output_option {
                        OutputOptions::CSV(node) => {
                            if let Err(e) = CsvWriter.write_surface3d(&node.value, surface3d_result, 0, 0) {
                                eprintln!("Error saving CSV: {e}");
                            } else {
                                println!("CSV output saved to {}", node.value);
                            }
                        }
                        OutputOptions::PPM(node) => {
                            let geom = &node.value.1;
                            if let Err(e) = PpmWriter.write_surface3d(&node.value.0, surface3d_result, geom.width, geom.height) {
                                eprintln!("Error saving PPM: {e}");
                            } else {
                                println!("PPM output saved to {}", node.value.0);
                            }
                        }
                        OutputOptions::SVG(node) => {
                            let geom = &node.value.1;
                            if let Err(e) = SvgWriter.write_surface3d(&node.value.0, surface3d_result, geom.width, geom.height) {
                                eprintln!("Error saving SVG: {e}");
                            } else {
                                println!("SVG output saved to {}", node.value.0);
                            }
                        }
                        OutputOptions::LaTeX(node) => {
                            let geom = &node.value.1;
                            if let Err(e) = LatexWriter.write_surface3d(&node.value.0, surface3d_result, geom.width, geom.height) {
                                eprintln!("Error saving LaTeX: {e}");
                            } else {
                                println!("LaTeX output saved to {}", node.value.0);
                            }
                        }
                        OutputOptions::Sixel(node) => {
                            let geom = &node.value.1;
                            if let Err(e) = SixelWriter.write_surface3d(&node.value.0, surface3d_result, geom.width, geom.height) {
                                eprintln!("Error saving Sixel: {e}");
                            } else {
                                println!("Sixel output saved to {}", node.value.0);
                            }
                        }
                        OutputOptions::Regis(node) => {
                            if let Err(e) = RegisWriter.write_surface3d(&node.value.0, surface3d_result, 800, 800) {
                                eprintln!("Error saving REGIS: {e}");
                            } else {
                                println!("REGIS output saved to {}", node.value.0);
                            }
                        }
                    }
                }
            }
        }
    }
}

impl Eval<Command, (), ()> for CommandEvaluator {
    fn eval(command: &Command, _context: &()) -> Result<(), EvaluationError> {
        let evaluator = CommandEvaluator::new();

        let plot_result = evaluator.evaluate_expression(command)?;
        evaluator.handle_display(command, &plot_result);
        evaluator.handle_output(command, &plot_result);

        Ok(())
    }
}

pub fn evaluate_command(command: &Command) -> Result<(), EvaluationError> {
    CommandEvaluator::eval(command, &())
}
