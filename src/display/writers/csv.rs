use std::{error::Error, io::Write};

use crate::{
    values::{ExpressionRange1dResult, Expression3dResult},
    parametric2d::Parametric2DResult,
};

use super::super::OutputWriter;

#[derive(Clone, Debug)]
pub struct CsvWriter;

impl OutputWriter for CsvWriter {
    fn write(
        &self,
        filename: &str,
        x_result: &ExpressionRange1dResult,
        y_result: &ExpressionRange1dResult,
        _width: usize,
        _height: usize,
    ) -> Result<(), Box<dyn Error>> {
        let mut file = std::fs::File::create(filename)?;
        writeln!(file, "x,y")?;
        for (&x_val, &y_val) in x_result.0.iter().zip(y_result.0.iter()) {
            writeln!(file, "{x_val},{y_val}")?;
        }
        Ok(())
    }

    fn write_parametric(
        &self,
        filename: &str,
        result: &Parametric2DResult,
        _width: usize,
        _height: usize,
    ) -> Result<(), Box<dyn Error>> {
        let mut file = std::fs::File::create(filename)?;
        writeln!(file, "x,y")?;
        for (&x_val, &y_val) in result.x_values.iter().zip(result.y_values.iter()) {
            writeln!(file, "{x_val},{y_val}")?;
        }
        Ok(())
    }

    fn write_surface3d(
        &self,
        filename: &str,
        result: &Expression3dResult,
        _width: usize,
        _height: usize,
    ) -> Result<(), Box<dyn Error>> {
        let mut file = std::fs::File::create(filename)?;
        writeln!(file, "x,y,z")?;
        for (y_idx, y_val) in result.y_values.iter().enumerate() {
            for (x_idx, x_val) in result.x_values.iter().enumerate() {
                if let Some(z_val) = result.get_z(x_idx, y_idx) {
                    writeln!(file, "{x_val},{y_val},{z_val}")?;
                }
            }
        }
        Ok(())
    }
}