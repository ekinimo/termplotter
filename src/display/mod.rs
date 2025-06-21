use std::error::Error;

use crate::{
    values::{ExpressionRange1dResult, Expression3dResult},
    parametric2d::Parametric2DResult,
};

pub mod renderers;
pub mod writers;
pub mod utils;

// Re-export all the public types for convenience
pub use renderers::{AsciiRenderer, AnsiRenderer, RegisRenderer, SixelRenderer};
pub use writers::{CsvWriter, PpmWriter, SvgWriter, LatexWriter, SixelWriter, RegisWriter};
pub use utils::{Bitmap, Point3D, SurfaceBounds};

#[derive(Clone, Copy, Debug)]
pub struct PlotConfig {
    pub width: usize,
    #[allow(dead_code)]
    pub height: usize,
    pub data_points: usize,
    pub x_min: f64,
    pub x_max: f64,
    pub y_min: f64,
    pub y_max: f64,
}

#[derive(Clone, Copy, Debug)]
pub struct Plot3DConfig {
    pub width: usize,
    #[allow(dead_code)]
    pub height: usize,
    pub data_points: usize,
    pub x_min: f64,
    pub x_max: f64,
    pub y_min: f64,
    pub y_max: f64,
    pub z_min: f64,
    pub z_max: f64,
}

impl PlotConfig {
    pub fn new(width: usize, height: usize, data_points: usize, x_min: f64, x_max: f64, y_min: f64, y_max: f64) -> Self {
        Self { width, height, data_points, x_min, x_max, y_min, y_max }
    }
}

impl Plot3DConfig {
    pub fn new(size: (usize, usize), data_points: usize, x_range: (f64, f64), y_range: (f64, f64), z_range: (f64, f64)) -> Self {
        Self { 
            width: size.0, height: size.1, data_points, 
            x_min: x_range.0, x_max: x_range.1, 
            y_min: y_range.0, y_max: y_range.1,
            z_min: z_range.0, z_max: z_range.1
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct PlotBounds {
    pub x_min: f64,
    pub x_max: f64,
    pub y_min: f64,
    pub y_max: f64,
}

impl PlotBounds {
    pub fn new(x_min: f64, x_max: f64, y_min: f64, y_max: f64) -> Self {
        Self { x_min, x_max, y_min, y_max }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Plot3DStyle {
    pub plot_width: usize,
    pub plot_height: usize,
    pub axis_char: char,
    pub axis_color: u8,
}

impl Plot3DStyle {
    pub fn new(plot_width: usize, plot_height: usize, axis_char: char, axis_color: u8) -> Self {
        Self { plot_width, plot_height, axis_char, axis_color }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct LabelConfig {
    pub margin: usize,
    pub plot_width: usize,
    pub plot_height: usize,
    pub x_min: f64,
    pub x_max: f64,
    pub y_min: f64,
    pub y_max: f64,
}

impl LabelConfig {
    pub fn new(margin: usize, plot_width: usize, plot_height: usize, x_min: f64, x_max: f64, y_min: f64, y_max: f64) -> Self {
        Self { margin, plot_width, plot_height, x_min, x_max, y_min, y_max }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Label3DConfig {
    pub margin: usize,
    pub plot_width: usize,
    pub plot_height: usize,
    pub x_min: f64,
    pub x_max: f64,
    pub y_min: f64,
    pub y_max: f64,
    pub z_min: f64,
    pub z_max: f64,
}

impl Label3DConfig {
    pub fn new(layout: (usize, usize, usize), x_range: (f64, f64), y_range: (f64, f64), z_range: (f64, f64)) -> Self {
        Self { 
            margin: layout.0, plot_width: layout.1, plot_height: layout.2,
            x_min: x_range.0, x_max: x_range.1,
            y_min: y_range.0, y_max: y_range.1,
            z_min: z_range.0, z_max: z_range.1
        }
    }
}

/// Trait for rendering data to terminal/display formats
pub trait DisplayRenderer {
    fn render(
        &self,
        result: &ExpressionRange1dResult,
        width: usize,
        height: usize,
        x_range: &ExpressionRange1dResult,
    ) -> String;

    fn render_parametric(
        &self,
        result: &Parametric2DResult,
        width: usize,
        height: usize,
    ) -> String;

    fn render_surface3d(
        &self,
        result: &Expression3dResult,
        width: usize,
        height: usize,
    ) -> String;
}

/// Trait for writing data to file formats
pub trait OutputWriter {
    fn write(
        &self,
        filename: &str,
        x_result: &ExpressionRange1dResult,
        y_result: &ExpressionRange1dResult,
        width: usize,
        height: usize,
    ) -> Result<(), Box<dyn Error>>;

    fn write_parametric(
        &self,
        filename: &str,
        result: &Parametric2DResult,
        width: usize,
        height: usize,
    ) -> Result<(), Box<dyn Error>>;

    fn write_surface3d(
        &self,
        filename: &str,
        result: &Expression3dResult,
        width: usize,
        height: usize,
    ) -> Result<(), Box<dyn Error>>;
}