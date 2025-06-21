use std::{error::Error, io::Write};

use crate::{
    values::{ExpressionRange1dResult, Expression3dResult},
    parametric2d::Parametric2DResult,
};

use super::super::{OutputWriter, SixelRenderer, DisplayRenderer};

#[derive(Clone, Debug)]
pub struct SixelWriter;

impl OutputWriter for SixelWriter {
    fn write(
        &self,
        filename: &str,
        x_result: &ExpressionRange1dResult,
        y_result: &ExpressionRange1dResult,
        width: usize,
        height: usize,
    ) -> Result<(), Box<dyn Error>> {
        let mut file = std::fs::File::create(filename)?;
        let sixel_output = SixelRenderer.render(y_result, width, height, x_result);
        write!(file, "{sixel_output}")?;
        Ok(())
    }

    fn write_parametric(
        &self,
        filename: &str,
        result: &Parametric2DResult,
        width: usize,
        height: usize,
    ) -> Result<(), Box<dyn Error>> {
        let mut file = std::fs::File::create(filename)?;
        let sixel_output = SixelRenderer.render_parametric(result, width, height);
        write!(file, "{sixel_output}")?;
        Ok(())
    }

    fn write_surface3d(
        &self,
        filename: &str,
        result: &Expression3dResult,
        width: usize,
        height: usize,
    ) -> Result<(), Box<dyn Error>> {
        let mut file = std::fs::File::create(filename)?;
        let sixel_output = SixelRenderer.render_surface3d(result, width, height);
        write!(file, "{sixel_output}")?;
        Ok(())
    }
}