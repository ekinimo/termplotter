use std::{error::Error, io::Write};

use crate::{
    values::{ExpressionRange1dResult, Expression3dResult},
    parametric2d::Parametric2DResult,
};

use super::super::{OutputWriter, RegisRenderer, DisplayRenderer};

#[derive(Clone, Debug)]
pub struct RegisWriter;

impl OutputWriter for RegisWriter {
    fn write(
        &self,
        filename: &str,
        x_result: &ExpressionRange1dResult,
        y_result: &ExpressionRange1dResult,
        width: usize,
        height: usize,
    ) -> Result<(), Box<dyn Error>> {
        let mut file = std::fs::File::create(filename)?;
        let regis_output = RegisRenderer.render(y_result, width, height, x_result);
        write!(file, "{regis_output}")?;
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
        let regis_output = RegisRenderer.render_parametric(result, width, height);
        write!(file, "{regis_output}")?;
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
        let regis_output = RegisRenderer.render_surface3d(result, width, height);
        write!(file, "{regis_output}")?;
        Ok(())
    }
}