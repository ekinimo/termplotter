use std::{error::Error, io::Write};

use crate::{
    command::Command,
    command_options::{DisplayOption, OutputOptions},
    values::{ExpressionRange1dResult, Expression3dResult},
    parametric2d::Parametric2DResult,
};

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

#[derive(Clone, Debug)]
pub struct Point3D {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Point3D {
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }


    pub fn to_isometric(&self, width: usize, height: usize, bounds: &SurfaceBounds) -> (usize, usize) {
        let norm_x = (self.x - bounds.x_min) / (bounds.x_max - bounds.x_min);
        let norm_y = (self.y - bounds.y_min) / (bounds.y_max - bounds.y_min);
        let norm_z = if bounds.z_max != bounds.z_min {
            (self.z - bounds.z_min) / (bounds.z_max - bounds.z_min)
        } else {
            0.5
        };

        let iso_x = (norm_x - norm_y) * 0.866; // cos(30°) ≈ 0.866
        let iso_y = (norm_x + norm_y) * 0.5 - norm_z; // sin(30°) = 0.5

        let margin = 0.1;
        let screen_x = ((iso_x + 1.0) * 0.5 * (1.0 - 2.0 * margin) + margin) * width as f64;
        let screen_y = ((1.0 - (iso_y + 1.0) * 0.5) * (1.0 - 2.0 * margin) + margin) * height as f64;

        (
            screen_x.max(0.0).min(width as f64 - 1.0) as usize,
            screen_y.max(0.0).min(height as f64 - 1.0) as usize,
        )
    }
}

#[derive(Clone, Debug)]
pub struct SurfaceBounds {
    pub x_min: f64,
    pub x_max: f64,
    pub y_min: f64,
    pub y_max: f64,
    pub z_min: f64,
    pub z_max: f64,
}

impl SurfaceBounds {
    pub fn from_surface(result: &Expression3dResult) -> Self {
        Self {
            x_min: result.x_min(),
            x_max: result.x_max(),
            y_min: result.y_min(),
            y_max: result.y_max(),
            z_min: result.z_min(),
            z_max: result.z_max(),
        }
    }
}


#[derive(Clone, Debug)]
pub struct Bitmap {
    pub data: Vec<Vec<u8>>,
    pub width: usize,
    pub height: usize,
}

impl Bitmap {
    pub fn new(width: usize, height: usize, background: u8) -> Self {
        Self {
            data: vec![vec![background; width]; height],
            width,
            height,
        }
    }

    pub fn set_pixel(&mut self, x: usize, y: usize, color: u8) {
        if x < self.width && y < self.height {
            self.data[y][x] = color;
        }
    }

    pub fn get_pixel(&self, x: usize, y: usize) -> u8 {
        if x < self.width && y < self.height {
            self.data[y][x]
        } else {
            0
        }
    }

    pub fn create_plot(
        &mut self,
        y_result: &ExpressionRange1dResult,
        x_range: &ExpressionRange1dResult,
        margin: usize,
    ) {
        if y_result.0.is_empty() {
            return;
        }

        let plot_width = self.width.saturating_sub(2 * margin);
        let plot_height = self.height.saturating_sub(2 * margin);

        if plot_width == 0 || plot_height == 0 {
            return;
        }

        let min_val = y_result.min();
        let max_val = y_result.max();
        let x_min = x_range.min();
        let x_max = x_range.max();
        let y_range = max_val - min_val;

        if y_range.abs() < f64::EPSILON {
            // Constant value - horizontal line
            let y_pos = margin + plot_height / 2;
            for x in margin..margin + plot_width {
                self.set_pixel(x, y_pos, 1);
            }
        } else {
            // Plot points and lines
            let mut points = Vec::new();
            for (i, &y_val) in y_result.0.iter().enumerate() {
                let x_pos = if y_result.0.len() > 1 {
                    margin + (i * plot_width) / (y_result.0.len() - 1)
                } else {
                    margin + plot_width / 2
                };

                let normalized_y = (y_val - min_val) / y_range;
                let y_pos = margin + ((1.0 - normalized_y) * plot_height as f64) as usize;
                let y_pos = y_pos.min(margin + plot_height - 1);

                points.push((x_pos, y_pos));
            }

            for i in 1..points.len() {
                self.draw_line(
                    points[i - 1].0,
                    points[i - 1].1,
                    points[i].0,
                    points[i].1,
                    1,
                );
            }

            // Draw points
            for &(x, y) in &points {
                self.set_pixel(x, y, 1);
                for dx in 0..=1 {
                    for dy in 0..=1 {
                        self.set_pixel(x.saturating_add(dx), y.saturating_add(dy), 1);
                        self.set_pixel(x.saturating_sub(dx), y.saturating_sub(dy), 1);
                    }
                }
            }
        }

        self.draw_axes_and_grid(margin, plot_width, plot_height);

        self.add_value_labels(
            margin,
            plot_width,
            plot_height,
            min_val,
            max_val,
            x_min,
            x_max,
        );
    }

    pub fn create_parametric_plot(
        &mut self,
        parametric_result: &Parametric2DResult,
        margin: usize,
    ) {
        if parametric_result.is_empty() {
            return;
        }

        let plot_width = self.width.saturating_sub(2 * margin);
        let plot_height = self.height.saturating_sub(2 * margin);

        if plot_width == 0 || plot_height == 0 {
            return;
        }

        let x_min = parametric_result.x_values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let x_max = parametric_result.x_values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        let y_min = parametric_result.y_values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let y_max = parametric_result.y_values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));

        let x_range = x_max - x_min;
        let y_range = y_max - y_min;

        if x_range.abs() < f64::EPSILON || y_range.abs() < f64::EPSILON {
            return;
        }

        let mut points = Vec::new();
        for (&x_val, &y_val) in parametric_result.x_values.iter().zip(parametric_result.y_values.iter()) {
            let x_pos = margin + ((x_val - x_min) / x_range * plot_width as f64) as usize;
            let y_pos = margin + ((y_max - y_val) / y_range * plot_height as f64) as usize;
            let x_pos = x_pos.min(margin + plot_width - 1);
            let y_pos = y_pos.min(margin + plot_height - 1);
            points.push((x_pos, y_pos));
        }

        // Draw lines connecting parametric points
        for i in 1..points.len() {
            self.draw_line(
                points[i - 1].0,
                points[i - 1].1,
                points[i].0,
                points[i].1,
                1,
            );
        }

        // Draw points
        for &(x, y) in &points {
            self.set_pixel(x, y, 1);
            for dx in 0..=1 {
                for dy in 0..=1 {
                    self.set_pixel(x.saturating_add(dx), y.saturating_add(dy), 1);
                    self.set_pixel(x.saturating_sub(dx), y.saturating_sub(dy), 1);
                }
            }
        }

        self.draw_axes_and_grid(margin, plot_width, plot_height);

        self.add_parametric_value_labels(
            margin,
            plot_width,
            plot_height,
            x_min,
            x_max,
            y_min,
            y_max,
        );
    }

    pub fn create_surface3d_plot(
        &mut self,
        surface_result: &Expression3dResult,
        margin: usize,
    ) {
                if surface_result.is_empty() {
            return;
        }

        let plot_width = self.width.saturating_sub(2 * margin);
        let plot_height = self.height.saturating_sub(2 * margin);

        if plot_width == 0 || plot_height == 0 {
            return;
        }

        let bounds = SurfaceBounds::from_surface(surface_result);

        // Draw wireframe mesh
        self.draw_wireframe_mesh(surface_result, &bounds, margin, plot_width, plot_height);

        // Draw surface points
        self.draw_surface_points(surface_result, &bounds, margin, plot_width, plot_height);

        // Draw axes
        self.draw_3d_axes(&bounds, margin, plot_width, plot_height);

        // Add labels
        self.add_isometric_labels(&bounds, margin, plot_width, plot_height);

    }

        fn draw_wireframe_mesh(
        &mut self,
        surface_result: &Expression3dResult,
        bounds: &SurfaceBounds,
        margin: usize,
        plot_width: usize,
        plot_height: usize,
    ) {
        let x_len = surface_result.x_len();
        let y_len = surface_result.y_len();

        // Draw grid lines in X direction
        for y_idx in 0..y_len {
            let mut prev_point = None;
            for x_idx in 0..x_len {
                if let Some(z_val) = surface_result.get_z(x_idx, y_idx) {
                    let point3d = Point3D::new(
                        surface_result.x_values[x_idx],
                        surface_result.y_values[y_idx],
                        z_val,
                    );
                    let (screen_x, screen_y) = point3d.to_isometric(plot_width, plot_height, bounds);
                    let screen_x = screen_x + margin;
                    let screen_y = screen_y + margin;

                    if let Some((prev_x, prev_y)) = prev_point {
                        self.draw_line(prev_x, prev_y, screen_x, screen_y, 2); // Grid color
                    }
                    prev_point = Some((screen_x, screen_y));
                }
            }
        }

        // Draw grid lines in Y direction
        for x_idx in 0..x_len {
            let mut prev_point = None;
            for y_idx in 0..y_len {
                if let Some(z_val) = surface_result.get_z(x_idx, y_idx) {
                    let point3d = Point3D::new(
                        surface_result.x_values[x_idx],
                        surface_result.y_values[y_idx],
                        z_val,
                    );
                    let (screen_x, screen_y) = point3d.to_isometric(plot_width, plot_height, bounds);
                    let screen_x = screen_x + margin;
                    let screen_y = screen_y + margin;

                    if let Some((prev_x, prev_y)) = prev_point {
                        self.draw_line(prev_x, prev_y, screen_x, screen_y, 2); // Grid color
                    }
                    prev_point = Some((screen_x, screen_y));
                }
            }
        }
    }

    fn draw_surface_points(
        &mut self,
        surface_result: &Expression3dResult,
        bounds: &SurfaceBounds,
        margin: usize,
        plot_width: usize,
        plot_height: usize,
    ) {
        let z_range = bounds.z_max - bounds.z_min;

        for (y_idx, &y_val) in surface_result.y_values.iter().enumerate() {
            for (x_idx, &x_val) in surface_result.x_values.iter().enumerate() {
                if let Some(z_val) = surface_result.get_z(x_idx, y_idx) {
                    let point3d = Point3D::new(x_val, y_val, z_val);
                    let (screen_x, screen_y) = point3d.to_isometric(plot_width, plot_height, bounds);
                    let screen_x = screen_x + margin;
                    let screen_y = screen_y + margin;

                    // Color based on height (Z value)
                    let color = if z_range > f64::EPSILON {
                        let normalized_z = (z_val - bounds.z_min) / z_range;
                        if normalized_z < 0.25 { 4 }      // Low
                        else if normalized_z < 0.5 { 3 }  // Medium-low
                        else if normalized_z < 0.75 { 1 } // Medium-high
                        else { 5 }                        // High
                    } else {
                        1
                    };

                    // Draw a small cross or dot for each point
                    self.set_pixel(screen_x, screen_y, color);
                    if screen_x > 0 { self.set_pixel(screen_x - 1, screen_y, color); }
                    if screen_x < self.width - 1 { self.set_pixel(screen_x + 1, screen_y, color); }
                    if screen_y > 0 { self.set_pixel(screen_x, screen_y - 1, color); }
                    if screen_y < self.height - 1 { self.set_pixel(screen_x, screen_y + 1, color); }
                }
            }
        }
    }

    fn draw_3d_axes(
        &mut self,
        bounds: &SurfaceBounds,
        margin: usize,
        plot_width: usize,
        plot_height: usize,
    ) {
        // Origin point
        let origin = Point3D::new(bounds.x_min, bounds.y_min, bounds.z_min);
        let (origin_x, origin_y) = origin.to_isometric(plot_width, plot_height, bounds);
        let origin_x = origin_x + margin;
        let origin_y = origin_y + margin;

        // X-axis
        let x_end = Point3D::new(bounds.x_max, bounds.y_min, bounds.z_min);
        let (x_end_x, x_end_y) = x_end.to_isometric(plot_width, plot_height, bounds);
        let x_end_x = x_end_x + margin;
        let x_end_y = x_end_y + margin;
        self.draw_line(origin_x, origin_y, x_end_x, x_end_y, 3); // Yellow for X

        // Y-axis
        let y_end = Point3D::new(bounds.x_min, bounds.y_max, bounds.z_min);
        let (y_end_x, y_end_y) = y_end.to_isometric(plot_width, plot_height, bounds);
        let y_end_x = y_end_x + margin;
        let y_end_y = y_end_y + margin;
        self.draw_line(origin_x, origin_y, y_end_x, y_end_y, 3); // Yellow for Y

        // Z-axis
        let z_end = Point3D::new(bounds.x_min, bounds.y_min, bounds.z_max);
        let (z_end_x, z_end_y) = z_end.to_isometric(plot_width, plot_height, bounds);
        let z_end_x = z_end_x + margin;
        let z_end_y = z_end_y + margin;
        self.draw_line(origin_x, origin_y, z_end_x, z_end_y, 3); // Yellow for Z

        // Draw axis labels
        self.render_text("X", x_end_x + 5, x_end_y, 3);
        self.render_text("Y", y_end_x + 5, y_end_y, 3);
        self.render_text("Z", z_end_x + 5, z_end_y, 3);
    }

    fn add_isometric_labels(
        &mut self,
        bounds: &SurfaceBounds,
        margin: usize,
        _plot_width: usize,
        _plot_height: usize,
    ) {
        // Add corner labels with ranges
        let x_text = format!("X:[{:.1},{:.1}]", bounds.x_min, bounds.x_max);
        let y_text = format!("Y:[{:.1},{:.1}]", bounds.y_min, bounds.y_max);
        let z_text = format!("Z:[{:.1},{:.1}]", bounds.z_min, bounds.z_max);

        self.render_text(&x_text, margin, margin - 20, 4);
        self.render_text(&y_text, margin, margin - 10, 4);
        self.render_text(&z_text, margin, margin, 4);

        // Add title
        self.render_text("Isometric 3D Surface", margin, margin - 35, 4);
    }

    fn add_surface3d_value_labels(
        &mut self,
        margin: usize,
        plot_width: usize,
        plot_height: usize,
        x_min: f64,
        x_max: f64,
        y_min: f64,
        y_max: f64,
        z_min: f64,
        z_max: f64,
    ) {
        // X-axis labels
        for i in 0..=5 {
            let x = margin + (i * plot_width) / 5;
            let value = x_min + (i as f64 / 5.0) * (x_max - x_min);
            let text = format!("{:.1}", value);
            let text_x = x.saturating_sub(text.len() * 3);
            self.render_text(&text, text_x, margin + plot_height + 5, 4);
        }

        // Y-axis labels
        for i in 0..=5 {
            let y = margin + (i * plot_height) / 5;
            let value = y_max - (i as f64 / 5.0) * (y_max - y_min);
            let text = format!("{:.1}", value);
            self.render_text(&text, margin.saturating_sub(40), y.saturating_sub(3), 4);
        }

        // Z-range info
        let z_text = format!("Z:[{:.1}, {:.1}]", z_min, z_max);
        self.render_text(&z_text, margin + 5, margin - 15, 4);
    }

    fn add_parametric_value_labels(
        &mut self,
        margin: usize,
        plot_width: usize,
        plot_height: usize,
        x_min: f64,
        x_max: f64,
        y_min: f64,
        y_max: f64,
    ) {
        for i in 0..=5 {
            let y = margin + (i * plot_height) / 5;
            let value = y_max - (i as f64 / 5.0) * (y_max - y_min);
            let text = format!("{:.1}", value);
            self.render_text(&text, margin.saturating_sub(40), y.saturating_sub(3), 4);
        }

        for i in 0..=5 {
            let x = margin + (i * plot_width) / 5;
            let value = x_min + (i as f64 / 5.0) * (x_max - x_min);
            let text = format!("{:.1}", value);
            let text_x = x.saturating_sub(text.len() * 3);
            self.render_text(&text, text_x, margin + plot_height + 5, 4);
        }
    }

    fn draw_line(&mut self, x0: usize, y0: usize, x1: usize, y1: usize, color: u8) {
        let dx = if x1 > x0 { x1 - x0 } else { x0 - x1 };
        let dy = if y1 > y0 { y1 - y0 } else { y0 - y1 };
        let x_step = if x0 < x1 { 1i32 } else { -1i32 };
        let y_step = if y0 < y1 { 1i32 } else { -1i32 };

        let mut error = dx as i32 - dy as i32;
        let mut x = x0 as i32;
        let mut y = y0 as i32;

        loop {
            self.set_pixel(x as usize, y as usize, color);

            if x == x1 as i32 && y == y1 as i32 {
                break;
            }

            let e2 = 2 * error;
            if e2 > -(dy as i32) {
                error -= dy as i32;
                x += x_step;
            }
            if e2 < dx as i32 {
                error += dx as i32;
                y += y_step;
            }
        }
    }

    fn draw_axes_and_grid(&mut self, margin: usize, plot_width: usize, plot_height: usize) {
        let x_axis_y = margin + plot_height - 1;
        for x in margin..margin + plot_width {
            self.set_pixel(x, x_axis_y, 3);
        }

        // Y-axis
        for y in margin..margin + plot_height {
            self.set_pixel(margin, y, 3);
        }

        // Grid lines
        for i in 1..10 {
            let x = margin + (i * plot_width) / 10;
            for y in margin..margin + plot_height {
                if self.get_pixel(x, y) == 0 {
                    self.set_pixel(x, y, 2);
                }
            }
        }

        for i in 1..8 {
            let y = margin + (i * plot_height) / 8;
            for x in margin..margin + plot_width {
                if self.get_pixel(x, y) == 0 {
                    self.set_pixel(x, y, 2);
                }
            }
        }
    }

    fn add_value_labels(
        &mut self,
        margin: usize,
        plot_width: usize,
        plot_height: usize,
        min_val: f64,
        max_val: f64,
        x_min: f64,
        x_max: f64,
    ) {
        for i in 0..=5 {
            let y = margin + (i * plot_height) / 5;
            let value = max_val - (i as f64 / 5.0) * (max_val - min_val);
            let text = format!("{:.1}", value);
            self.render_text(&text, margin.saturating_sub(40), y.saturating_sub(3), 4);
        }

        for i in 0..=5 {
            let x = margin + (i * plot_width) / 5;
            let value = x_min + (i as f64 / 5.0) * (x_max - x_min);
            let text = format!("{:.1}", value);
            let text_x = x.saturating_sub(text.len() * 3);
            self.render_text(&text, text_x, margin + plot_height + 5, 4);
        }
    }

    fn render_text(&mut self, text: &str, start_x: usize, start_y: usize, color: u8) {
        let mut x_offset = 0;
        for ch in text.chars() {
            let bitmap = get_char_bitmap(ch);
            for (row, bitmap_row) in bitmap.iter().enumerate() {
                for (col, &pixel) in bitmap_row.iter().enumerate() {
                    if pixel {
                        let y = start_y + row;
                        let x = start_x + x_offset + col;
                        if y < self.height && x < self.width {
                            self.set_pixel(x, y, color);
                        }
                    }
                }
            }
            x_offset += 6; // char width + spacing
        }
    }
}

#[derive(Clone, Debug)]
pub struct AsciiRenderer;

impl DisplayRenderer for AsciiRenderer {
    fn render(
        &self,
        result: &ExpressionRange1dResult,
        width: usize,
        height: usize,
        x_range: &ExpressionRange1dResult,
    ) -> String {
        if result.0.is_empty() {
            return "No data to plot".to_string();
        }

        let min_val = result.min();
        let max_val = result.max();
        let x_min = x_range.min();
        let x_max = x_range.max();

        if (max_val - min_val).abs() < f64::EPSILON {
            return format!(
                "Constant value: {:.3} (all {} points)\nX range: [{:.2}, {:.2}]\n{}",
                min_val,
                result.0.len(),
                x_min,
                x_max,
                " ".repeat(width / 2) + &"*".repeat(result.0.len().min(width))
            );
        }

        let mut grid = vec![vec![' '; width]; height];
        let padding = (max_val - min_val) * 0.1;
        let plot_min = min_val - padding;
        let plot_max = max_val + padding;
        let plot_range = plot_max - plot_min;

        // Plot data points
        let data_width = width.saturating_sub(6);
        for (i, &value) in result.0.iter().enumerate() {
            let x_pos = if result.0.len() > 1 {
                5 + (i * data_width / (result.0.len() - 1))
            } else {
                width / 2
            };

            if x_pos < width {
                let normalized_y = (value - plot_min) / plot_range;
                let y_pos = ((1.0 - normalized_y) * (height - 1) as f64) as usize;
                let y_pos = y_pos.min(height - 1);
                grid[y_pos][x_pos] = '*';
            }
        }

        // Add axes and labels
        add_ascii_axes(&mut grid, width, height, plot_min, plot_max);
        format_ascii_output(
            grid,
            width,
            result.0.len(),
            x_min,
            x_max,
            min_val,
            max_val,
            x_range,
        )
    }

    fn render_parametric(
        &self,
        result: &Parametric2DResult,
        width: usize,
        height: usize,
    ) -> String {
        if result.is_empty() {
            return "No parametric data to plot".to_string();
        }

        let x_min = result.x_values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let x_max = result.x_values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        let y_min = result.y_values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let y_max = result.y_values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));

        let x_range = x_max - x_min;
        let y_range = y_max - y_min;

        if x_range.abs() < f64::EPSILON || y_range.abs() < f64::EPSILON {
            return format!(
                "Parametric plot: X=[{:.3}, {:.3}], Y=[{:.3}, {:.3}] (constant values)",
                x_min, x_max, y_min, y_max
            );
        }

        let mut grid = vec![vec![' '; width]; height];
        let padding_x = x_range * 0.1;
        let padding_y = y_range * 0.1;
        let plot_x_min = x_min - padding_x;
        let plot_x_max = x_max + padding_x;
        let plot_y_min = y_min - padding_y;
        let plot_y_max = y_max + padding_y;
        let plot_x_range = plot_x_max - plot_x_min;
        let plot_y_range = plot_y_max - plot_y_min;

        // Plot parametric data points
        let data_width = width.saturating_sub(6);
        for (&x_val, &y_val) in result.x_values.iter().zip(result.y_values.iter()) {
            let x_pos = 5 + ((x_val - plot_x_min) / plot_x_range * data_width as f64) as usize;
            let y_pos = ((plot_y_max - y_val) / plot_y_range * (height - 1) as f64) as usize;
            
            if x_pos < width && y_pos < height {
                grid[y_pos][x_pos] = '*';
            }
        }

        // Add axes and labels for parametric plots
        add_ascii_parametric_axes(&mut grid, width, height, plot_x_min, plot_x_max, plot_y_min, plot_y_max);
        
        format_ascii_parametric_output(
            grid,
            width,
            result.len(),
            x_min,
            x_max,
            y_min,
            y_max,
        )
    }

    fn render_surface3d(
        &self,
        result: &Expression3dResult,
        width: usize,
        height: usize,
    ) -> String {
        if result.is_empty() {
            return "No 3D surface data to plot".to_string();
        }

        let mut grid = vec![vec![' '; width]; height];
        
        // Create a simple ASCII 3D projection using contour lines
        let z_min = result.z_min();
        let z_max = result.z_max();
        let z_range = z_max - z_min;
        
        if z_range.abs() < f64::EPSILON {
            return format!("3D Surface: constant Z = {:.3}", z_min);
        }
        
        // Map surface to ASCII characters based on height
        let data_width = width.saturating_sub(6);
        let data_height = height.saturating_sub(4);
        
        for (y_idx, z_row) in result.data.iter().enumerate() {
            for (x_idx, &z_val) in z_row.iter().enumerate() {
                let screen_x = 5 + (x_idx * data_width / result.x_len().max(1));
                let screen_y = 2 + (y_idx * data_height / result.y_len().max(1));
                
                if screen_x < width && screen_y < height {
                    let normalized_z = (z_val - z_min) / z_range;
                    let char_val = if normalized_z < 0.2 {
                        '.'
                    } else if normalized_z < 0.4 {
                        ':'
                    } else if normalized_z < 0.6 {
                        '+'
                    } else if normalized_z < 0.8 {
                        '*'
                    } else {
                        '#'
                    };
                    grid[screen_y][screen_x] = char_val;
                }
            }
        }
        
        format_ascii_surface3d_output(
            grid,
            width,
            result.x_len() * result.y_len(),
            result.x_min(),
            result.x_max(),
            result.y_min(),
            result.y_max(),
            z_min,
            z_max,
        )
    }
}

#[derive(Clone, Debug)]
pub struct AnsiRenderer;

impl DisplayRenderer for AnsiRenderer {
    fn render(
        &self,
        result: &ExpressionRange1dResult,
        width: usize,
        height: usize,
        x_range: &ExpressionRange1dResult,
    ) -> String {
        if result.0.is_empty() {
            return "\x1b[31mNo data to plot\x1b[0m".to_string();
        }

        let min_val = result.min();
        let max_val = result.max();
        let x_min = x_range.min();
        let x_max = x_range.max();

        if (max_val - min_val).abs() < f64::EPSILON {
            return format!(
                "\x1b[33mConstant value: {:.3}\x1b[0m (all {} points)\nX range: [{:.2}, {:.2}]\n{}",
                min_val,
                result.0.len(),
                x_min,
                x_max,
                " ".repeat(width / 2) + &"\x1b[32m●\x1b[0m".repeat(result.0.len().min(width))
            );
        }

        let mut grid = vec![vec![' '; width]; height];
        let mut colors = vec![vec![0u8; width]; height];
        let padding = (max_val - min_val) * 0.1;
        let plot_min = min_val - padding;
        let plot_max = max_val + padding;
        let plot_range = plot_max - plot_min;

        // Plot data points
        let data_width = width.saturating_sub(6);
        for (i, &value) in result.0.iter().enumerate() {
            let x_pos = if result.0.len() > 1 {
                5 + (i * data_width / (result.0.len() - 1))
            } else {
                width / 2
            };

            if x_pos < width {
                let normalized_y = (value - plot_min) / plot_range;
                let y_pos = ((1.0 - normalized_y) * (height - 1) as f64) as usize;
                let y_pos = y_pos.min(height - 1);
                grid[y_pos][x_pos] = '●';
                colors[y_pos][x_pos] = 1;
            }
        }

        // Add axes and labels
        add_ansi_axes(&mut grid, &mut colors, width, height, plot_min, plot_max);
        format_ansi_output(
            grid,
            colors,
            width,
            result.0.len(),
            x_min,
            x_max,
            min_val,
            max_val,
            x_range,
        )
    }

    fn render_parametric(
        &self,
        result: &Parametric2DResult,
        width: usize,
        height: usize,
    ) -> String {
        if result.is_empty() {
            return "\x1b[31mNo parametric data to plot\x1b[0m".to_string();
        }

        let x_min = result.x_values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let x_max = result.x_values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        let y_min = result.y_values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let y_max = result.y_values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));

        let x_range = x_max - x_min;
        let y_range = y_max - y_min;

        if x_range.abs() < f64::EPSILON || y_range.abs() < f64::EPSILON {
            return format!(
                "\x1b[33mParametric plot: X=[{:.3}, {:.3}], Y=[{:.3}, {:.3}] (constant values)\x1b[0m",
                x_min, x_max, y_min, y_max
            );
        }

        let mut grid = vec![vec![' '; width]; height];
        let mut colors = vec![vec![0u8; width]; height];
        let padding_x = x_range * 0.1;
        let padding_y = y_range * 0.1;
        let plot_x_min = x_min - padding_x;
        let plot_x_max = x_max + padding_x;
        let plot_y_min = y_min - padding_y;
        let plot_y_max = y_max + padding_y;
        let plot_x_range = plot_x_max - plot_x_min;
        let plot_y_range = plot_y_max - plot_y_min;

        // Plot parametric data points
        let data_width = width.saturating_sub(6);
        for (&x_val, &y_val) in result.x_values.iter().zip(result.y_values.iter()) {
            let x_pos = 5 + ((x_val - plot_x_min) / plot_x_range * data_width as f64) as usize;
            let y_pos = ((plot_y_max - y_val) / plot_y_range * (height - 1) as f64) as usize;
            
            if x_pos < width && y_pos < height {
                grid[y_pos][x_pos] = '●';
                colors[y_pos][x_pos] = 1;
            }
        }

        // Add axes and labels for parametric plots
        add_ansi_parametric_axes(&mut grid, &mut colors, width, height, plot_x_min, plot_x_max, plot_y_min, plot_y_max);
        
        format_ansi_parametric_output(
            grid,
            colors,
            width,
            result.len(),
            x_min,
            x_max,
            y_min,
            y_max,
        )
    }

    fn render_surface3d(
        &self,
        result: &Expression3dResult,
        width: usize,
        height: usize,
    ) -> String {
        if result.is_empty() {
            return "\\x1b[31mNo 3D surface data to plot\\x1b[0m".to_string();
        }

        let mut grid = vec![vec![' '; width]; height];
        let mut colors = vec![vec![0u8; width]; height];
        
        // Create a colorized ANSI 3D projection
        let z_min = result.z_min();
        let z_max = result.z_max();
        let z_range = z_max - z_min;
        
        if z_range.abs() < f64::EPSILON {
            return format!("\\x1b[33m3D Surface: constant Z = {:.3}\\x1b[0m", z_min);
        }
        
        // Map surface to ANSI characters and colors based on height
        let data_width = width.saturating_sub(6);
        let data_height = height.saturating_sub(4);
        
        for (y_idx, z_row) in result.data.iter().enumerate() {
            for (x_idx, &z_val) in z_row.iter().enumerate() {
                let screen_x = 5 + (x_idx * data_width / result.x_len().max(1));
                let screen_y = 2 + (y_idx * data_height / result.y_len().max(1));
                
                if screen_x < width && screen_y < height {
                    let normalized_z = (z_val - z_min) / z_range;
                    let (char_val, color_val) = if normalized_z < 0.2 {
                        ('·', 4) // dim
                    } else if normalized_z < 0.4 {
                        ('▪', 2) // blue-ish
                    } else if normalized_z < 0.6 {
                        ('▫', 3) // yellow-ish
                    } else if normalized_z < 0.8 {
                        ('●', 1) // bright
                    } else {
                        ('█', 5) // very bright
                    };
                    grid[screen_y][screen_x] = char_val;
                    colors[screen_y][screen_x] = color_val;
                }
            }
        }
        
        format_ansi_surface3d_output(
            grid,
            colors,
            width,
            result.x_len() * result.y_len(),
            result.x_min(),
            result.x_max(),
            result.y_min(),
            result.y_max(),
            z_min,
            z_max,
        )
    }
}

#[derive(Clone, Debug)]
pub struct RegisRenderer;
fn regis_init(width: usize, height: usize) -> String {
    let mut init = format!("\x1bP0p\nS(A[0,0][{},{}])\nS(E)\n", width, height);
    init.push_str("S(C1)\n");
    init
}

fn regis_finish() -> String {
    "\x1b\\".to_string()
}

fn regis_draw_grid_and_axes(
    width: usize,
    height: usize,
    result: &ExpressionRange1dResult,
    x_range: &ExpressionRange1dResult,
) -> String {
    let mut output = String::new();

    let y_min = result.min();
    let y_max = result.max();
    let y_range = y_max - y_min;
    let x_min = x_range.min();
    let x_max = x_range.max();
    let x_range_val = x_max - x_min;

    let y_padding = if y_range > 0.0 { y_range * 0.1 } else { 1.0 };
    let plot_y_min = y_min - y_padding;
    let plot_y_max = y_max + y_padding;
    let plot_y_range = plot_y_max - plot_y_min;

    let data_to_screen_x = |x_data: f64| -> usize {
        if x_range_val > 0.0 {
            ((x_data - x_min) / x_range_val * width as f64) as usize
        } else {
            width / 2
        }
    };

    let data_to_screen_y = |y_data: f64| -> usize {
        if plot_y_range > 0.0 {
            let normalized = (y_data - plot_y_min) / plot_y_range;
            ((1.0 - normalized) * height as f64) as usize
        } else {
            height / 2
        }
    };

    // Draw grid lines
    output.push_str("W(P3)\nS(C2)\n");
    for i in 1..10 {
        let x = (i * width) / 10;
        output.push_str(&format!("P[{},0]\nV[{},{}]\n", x, x, height));
    }
    for i in 1..8 {
        let y = (i * height) / 8;
        output.push_str(&format!("P[0,{}]\nV[{},{}]\n", y, width, y));
    }

    // Draw axes
    output.push_str("W(P0)\nS(C1)\n");
    let x_axis_y = if plot_y_min <= 0.0 && plot_y_max >= 0.0 {
        data_to_screen_y(0.0)
    } else {
        height - 1
    };
    output.push_str(&format!("P[0,{}]\nV[{},{}]\n", x_axis_y, width, x_axis_y));
    let y_axis_x = if x_min <= 0.0 && x_max >= 0.0 {
        data_to_screen_x(0.0)
    } else {
        0
    };
    output.push_str(&format!("P[{},0]\nV[{},{}]\n", y_axis_x, y_axis_x, height));

    // Add axis labels
    output.push_str("W(P2)\nS(C1)\n");

    // X-axis labels
    for i in 0..=5 {
        let x_screen = (i * width) / 5;
        let x_data = x_min + (i as f64 / 5.0) * x_range_val;
        let label_y = (x_axis_y + 20).min(height - 10);

        let label = if x_data.abs() < 0.01 {
            "0".to_string()
        } else if x_data.abs() >= 1000.0 {
            format!("{:.0}", x_data)
        } else if x_data.fract() == 0.0 {
            format!("{:.0}", x_data)
        } else {
            format!("{:.1}", x_data)
        };

        let text_x = x_screen.saturating_sub(label.len() * 3);
        output.push_str(&format!("P[{},{}]\nT(S1)'{}'\n", text_x, label_y, label));
    }

    // Y-axis labels
    for i in 0..=5 {
        let y_screen = (i * height) / 5;
        let y_data = plot_y_max - (i as f64 / 5.0) * plot_y_range;
        let label_x = y_axis_x.saturating_sub(30).max(5);

        let label = if y_data.abs() < 0.01 {
            "0".to_string()
        } else if y_data.abs() >= 1000.0 {
            format!("{:.0}", y_data)
        } else {
            format!("{:.2}", y_data)
        };

        let text_y = y_screen.saturating_sub(5);
        output.push_str(&format!("P[{},{}]\nT(S1)'{}'\n", label_x, text_y, label));
    }

    // Add Info
    output.push_str(&format!("P[5,15]\nT(S2)'Max: {:.2}'\n", y_max));
    output.push_str(&format!("P[5,{}]\nT(S2)'Min: {:.2}'\n", height - 30, y_min));
    output.push_str(&format!(
        "P[{},{}]\nT(S2)'X: {:.1} to {:.1}'\n",
        width - 100,
        height - 15,
        x_min,
        x_max
    ));

    output
}

fn regis_plot_data(result: &ExpressionRange1dResult, width: usize, height: usize) -> String {
    if result.0.is_empty() {
        return String::new();
    }

    let mut output = String::new();

    let y_min = result.min();
    let y_max = result.max();
    let y_range = y_max - y_min;

    let y_padding = if y_range > 0.0 { y_range * 0.1 } else { 1.0 };
    let plot_y_min = y_min - y_padding;
    let plot_y_max = y_max + y_padding;
    let plot_y_range = plot_y_max - plot_y_min;

    let data_to_screen_x = |index: usize| -> usize {
        if result.0.len() > 1 {
            (index * width) / (result.0.len() - 1)
        } else {
            width / 2
        }
    };

    let data_to_screen_y = |y_data: f64| -> usize {
        if plot_y_range > 0.0 {
            let normalized = (y_data - plot_y_min) / plot_y_range;
            let screen_y = ((1.0 - normalized) * height as f64) as usize;
            screen_y.min(height - 1)
        } else {
            height / 2
        }
    };

    output.push_str("W(P1)\nS(C3)\n");

    if result.0.len() > 1 {
        let x0 = data_to_screen_x(0);
        let y0 = data_to_screen_y(result.0[0]);
        output.push_str(&format!("P[{},{}]\n", x0, y0));

        for (i, &y_val) in result.0.iter().enumerate().skip(1) {
            let x = data_to_screen_x(i);
            let y = data_to_screen_y(y_val);
            output.push_str(&format!("V[{},{}]\n", x, y));
        }

        let x0 = data_to_screen_x(0);
        let y0 = data_to_screen_y(result.0[0])
            .saturating_add(1)
            .min(height - 1);
        output.push_str(&format!("P[{},{}]\n", x0, y0));

        for (i, &y_val) in result.0.iter().enumerate().skip(1) {
            let x = data_to_screen_x(i);
            let y = data_to_screen_y(y_val).saturating_add(1).min(height - 1);
            output.push_str(&format!("V[{},{}]\n", x, y));
        }
    }

    output.push_str("S(C4)\n");
    for (i, &y_val) in result.0.iter().enumerate() {
        let x = data_to_screen_x(i);
        let y = data_to_screen_y(y_val);

        let size = 1;
        for dx in 0..=size {
            for dy in 0..=size {
                let px = x.saturating_add(dx).min(width - 1);
                let py = y.saturating_add(dy).min(height - 1);
                output.push_str(&format!("P[{},{}]\nV[{},{}]\n", px, py, px, py));
            }
        }
    }

    // Add Info
    output.push_str("S(C1)\n");
    output.push_str(&format!(
        "P[{},25]\nT(S1)'Data: {} points'\n",
        width - 150,
        result.0.len()
    ));
    output.push_str(&format!(
        "P[{},45]\nT(S1)'Y: {:.2} to {:.2}'\n",
        width - 150,
        y_min,
        y_max
    ));

    output
}

fn regis_parametric_grid_and_axes(
    result: &Parametric2DResult,
    width: usize,
    height: usize,
) -> String {
    let mut output = String::new();

    let x_min = result.x_values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
    let x_max = result.x_values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
    let y_min = result.y_values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
    let y_max = result.y_values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));

    let x_range = x_max - x_min;
    let y_range = y_max - y_min;

    let x_padding = if x_range > 0.0 { x_range * 0.1 } else { 1.0 };
    let y_padding = if y_range > 0.0 { y_range * 0.1 } else { 1.0 };
    let plot_x_min = x_min - x_padding;
    let plot_x_max = x_max + x_padding;
    let plot_y_min = y_min - y_padding;
    let plot_y_max = y_max + y_padding;
    let plot_x_range = plot_x_max - plot_x_min;
    let plot_y_range = plot_y_max - plot_y_min;

    let data_to_screen_x = |x_data: f64| -> usize {
        if plot_x_range > 0.0 {
            ((x_data - plot_x_min) / plot_x_range * width as f64) as usize
        } else {
            width / 2
        }
    };

    let data_to_screen_y = |y_data: f64| -> usize {
        if plot_y_range > 0.0 {
            let normalized = (y_data - plot_y_min) / plot_y_range;
            ((1.0 - normalized) * height as f64) as usize
        } else {
            height / 2
        }
    };

    // Draw grid lines
    output.push_str("W(P3)\nS(C2)\n");
    for i in 1..10 {
        let x = (i * width) / 10;
        output.push_str(&format!("P[{},0]\nV[{},{}]\n", x, x, height));
    }
    for i in 1..8 {
        let y = (i * height) / 8;
        output.push_str(&format!("P[0,{}]\nV[{},{}]\n", y, width, y));
    }

    // Draw axes
    output.push_str("W(P0)\nS(C1)\n");
    
    // X-axis (horizontal line at y=0 if 0 is in range)
    let x_axis_y = if plot_y_min <= 0.0 && plot_y_max >= 0.0 {
        data_to_screen_y(0.0)
    } else {
        height - 1
    };
    output.push_str(&format!("P[0,{}]\nV[{},{}]\n", x_axis_y, width, x_axis_y));
    
    // Y-axis (vertical line at x=0 if 0 is in range)
    let y_axis_x = if plot_x_min <= 0.0 && plot_x_max >= 0.0 {
        data_to_screen_x(0.0)
    } else {
        0
    };
    output.push_str(&format!("P[{},0]\nV[{},{}]\n", y_axis_x, y_axis_x, height));

    // Add axis labels
    output.push_str("W(P2)\nS(C1)\n");

    // X-axis labels
    for i in 0..=5 {
        let x_screen = (i * width) / 5;
        let x_data = plot_x_min + (i as f64 / 5.0) * plot_x_range;
        let label_y = (x_axis_y + 20).min(height - 10);

        let label = if x_data.abs() < 0.01 {
            "0".to_string()
        } else if x_data.abs() >= 1000.0 {
            format!("{:.0}", x_data)
        } else if x_data.fract() == 0.0 {
            format!("{:.0}", x_data)
        } else {
            format!("{:.1}", x_data)
        };

        let text_x = x_screen.saturating_sub(label.len() * 3);
        output.push_str(&format!("P[{},{}]\nT(S1)'{}'\n", text_x, label_y, label));
    }

    // Y-axis labels
    for i in 0..=5 {
        let y_screen = (i * height) / 5;
        let y_data = plot_y_max - (i as f64 / 5.0) * plot_y_range;
        let label_x = y_axis_x.saturating_sub(30).max(5);

        let label = if y_data.abs() < 0.01 {
            "0".to_string()
        } else if y_data.abs() >= 1000.0 {
            format!("{:.0}", y_data)
        } else {
            format!("{:.2}", y_data)
        };

        let text_y = y_screen.saturating_sub(5);
        output.push_str(&format!("P[{},{}]\nT(S1)'{}'\n", label_x, text_y, label));
    }

    // Add plot info
    output.push_str(&format!("P[5,15]\nT(S2)'Parametric Plot: {} points'\n", result.len()));
    output.push_str(&format!("P[5,30]\nT(S2)'X: {:.2} to {:.2}'\n", x_min, x_max));
    output.push_str(&format!("P[5,45]\nT(S2)'Y: {:.2} to {:.2}'\n", y_min, y_max));

    output
}

fn regis_parametric_plot(result: &Parametric2DResult, width: usize, height: usize) -> String {
    if result.is_empty() {
        return String::new();
    }

    let mut output = String::new();
    output.push_str("W(P1)\nS(C3)\n");

    let x_min = result.x_values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
    let x_max = result.x_values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
    let y_min = result.y_values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
    let y_max = result.y_values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));

    let x_range = x_max - x_min;
    let y_range = y_max - y_min;

    if x_range.abs() < f64::EPSILON || y_range.abs() < f64::EPSILON {
        return output;
    }

    // Add padding to match the axes coordinate system
    let x_padding = x_range * 0.1;
    let y_padding = y_range * 0.1;
    let plot_x_min = x_min - x_padding;
    let plot_x_max = x_max + x_padding;
    let plot_y_min = y_min - y_padding;
    let plot_y_max = y_max + y_padding;
    let plot_x_range = plot_x_max - plot_x_min;
    let plot_y_range = plot_y_max - plot_y_min;

    let data_to_screen_x = |x_val: f64| -> usize {
        if plot_x_range > 0.0 {
            ((x_val - plot_x_min) / plot_x_range * width as f64) as usize
        } else {
            width / 2
        }
    };

    let data_to_screen_y = |y_val: f64| -> usize {
        if plot_y_range > 0.0 {
            let normalized = (y_val - plot_y_min) / plot_y_range;
            ((1.0 - normalized) * height as f64) as usize
        } else {
            height / 2
        }
    };

    if let (Some(&first_x), Some(&first_y)) = (result.x_values.first(), result.y_values.first()) {
        let x0 = data_to_screen_x(first_x);
        let y0 = data_to_screen_y(first_y);
        output.push_str(&format!("P[{},{}]\n", x0, y0));

        for (&x_val, &y_val) in result.x_values.iter().zip(result.y_values.iter()).skip(1) {
            let x = data_to_screen_x(x_val);
            let y = data_to_screen_y(y_val);
            output.push_str(&format!("V[{},{}]\n", x, y));
        }
    }

    output
}

impl DisplayRenderer for RegisRenderer {
    fn render(
        &self,
        result: &ExpressionRange1dResult,
        width: usize,
        height: usize,
        x_range: &ExpressionRange1dResult,
    ) -> String {
        if result.0.is_empty() {
            return "No data to plot".to_string();
        }

        let mut output = String::new();
        output.push_str(&regis_init(width, height));
        output.push_str(&regis_draw_grid_and_axes(width, height, result, x_range));
        output.push_str(&regis_plot_data(result, width, height));
        output.push_str(&regis_finish());
        output
    }

    fn render_parametric(
        &self,
        result: &Parametric2DResult,
        width: usize,
        height: usize,
    ) -> String {
        if result.is_empty() {
            return "No parametric data to plot".to_string();
        }

        let mut output = String::new();
        output.push_str(&regis_init(width, height));
        output.push_str(&regis_parametric_grid_and_axes(result, width, height));
        output.push_str(&regis_parametric_plot(result, width, height));
        output.push_str(&regis_finish());
        output
    }

    fn render_surface3d(
        &self,
        result: &Expression3dResult,
        width: usize,
        height: usize,
    ) -> String {
        if result.is_empty() {
            return "No 3D surface data to plot".to_string();
        }

        let mut output = String::new();
        output.push_str(&regis_init(width, height));
        output.push_str(&regis_surface3d_grid_and_axes(result, width, height));
        output.push_str(&regis_surface3d_plot(result, width, height));
        output.push_str(&regis_finish());
        output
    }
}

#[derive(Clone, Debug)]
pub struct SixelRenderer;

impl DisplayRenderer for SixelRenderer {
    fn render(
        &self,
        result: &ExpressionRange1dResult,
        width: usize,
        height: usize,
        x_range: &ExpressionRange1dResult,
    ) -> String {
        if result.0.is_empty() {
            return "No data to plot".to_string();
        }

        let margin = 50;
        let total_width = width + 2 * margin;
        let total_height = height + 2 * margin;

        let mut bitmap = Bitmap::new(total_width, total_height, 0);
        bitmap.create_plot(result, x_range, margin);

        bitmap_to_sixel(&bitmap, result)
    }

    fn render_parametric(
        &self,
        result: &Parametric2DResult,
        width: usize,
        height: usize,
    ) -> String {
        if result.is_empty() {
            return "No parametric data to plot".to_string();
        }

        let margin = 50;
        let total_width = width + 2 * margin;
        let total_height = height + 2 * margin;

        let mut bitmap = Bitmap::new(total_width, total_height, 0);
        bitmap.create_parametric_plot(result, margin);

        bitmap_to_sixel(&bitmap, &ExpressionRange1dResult::from(result.y_values.clone()))
    }

    fn render_surface3d(
        &self,
        result: &Expression3dResult,
        width: usize,
        height: usize,
    ) -> String {
        if result.is_empty() {
            return "No 3D surface data to plot".to_string();
        }

        let margin = 50;
        let total_width = width + 2 * margin;
        let total_height = height + 2 * margin;

        let mut bitmap = Bitmap::new(total_width, total_height, 0);
        bitmap.create_surface3d_plot(result, margin);

        bitmap_to_sixel(&bitmap, &ExpressionRange1dResult::from(vec![result.z_min(), result.z_max()]))
    }
}

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
            writeln!(file, "{},{}", x_val, y_val)?;
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
            writeln!(file, "{},{}", x_val, y_val)?;
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
                    writeln!(file, "{},{},{}", x_val, y_val, z_val)?;
                }
            }
        }
        Ok(())
    }
}

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
        writeln!(file, "P3\n{} {}\n255", total_width, total_height)?;

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
                write!(file, "{} {} {} ", r, g, b)?;
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
        writeln!(file, "P3\n{} {}\n255", total_width, total_height)?;

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
                write!(file, "{} {} {} ", r, g, b)?;
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
        writeln!(file, "P3\n{} {}\n255", total_width, total_height)?;

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
                write!(file, "{} {} {} ", r, g, b)?;
            }
            writeln!(file)?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct SvgWriter;

impl OutputWriter for SvgWriter {
    fn write(
        &self,
        filename: &str,
        x_result: &ExpressionRange1dResult,
        y_result: &ExpressionRange1dResult,
        width: usize,
        height: usize,
    ) -> Result<(), Box<dyn Error>> {
        let mut file = std::fs::File::create(filename)?;
        let margin = 50;
        let plot_width = width - 2 * margin;
        let plot_height = height - 2 * margin;

        writeln!(file, r#"<?xml version="1.0" encoding="UTF-8"?>"#)?;
        writeln!(
            file,
            r#"<svg width="{}" height="{}" xmlns="http://www.w3.org/2000/svg">"#,
            width, height
        )?;
        writeln!(
            file,
            r#"<rect width="{}" height="{}" fill="black"/>"#,
            width, height
        )?;

        if !y_result.0.is_empty() {
            let min_val = y_result.min();
            let max_val = y_result.max();
            let x_min = x_result.min();
            let x_max = x_result.max();
            let y_range = max_val - min_val;
            let x_range_val = x_max - x_min;

            // Draw grid
            for i in 1..10 {
                let x = margin + (i * plot_width) / 10;
                writeln!(
                    file,
                    r#"<line x1="{}" y1="{}" x2="{}" y2="{}" stroke="rgb(64,64,64)" stroke-width="1"/>"#,
                    x,
                    margin,
                    x,
                    margin + plot_height
                )?;
            }
            for i in 1..8 {
                let y = margin + (i * plot_height) / 8;
                writeln!(
                    file,
                    r#"<line x1="{}" y1="{}" x2="{}" y2="{}" stroke="rgb(64,64,64)" stroke-width="1"/>"#,
                    margin,
                    y,
                    margin + plot_width,
                    y
                )?;
            }

            // Draw axes
            writeln!(
                file,
                r#"<line x1="{}" y1="{}" x2="{}" y2="{}" stroke="yellow" stroke-width="2"/>"#,
                margin,
                margin + plot_height,
                margin + plot_width,
                margin + plot_height
            )?;
            writeln!(
                file,
                r#"<line x1="{}" y1="{}" x2="{}" y2="{}" stroke="yellow" stroke-width="2"/>"#,
                margin,
                margin,
                margin,
                margin + plot_height
            )?;

            // Y-axis labels
            for i in 0..=5 {
                let y = margin + (i * plot_height) / 5;
                let value = max_val - (i as f64 / 5.0) * (max_val - min_val);
                writeln!(
                    file,
                    r#"<text x="{}" y="{}" fill="rgb(192,192,192)" font-family="monospace" font-size="10" text-anchor="end">{:.1}</text>"#,
                    margin - 5,
                    y + 3,
                    value
                )?;
            }

            // X-axis labels
            for i in 0..=5 {
                let x = margin + (i * plot_width) / 5;
                let value = x_min + (i as f64 / 5.0) * (x_max - x_min);
                writeln!(
                    file,
                    r#"<text x="{}" y="{}" fill="rgb(192,192,192)" font-family="monospace" font-size="10" text-anchor="middle">{:.1}</text>"#,
                    x,
                    margin + plot_height + 15,
                    value
                )?;
            }

            // Plot data
            if y_range > f64::EPSILON && x_range_val > f64::EPSILON {
                let mut path_data = String::new();
                for (i, (&x_val, &y_val)) in x_result.0.iter().zip(y_result.0.iter()).enumerate() {
                    let x_svg =
                        margin + ((x_val - x_min) / x_range_val * plot_width as f64) as usize;
                    let y_svg =
                        margin + (((max_val - y_val) / y_range) * plot_height as f64) as usize;

                    if i == 0 {
                        path_data.push_str(&format!("M {} {}", x_svg, y_svg));
                    } else {
                        path_data.push_str(&format!(" L {} {}", x_svg, y_svg));
                    }

                    // Draw data points
                    writeln!(
                        file,
                        r#"<circle cx="{}" cy="{}" r="2" fill="cyan"/>"#,
                        x_svg, y_svg
                    )?;
                }
                writeln!(
                    file,
                    r#"<path d="{}" fill="none" stroke="cyan" stroke-width="2"/>"#,
                    path_data
                )?;
            }
        }

        writeln!(file, "</svg>")?;
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
        let margin = 50;
        let plot_width = width - 2 * margin;
        let plot_height = height - 2 * margin;

        writeln!(file, r#"<?xml version="1.0" encoding="UTF-8"?>"#)?;
        writeln!(
            file,
            r#"<svg width="{}" height="{}" xmlns="http://www.w3.org/2000/svg">"#,
            width, height
        )?;
        writeln!(
            file,
            r#"<rect width="{}" height="{}" fill="black"/>"#,
            width, height
        )?;

        if !result.is_empty() {
            let x_min = result.x_values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
            let x_max = result.x_values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
            let y_min = result.y_values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
            let y_max = result.y_values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
            let x_range = x_max - x_min;
            let y_range = y_max - y_min;

            // Plot parametric data
            if x_range > f64::EPSILON && y_range > f64::EPSILON {
                let mut path_data = String::new();
                for (i, (&x_val, &y_val)) in result.x_values.iter().zip(result.y_values.iter()).enumerate() {
                    let x_svg = margin + ((x_val - x_min) / x_range * plot_width as f64) as usize;
                    let y_svg = margin + (((y_max - y_val) / y_range) * plot_height as f64) as usize;

                    if i == 0 {
                        path_data.push_str(&format!("M {} {}", x_svg, y_svg));
                    } else {
                        path_data.push_str(&format!(" L {} {}", x_svg, y_svg));
                    }

                    writeln!(
                        file,
                        r#"<circle cx="{}" cy="{}" r="2" fill="cyan"/>"#,
                        x_svg, y_svg
                    )?;
                }
                writeln!(
                    file,
                    r#"<path d="{}" fill="none" stroke="cyan" stroke-width="2"/>"#,
                    path_data
                )?;
            }
        }

        writeln!(file, "</svg>")?;
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
        let margin = 50;
        let plot_width = width - 2 * margin;
        let plot_height = height - 2 * margin;

        writeln!(file, r#"<?xml version="1.0" encoding="UTF-8"?>"#)?;
        writeln!(
            file,
            r#"<svg width="{}" height="{}" xmlns="http://www.w3.org/2000/svg">"#,
            width, height
        )?;
        writeln!(
            file,
            r#"<rect width="{}" height="{}" fill="black"/>"#,
            width, height
        )?;

        if !result.is_empty() {
            let x_min = result.x_min();
            let x_max = result.x_max();
            let y_min = result.y_min();
            let y_max = result.y_max();
            let z_min = result.z_min();
            let z_max = result.z_max();
            let x_range = x_max - x_min;
            let y_range = y_max - y_min;
            let z_range = z_max - z_min;

            // Create a simple 3D-like projection by mapping x,y to 2D and using z for color intensity
            if x_range > f64::EPSILON && y_range > f64::EPSILON {
                for (y_idx, &y_val) in result.y_values.iter().enumerate() {
                    for (x_idx, &x_val) in result.x_values.iter().enumerate() {
                        if let Some(z_val) = result.get_z(x_idx, y_idx) {
                            let x_svg = margin + ((x_val - x_min) / x_range * plot_width as f64) as usize;
                            let y_svg = margin + (((y_max - y_val) / y_range) * plot_height as f64) as usize;
                            
                            // Color intensity based on z value
                            let intensity = if z_range > f64::EPSILON {
                                ((z_val - z_min) / z_range * 255.0) as u8
                            } else {
                                128
                            };
                            
                            writeln!(
                                file,
                                r#"<rect x="{}" y="{}" width="3" height="3" fill="rgb({},{},{})" opacity="0.8"/>"#,
                                x_svg.saturating_sub(1), y_svg.saturating_sub(1),
                                intensity, intensity, 255
                            )?;
                        }
                    }
                }
            }
        }

        writeln!(file, "</svg>")?;
        Ok(())
    }
}
#[derive(Clone, Debug)]
pub struct LatexWriter;

impl OutputWriter for LatexWriter {
    fn write(
        &self,
        filename: &str,
        x_result: &ExpressionRange1dResult,
        y_result: &ExpressionRange1dResult,
        width: usize,
        height: usize,
    ) -> Result<(), Box<dyn Error>> {
        let mut file = std::fs::File::create(filename)?;

        writeln!(file, r"\documentclass{{article}}")?;
        writeln!(file, r"\usepackage{{pgfplots}}")?;
        writeln!(file, r"\pgfplotsset{{compat=1.18}}")?;
        writeln!(file, r"\begin{{document}}")?;
        writeln!(file, r"\begin{{tikzpicture}}")?;
        writeln!(file, r"\begin{{axis}}[")?;
        writeln!(
            file,
            r"    width={}cm, height={}cm,",
            width as f64 / 100.0,
            height as f64 / 100.0
        )?;
        writeln!(file, r"    xlabel={{X}}, ylabel={{Y}},")?;
        writeln!(
            file,
            r"    xmin={:.3}, xmax={:.3},",
            x_result.min(),
            x_result.max()
        )?;
        writeln!(
            file,
            r"    ymin={:.3}, ymax={:.3},",
            y_result.min(),
            y_result.max()
        )?;
        writeln!(file, r"    grid=major")?;
        writeln!(file, r"]")?;

        writeln!(
            file,
            r"\addplot[blue, mark=*, mark size=1pt] coordinates {{"
        )?;
        for (&x_val, &y_val) in x_result.0.iter().zip(y_result.0.iter()) {
            writeln!(file, "    ({:.6}, {:.6})", x_val, y_val)?;
        }
        writeln!(file, r"}};")?;

        writeln!(file, r"\end{{axis}}")?;
        writeln!(file, r"\end{{tikzpicture}}")?;
        writeln!(file, r"\end{{document}}")?;
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

        writeln!(file, r"\documentclass{{article}}")?;
        writeln!(file, r"\usepackage{{pgfplots}}")?;
        writeln!(file, r"\pgfplotsset{{compat=1.18}}")?;
        writeln!(file, r"\begin{{document}}")?;
        writeln!(file, r"\begin{{tikzpicture}}")?;
        writeln!(file, r"\begin{{axis}}[")?;
        writeln!(
            file,
            r"    width={}cm, height={}cm,",
            width as f64 / 100.0,
            height as f64 / 100.0
        )?;
        writeln!(file, r"    xlabel={{X}}, ylabel={{Y}},")?;

        let x_min = result.x_values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let x_max = result.x_values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        let y_min = result.y_values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let y_max = result.y_values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));

        writeln!(file, r"    xmin={:.3}, xmax={:.3},", x_min, x_max)?;
        writeln!(file, r"    ymin={:.3}, ymax={:.3},", y_min, y_max)?;
        writeln!(file, r"    grid=major")?;
        writeln!(file, r"]")?;

        writeln!(
            file,
            r"\addplot[red, mark=*, mark size=1pt] coordinates {{"
        )?;
        for (&x_val, &y_val) in result.x_values.iter().zip(result.y_values.iter()) {
            writeln!(file, "    ({:.6}, {:.6})", x_val, y_val)?;
        }
        writeln!(file, r"}};")?;

        writeln!(file, r"\end{{axis}}")?;
        writeln!(file, r"\end{{tikzpicture}}")?;
        writeln!(file, r"\end{{document}}")?;
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

        writeln!(file, r"\documentclass{{article}}")?;
        writeln!(file, r"\usepackage{{pgfplots}}")?;
        writeln!(file, r"\pgfplotsset{{compat=1.18}}")?;
        writeln!(file, r"\begin{{document}}")?;
        writeln!(file, r"\begin{{tikzpicture}}")?;
        writeln!(file, r"\begin{{axis}}[")?;
        writeln!(
            file,
            r"    width={}cm, height={}cm,",
            width as f64 / 100.0,
            height as f64 / 100.0
        )?;
        writeln!(file, r"    xlabel={{X}}, ylabel={{Y}}, zlabel={{Z}},")?;
        writeln!(
            file,
            r"    xmin={:.3}, xmax={:.3},",
            result.x_min(),
            result.x_max()
        )?;
        writeln!(
            file,
            r"    ymin={:.3}, ymax={:.3},",
            result.y_min(),
            result.y_max()
        )?;
        writeln!(
            file,
            r"    zmin={:.3}, zmax={:.3},",
            result.z_min(),
            result.z_max()
        )?;
        writeln!(file, r"    grid=major,")?;
        writeln!(file, r"    view={{30}}{{30}}")?;
        writeln!(file, r"]")?;

        writeln!(file, r"\addplot3[surf, mesh/rows={}, mesh/cols={}]", result.y_len(), result.x_len())?;
        writeln!(file, "coordinates {{")?;
        for (y_idx, &y_val) in result.y_values.iter().enumerate() {
            for (x_idx, &x_val) in result.x_values.iter().enumerate() {
                if let Some(z_val) = result.get_z(x_idx, y_idx) {
                    writeln!(file, "    ({:.6}, {:.6}, {:.6})", x_val, y_val, z_val)?;
                }
            }
        }
        writeln!(file, "}};")?;

        writeln!(file, r"\end{{axis}}")?;
        writeln!(file, r"\end{{tikzpicture}}")?;
        writeln!(file, r"\end{{document}}")?;
        Ok(())
    }
}

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
        write!(file, "{}", sixel_output)?;
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
        write!(file, "{}", sixel_output)?;
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
        write!(file, "{}", sixel_output)?;
        Ok(())
    }
}

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
        write!(file, "{}", regis_output)?;
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
        write!(file, "{}", regis_output)?;
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
        write!(file, "{}", regis_output)?;
        Ok(())
    }
}

fn add_ascii_axes(
    grid: &mut Vec<Vec<char>>,
    width: usize,
    height: usize,
    plot_min: f64,
    plot_max: f64,
) {
    // Zero line
    if plot_min <= 0.0 && plot_max >= 0.0 {
        let plot_range = plot_max - plot_min;
        let zero_y = ((1.0 - (0.0 - plot_min) / plot_range) * (height - 1) as f64) as usize;
        if zero_y < height {
            for x in 5..width {
                if grid[zero_y][x] == ' ' {
                    grid[zero_y][x] = '-';
                }
            }
        }
    }

    // Y-axis
    for y in 0..height {
        if grid[y][5] == ' ' {
            grid[y][5] = '|';
        }
    }

    // Y-axis labels
    for i in 0..5 {
        let y = i * (height - 1) / 4;
        let value = plot_max - (i as f64 / 4.0) * (plot_max - plot_min);
        let label = format!("{:4.1}", value);
        for (j, ch) in label.chars().enumerate() {
            if j < 4 && y < height {
                grid[y][j] = ch;
            }
        }
    }
}

fn add_ansi_axes(
    grid: &mut Vec<Vec<char>>,
    colors: &mut Vec<Vec<u8>>,
    width: usize,
    height: usize,
    plot_min: f64,
    plot_max: f64,
) {
    // Zero line
    if plot_min <= 0.0 && plot_max >= 0.0 {
        let plot_range = plot_max - plot_min;
        let zero_y = ((1.0 - (0.0 - plot_min) / plot_range) * (height - 1) as f64) as usize;
        if zero_y < height {
            for x in 5..width {
                if grid[zero_y][x] == ' ' {
                    grid[zero_y][x] = '─';
                    colors[zero_y][x] = 3;
                }
            }
        }
    }

    // Y-axis
    for y in 0..height {
        if grid[y][5] == ' ' {
            grid[y][5] = '│';
            colors[y][5] = 2;
        }
    }

    // Y-axis labels
    for i in 0..5 {
        let y = i * (height - 1) / 4;
        let value = plot_max - (i as f64 / 4.0) * (plot_max - plot_min);
        let label = format!("{:4.1}", value);
        for (j, ch) in label.chars().enumerate() {
            if j < 4 && y < height {
                grid[y][j] = ch;
                colors[y][j] = 2;
            }
        }
    }
}

fn format_ascii_output(
    grid: Vec<Vec<char>>,
    width: usize,
    data_points: usize,
    x_min: f64,
    x_max: f64,
    y_min: f64,
    y_max: f64,
    x_range: &ExpressionRange1dResult,
) -> String {
    let mut output = format!(
        "┌─ ASCII Plot: {} points, X:[{:.2}, {:.2}], Y:[{:.2}, {:.2}] ─┐\n",
        data_points, x_min, x_max, y_min, y_max
    );

    for row in grid {
        output.push('│');
        output.push_str(&row.into_iter().collect::<String>());
        output.push_str("│\n");
    }

    output.push_str("└");
    output.push_str(&"─".repeat(width + 1));
    output.push_str("┘\n");

    // X-axis labels
    output.push_str("X: ");
    let num_markers = 5.min(data_points);
    let data_width = width.saturating_sub(6);

    for i in 0..num_markers {
        let x_index = if num_markers > 1 {
            i * (data_points - 1) / (num_markers - 1)
        } else {
            0
        };

        let marker_pos = if num_markers > 1 {
            5 + (i * data_width / (num_markers - 1))
        } else {
            width / 2
        };

        let x_value = if x_index < x_range.0.len() {
            x_range.0[x_index]
        } else {
            x_max
        };

        if i == 0 {
            output.push_str(&" ".repeat(marker_pos.saturating_sub(3)));
        } else {
            let prev_pos = if num_markers > 1 {
                5 + ((i - 1) * data_width / (num_markers - 1))
            } else {
                width / 2
            };
            let spacing = marker_pos.saturating_sub(prev_pos).saturating_sub(4);
            output.push_str(&" ".repeat(spacing));
        }

        output.push_str(&format!("{:.1}", x_value));
    }
    output.push('\n');

    output
}

fn format_ansi_output(
    grid: Vec<Vec<char>>,
    colors: Vec<Vec<u8>>,
    width: usize,
    data_points: usize,
    x_min: f64,
    x_max: f64,
    y_min: f64,
    y_max: f64,
    x_range: &ExpressionRange1dResult,
) -> String {
    let mut output = format!(
        "\x1b[36m┌─ ANSI Plot: {} points, X:[{:.2}, {:.2}], Y:[{:.2}, {:.2}] ─┐\x1b[0m\n",
        data_points, x_min, x_max, y_min, y_max
    );

    for (row, color_row) in grid.into_iter().zip(colors.into_iter()) {
        output.push_str("\x1b[36m│\x1b[0m");
        for (ch, color) in row.into_iter().zip(color_row.into_iter()) {
            match color {
                1 => output.push_str(&format!("\x1b[92m{}\x1b[0m", ch)), // Green data
                2 => output.push_str(&format!("\x1b[37m{}\x1b[0m", ch)), // White axes
                3 => output.push_str(&format!("\x1b[93m{}\x1b[0m", ch)), // Yellow zero line
                _ => output.push(ch),
            }
        }
        output.push_str("\x1b[36m│\x1b[0m\n");
    }

    output.push_str("\x1b[36m└");
    output.push_str(&"─".repeat(width + 1));
    output.push_str("┘\x1b[0m\n");

    // X-axis labels
    output.push_str("\x1b[37mX: \x1b[0m");
    let num_markers = 5.min(data_points);
    let data_width = width.saturating_sub(6);

    for i in 0..num_markers {
        let x_index = if num_markers > 1 {
            i * (data_points - 1) / (num_markers - 1)
        } else {
            0
        };

        let marker_pos = if num_markers > 1 {
            5 + (i * data_width / (num_markers - 1))
        } else {
            width / 2
        };

        let x_value = if x_index < x_range.0.len() {
            x_range.0[x_index]
        } else {
            x_max
        };

        if i == 0 {
            output.push_str(&" ".repeat(marker_pos.saturating_sub(3)));
        } else {
            let prev_pos = if num_markers > 1 {
                5 + ((i - 1) * data_width / (num_markers - 1))
            } else {
                width / 2
            };
            let spacing = marker_pos.saturating_sub(prev_pos).saturating_sub(4);
            output.push_str(&" ".repeat(spacing));
        }

        output.push_str(&format!("\x1b[93m{:.1}\x1b[0m", x_value));
    }
    output.push('\n');

    output
}

fn bitmap_to_sixel(bitmap: &Bitmap, result: &ExpressionRange1dResult) -> String {
    let mut output = String::new();

    output.push_str("\x1bPq");
    output.push_str("\"1;1;");
    output.push_str(&format!("{};{}", bitmap.width, bitmap.height));
    output.push_str("\n");

    //Colormap
    output.push_str("#0;2;0;0;0");
    output.push_str("#1;2;0;100;0");
    output.push_str("#2;2;25;25;25");
    output.push_str("#3;2;100;100;0");
    output.push_str("#4;2;75;75;75");

    for row_chunk in (0..bitmap.height).step_by(6) {
        for color in 1..=4 {
            output.push_str(&format!("#{}", color));

            let mut repeat_count = 0;
            let mut last_char = None;

            for x in 0..bitmap.width {
                let mut sixel_value = 0u8;

                for bit in 0..6 {
                    let y = row_chunk + bit;
                    if y < bitmap.height && bitmap.get_pixel(x, y) == color {
                        sixel_value |= 1 << bit;
                    }
                }

                let sixel_char = (sixel_value + 63) as char;

                if Some(sixel_char) == last_char {
                    repeat_count += 1;
                } else {
                    if let Some(prev_char) = last_char {
                        if repeat_count > 3 {
                            output.push_str(&format!("!{}{}", repeat_count, prev_char));
                        } else {
                            for _ in 0..repeat_count {
                                output.push(prev_char);
                            }
                        }
                    }
                    last_char = Some(sixel_char);
                    repeat_count = 1;
                }
            }

            if let Some(prev_char) = last_char {
                if repeat_count > 3 {
                    output.push_str(&format!("!{}{}", repeat_count, prev_char));
                } else {
                    for _ in 0..repeat_count {
                        output.push(prev_char);
                    }
                }
            }

            output.push('$');
        }
        output.push('-');
    }

    output.push_str("\x1b\\");

    // Add text summary
    output.push_str(&format!("\nSixel Plot: {} data points\n", result.0.len()));
    output.push_str("   🟢 Data line  🟡 Axes & ticks  ⬜ Grid lines  ⬛ Plot area\n");

    output
}

fn add_ansi_parametric_axes(
    grid: &mut Vec<Vec<char>>,
    colors: &mut Vec<Vec<u8>>,
    width: usize,
    height: usize,
    plot_x_min: f64,
    plot_x_max: f64,
    plot_y_min: f64,
    plot_y_max: f64,
) {
    // X-axis (check if 0 is in range)
    if plot_y_min <= 0.0 && plot_y_max >= 0.0 {
        let plot_y_range = plot_y_max - plot_y_min;
        let zero_y = ((plot_y_max - 0.0) / plot_y_range * (height - 1) as f64) as usize;
        if zero_y < height {
            for x in 5..width {
                if grid[zero_y][x] == ' ' {
                    grid[zero_y][x] = '─';
                    colors[zero_y][x] = 3;
                }
            }
        }
    }

    // Y-axis (check if 0 is in range)
    if plot_x_min <= 0.0 && plot_x_max >= 0.0 {
        let plot_x_range = plot_x_max - plot_x_min;
        let zero_x = 5 + ((0.0 - plot_x_min) / plot_x_range * (width - 6) as f64) as usize;
        if zero_x < width {
            for y in 0..height {
                if grid[y][zero_x] == ' ' {
                    grid[y][zero_x] = '│';
                    colors[y][zero_x] = 2;
                }
            }
        }
    }

    // Y-axis labels on the left
    for i in 0..5 {
        let y = i * (height - 1) / 4;
        let value = plot_y_max - (i as f64 / 4.0) * (plot_y_max - plot_y_min);
        let label = format!("{:4.1}", value);
        for (j, ch) in label.chars().enumerate() {
            if j < 4 && y < height {
                grid[y][j] = ch;
                colors[y][j] = 2;
            }
        }
    }
}

fn format_ansi_parametric_output(
    grid: Vec<Vec<char>>,
    colors: Vec<Vec<u8>>,
    width: usize,
    data_points: usize,
    x_min: f64,
    x_max: f64,
    y_min: f64,
    y_max: f64,
) -> String {
    let mut output = format!(
        "\x1b[36m┌─ ANSI Parametric Plot: {} points, X:[{:.2}, {:.2}], Y:[{:.2}, {:.2}] ─┐\x1b[0m\n",
        data_points, x_min, x_max, y_min, y_max
    );

    for (row, color_row) in grid.into_iter().zip(colors.into_iter()) {
        output.push_str("\x1b[36m│\x1b[0m");
        for (ch, color) in row.into_iter().zip(color_row.into_iter()) {
            match color {
                1 => output.push_str(&format!("\x1b[92m{}\x1b[0m", ch)), // Green data
                2 => output.push_str(&format!("\x1b[37m{}\x1b[0m", ch)), // White axes
                3 => output.push_str(&format!("\x1b[93m{}\x1b[0m", ch)), // Yellow zero line
                _ => output.push(ch),
            }
        }
        output.push_str("\x1b[36m│\x1b[0m\n");
    }

    output.push_str("\x1b[36m└");
    output.push_str(&"─".repeat(width + 1));
    output.push_str("┘\x1b[0m\n");

    // X-axis labels
    output.push_str("\x1b[37mX: \x1b[0m");
    for i in 0..5 {
        let x_value = x_min + (i as f64 / 4.0) * (x_max - x_min);
        if i == 0 {
            output.push_str("  ");
        } else {
            output.push_str("              ");
        }
        output.push_str(&format!("\x1b[93m{:.1}\x1b[0m", x_value));
    }
    output.push('\n');

    output
}

fn add_ascii_parametric_axes(
    grid: &mut Vec<Vec<char>>,
    width: usize,
    height: usize,
    plot_x_min: f64,
    plot_x_max: f64,
    plot_y_min: f64,
    plot_y_max: f64,
) {
    // X-axis (check if 0 is in range)
    if plot_y_min <= 0.0 && plot_y_max >= 0.0 {
        let plot_y_range = plot_y_max - plot_y_min;
        let zero_y = ((plot_y_max - 0.0) / plot_y_range * (height - 1) as f64) as usize;
        if zero_y < height {
            for x in 5..width {
                if grid[zero_y][x] == ' ' {
                    grid[zero_y][x] = '-';
                }
            }
        }
    }

    // Y-axis (check if 0 is in range)
    if plot_x_min <= 0.0 && plot_x_max >= 0.0 {
        let plot_x_range = plot_x_max - plot_x_min;
        let zero_x = 5 + ((0.0 - plot_x_min) / plot_x_range * (width - 6) as f64) as usize;
        if zero_x < width {
            for y in 0..height {
                if grid[y][zero_x] == ' ' {
                    grid[y][zero_x] = '|';
                }
            }
        }
    }

    // Y-axis labels on the left
    for i in 0..5 {
        let y = i * (height - 1) / 4;
        let value = plot_y_max - (i as f64 / 4.0) * (plot_y_max - plot_y_min);
        let label = format!("{:4.1}", value);
        for (j, ch) in label.chars().enumerate() {
            if j < 4 && y < height {
                grid[y][j] = ch;
            }
        }
    }
}

fn format_ascii_parametric_output(
    grid: Vec<Vec<char>>,
    width: usize,
    data_points: usize,
    x_min: f64,
    x_max: f64,
    y_min: f64,
    y_max: f64,
) -> String {
    let mut output = format!(
        "┌─ ASCII Parametric Plot: {} points, X:[{:.2}, {:.2}], Y:[{:.2}, {:.2}] ─┐\n",
        data_points, x_min, x_max, y_min, y_max
    );

    for row in grid {
        output.push('│');
        output.push_str(&row.into_iter().collect::<String>());
        output.push_str("│\n");
    }

    output.push_str("└");
    output.push_str(&"─".repeat(width + 1));
    output.push_str("┘\n");

    // X-axis labels
    output.push_str("X: ");
    for i in 0..5 {
        let x_value = x_min + (i as f64 / 4.0) * (x_max - x_min);
        if i == 0 {
            output.push_str("  ");
        } else {
            output.push_str("              ");
        }
        output.push_str(&format!("{:.1}", x_value));
    }
    output.push('\n');

    output
}

fn format_ascii_surface3d_output(
    grid: Vec<Vec<char>>,
    width: usize,
    data_points: usize,
    x_min: f64,
    x_max: f64,
    y_min: f64,
    y_max: f64,
    z_min: f64,
    z_max: f64,
) -> String {
    let mut output = format!(
        "┌─ ASCII 3D Surface: {} points, X:[{:.2}, {:.2}], Y:[{:.2}, {:.2}], Z:[{:.2}, {:.2}] ─┐\\n",
        data_points, x_min, x_max, y_min, y_max, z_min, z_max
    );

    for row in grid {
        output.push('│');
        output.push_str(&row.into_iter().collect::<String>());
        output.push_str("│\\n");
    }

    output.push_str("└");
    output.push_str(&"─".repeat(width + 1));
    output.push_str("┘\\n");
    output.push_str("Legend: . : + * # (low to high Z)\\n");

    output
}

fn format_ansi_surface3d_output(
    grid: Vec<Vec<char>>,
    colors: Vec<Vec<u8>>,
    width: usize,
    data_points: usize,
    x_min: f64,
    x_max: f64,
    y_min: f64,
    y_max: f64,
    z_min: f64,
    z_max: f64,
) -> String {
    let mut output = format!(
        "\\x1b[36m┌─ ANSI 3D Surface: {} points, X:[{:.2}, {:.2}], Y:[{:.2}, {:.2}], Z:[{:.2}, {:.2}] ─┐\\x1b[0m\\n",
        data_points, x_min, x_max, y_min, y_max, z_min, z_max
    );

    for (row, color_row) in grid.into_iter().zip(colors.into_iter()) {
        output.push_str("\\x1b[36m│\\x1b[0m");
        for (ch, color) in row.into_iter().zip(color_row.into_iter()) {
            match color {
                1 => output.push_str(&format!("\\x1b[92m{}\\x1b[0m", ch)), // Green
                2 => output.push_str(&format!("\\x1b[94m{}\\x1b[0m", ch)), // Blue
                3 => output.push_str(&format!("\\x1b[93m{}\\x1b[0m", ch)), // Yellow
                4 => output.push_str(&format!("\\x1b[90m{}\\x1b[0m", ch)), // Dim
                5 => output.push_str(&format!("\\x1b[91m{}\\x1b[0m", ch)), // Red
                _ => output.push(ch),
            }
        }
        output.push_str("\\x1b[36m│\\x1b[0m\\n");
    }

    output.push_str("\\x1b[36m└");
    output.push_str(&"─".repeat(width + 1));
    output.push_str("┘\\x1b[0m\\n");

    output
}

fn regis_surface3d_grid_and_axes(
    result: &Expression3dResult,
    width: usize,
    height: usize,
) -> String {
    let mut output = String::new();

    let x_min = result.x_min();
    let x_max = result.x_max();
    let y_min = result.y_min();
    let y_max = result.y_max();
    let z_min = result.z_min();
    let z_max = result.z_max();

    // Draw grid lines
    output.push_str("W(P3)\\nS(C2)\\n");
    for i in 1..10 {
        let x = (i * width) / 10;
        output.push_str(&format!("P[{},0]\\nV[{},{}]\\n", x, x, height));
    }
    for i in 1..8 {
        let y = (i * height) / 8;
        output.push_str(&format!("P[0,{}]\\nV[{},{}]\\n", y, width, y));
    }

    // Draw axes
    output.push_str("W(P0)\\nS(C1)\\n");
    output.push_str(&format!("P[0,{}]\\nV[{},{}]\\n", height/2, width, height/2));
    output.push_str(&format!("P[{},0]\\nV[{},{}]\\n", width/2, width/2, height));

    // Add axis labels
    output.push_str("W(P2)\\nS(C1)\\n");
    output.push_str(&format!("P[5,15]\\nT(S2)'3D Surface: {}x{} points'\\n", result.x_len(), result.y_len()));
    output.push_str(&format!("P[5,30]\\nT(S2)'X: {:.2} to {:.2}'\\n", x_min, x_max));
    output.push_str(&format!("P[5,45]\\nT(S2)'Y: {:.2} to {:.2}'\\n", y_min, y_max));
    output.push_str(&format!("P[5,60]\\nT(S2)'Z: {:.2} to {:.2}'\\n", z_min, z_max));

    output
}

fn regis_surface3d_plot(result: &Expression3dResult, width: usize, height: usize) -> String {
    if result.is_empty() {
        return String::new();
    }

    let mut output = String::new();
    output.push_str("W(P1)\\nS(C3)\\n");

    let z_min = result.z_min();
    let z_max = result.z_max();
    let z_range = z_max - z_min;

    if z_range.abs() < f64::EPSILON {
        return output;
    }

    // Create a wireframe representation
    for (y_idx, z_row) in result.data.iter().enumerate() {
        for (x_idx, &z_val) in z_row.iter().enumerate() {
            let screen_x = (x_idx * width) / result.x_len().max(1);
            let screen_y = (y_idx * height) / result.y_len().max(1);
            
            // Add some simple 3D projection based on Z value
            let z_offset = ((z_val - z_min) / z_range * 20.0) as usize;
            let proj_x = screen_x.saturating_sub(z_offset / 2);
            let proj_y = screen_y.saturating_sub(z_offset / 2);
            
            output.push_str(&format!("P[{},{}]\\nV[{},{}]\\n", proj_x, proj_y, proj_x + 2, proj_y + 2));
        }
    }

    output
}

fn get_char_bitmap(ch: char) -> Vec<Vec<bool>> {
    match ch {
        '0' => vec![
            vec![false, true, true, true, false],
            vec![true, false, false, false, true],
            vec![true, false, false, true, true],
            vec![true, false, true, false, true],
            vec![true, true, false, false, true],
            vec![true, false, false, false, true],
            vec![false, true, true, true, false],
        ],
        '1' => vec![
            vec![false, false, true, false, false],
            vec![false, true, true, false, false],
            vec![false, false, true, false, false],
            vec![false, false, true, false, false],
            vec![false, false, true, false, false],
            vec![false, false, true, false, false],
            vec![false, true, true, true, false],
        ],
        '2' => vec![
            vec![false, true, true, true, false],
            vec![true, false, false, false, true],
            vec![false, false, false, false, true],
            vec![false, false, false, true, false],
            vec![false, false, true, false, false],
            vec![false, true, false, false, false],
            vec![true, true, true, true, true],
        ],
        '3' => vec![
            vec![false, true, true, true, false],
            vec![true, false, false, false, true],
            vec![false, false, false, false, true],
            vec![false, false, true, true, false],
            vec![false, false, false, false, true],
            vec![true, false, false, false, true],
            vec![false, true, true, true, false],
        ],
        '4' => vec![
            vec![false, false, false, true, false],
            vec![false, false, true, true, false],
            vec![false, true, false, true, false],
            vec![true, false, false, true, false],
            vec![true, true, true, true, true],
            vec![false, false, false, true, false],
            vec![false, false, false, true, false],
        ],
        '5' => vec![
            vec![true, true, true, true, true],
            vec![true, false, false, false, false],
            vec![true, true, true, true, false],
            vec![false, false, false, false, true],
            vec![false, false, false, false, true],
            vec![true, false, false, false, true],
            vec![false, true, true, true, false],
        ],
        '6' => vec![
            vec![false, false, true, true, false],
            vec![false, true, false, false, false],
            vec![true, false, false, false, false],
            vec![true, true, true, true, false],
            vec![true, false, false, false, true],
            vec![true, false, false, false, true],
            vec![false, true, true, true, false],
        ],
        '7' => vec![
            vec![true, true, true, true, true],
            vec![false, false, false, false, true],
            vec![false, false, false, true, false],
            vec![false, false, true, false, false],
            vec![false, true, false, false, false],
            vec![false, true, false, false, false],
            vec![false, true, false, false, false],
        ],
        '8' => vec![
            vec![false, true, true, true, false],
            vec![true, false, false, false, true],
            vec![true, false, false, false, true],
            vec![false, true, true, true, false],
            vec![true, false, false, false, true],
            vec![true, false, false, false, true],
            vec![false, true, true, true, false],
        ],
        '9' => vec![
            vec![false, true, true, true, false],
            vec![true, false, false, false, true],
            vec![true, false, false, false, true],
            vec![false, true, true, true, true],
            vec![false, false, false, false, true],
            vec![false, false, false, true, false],
            vec![false, true, true, false, false],
        ],
        '.' => vec![
            vec![false, false, false, false, false],
            vec![false, false, false, false, false],
            vec![false, false, false, false, false],
            vec![false, false, false, false, false],
            vec![false, false, false, false, false],
            vec![false, true, true, false, false],
            vec![false, true, true, false, false],
        ],
        '-' => vec![
            vec![false, false, false, false, false],
            vec![false, false, false, false, false],
            vec![false, false, false, false, false],
            vec![true, true, true, true, false],
            vec![false, false, false, false, false],
            vec![false, false, false, false, false],
            vec![false, false, false, false, false],
        ],
        ' ' => vec![
            vec![false, false, false, false, false],
            vec![false, false, false, false, false],
            vec![false, false, false, false, false],
            vec![false, false, false, false, false],
            vec![false, false, false, false, false],
            vec![false, false, false, false, false],
            vec![false, false, false, false, false],
        ],
        _ => vec![
            vec![true, true, true, true, true],
            vec![true, false, false, false, true],
            vec![true, false, false, false, true],
            vec![true, false, false, false, true],
            vec![true, false, false, false, true],
            vec![true, false, false, false, true],
            vec![true, true, true, true, true],
        ],
    }
}
