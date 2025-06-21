use std::{error::Error, io::Write};

use crate::{
    values::{ExpressionRange1dResult, Expression3dResult},
    parametric2d::Parametric2DResult,
};

use super::super::{OutputWriter, Bitmap};

#[derive(Clone, Debug)]
pub struct PpmWriter;

impl OutputWriter for PpmWriter {
    fn write(
        &self,
        filename: &str,
        x_result: &ExpressionRange1dResult,
        y_result: &ExpressionRange1dResult,
        width: usize,
        height: usize,
    ) -> Result<(), Box<dyn Error>> {
        let margin = 50;
        let total_width = width + 2 * margin;
        let total_height = height + 2 * margin;

        let mut bitmap = Bitmap::new(total_width, total_height, 0);
        bitmap.create_plot(y_result, x_result, margin);

        let mut file = std::fs::File::create(filename)?;
        writeln!(file, "P3\n{total_width} {total_height}\n255")?;

        for y in 0..total_height {
            for x in 0..total_width {
                let (r, g, b) = match bitmap.get_pixel(x, y) {
                    0 => (0, 0, 0),
                    1 => (0, 255, 255),
                    2 => (64, 64, 64),
                    3 => (255, 255, 0),
                    4 => (192, 192, 192),
                    _ => (255, 0, 0),
                };
                write!(file, "{r} {g} {b} ")?;
            }
            writeln!(file)?;
        }
        Ok(())
    }

    fn write_parametric(
        &self,
        filename: &str,
        result: &Parametric2DResult,
        width: usize,
        height: usize,
    ) -> Result<(), Box<dyn Error>> {
        let margin = 50;
        let total_width = width + 2 * margin;
        let total_height = height + 2 * margin;

        let mut bitmap = Bitmap::new(total_width, total_height, 0);
        bitmap.create_parametric_plot(result, margin);

        let mut file = std::fs::File::create(filename)?;
        writeln!(file, "P3\n{total_width} {total_height}\n255")?;

        for y in 0..total_height {
            for x in 0..total_width {
                let (r, g, b) = match bitmap.get_pixel(x, y) {
                    0 => (0, 0, 0),
                    1 => (0, 255, 255),
                    2 => (64, 64, 64),
                    3 => (255, 255, 0),
                    4 => (192, 192, 192),
                    _ => (255, 0, 0),
                };
                write!(file, "{r} {g} {b} ")?;
            }
            writeln!(file)?;
        }
        Ok(())
    }

    fn write_surface3d(
        &self,
        filename: &str,
        result: &Expression3dResult,
        width: usize,
        height: usize,
    ) -> Result<(), Box<dyn Error>> {
        let margin = 50;
        let total_width = width + 2 * margin;
        let total_height = height + 2 * margin;

        let mut bitmap = Bitmap::new(total_width, total_height, 0);
        bitmap.create_surface3d_plot(result, margin);

        let mut file = std::fs::File::create(filename)?;
        writeln!(file, "P3\n{total_width} {total_height}\n255")?;

        for y in 0..total_height {
            for x in 0..total_width {
                let (r, g, b) = match bitmap.get_pixel(x, y) {
                    0 => (0, 0, 0),
                    1 => (0, 255, 255),
                    2 => (64, 64, 64),
                    3 => (255, 255, 0),
                    4 => (192, 192, 192),
                    _ => (255, 0, 0),
                };
                write!(file, "{r} {g} {b} ")?;
            }
            writeln!(file)?;
        }
        Ok(())
    }
}