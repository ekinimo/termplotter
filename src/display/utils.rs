use crate::{
    values::{ExpressionRange1dResult, Expression3dResult},
    parametric2d::Parametric2DResult,
};

use super::{LabelConfig, Label3DConfig};

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
        // Normalize coordinates to [0,1] range
        let norm_x = (self.x - bounds.x_min) / (bounds.x_max - bounds.x_min);
        let norm_y = (self.y - bounds.y_min) / (bounds.y_max - bounds.y_min);
        let norm_z = if bounds.z_max != bounds.z_min {
            (self.z - bounds.z_min) / (bounds.z_max - bounds.z_min)
        } else {
            0.5
        };

        // Standard isometric projection with 30/30 degree angles
        // This creates a proper "cube view" where all axes are equally visible
        let cos30 = 0.866; // cos(30°)
        let sin30 = 0.5;   // sin(30°)
        
        // Project normalized coordinates
        let iso_x = norm_x * cos30 - norm_y * cos30;
        let iso_y = norm_x * sin30 + norm_y * sin30 + norm_z;

        // Map to screen coordinates with proper scaling
        let scale = 0.4; // Use consistent scale for all dimensions
        let screen_x = (width as f64 * 0.5) + (iso_x * width as f64 * scale);
        let screen_y = (height as f64 * 0.7) - (iso_y * height as f64 * scale);

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
            (margin, plot_width, plot_height),
            (min_val, max_val),
            (x_min, x_max),
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
            LabelConfig::new(margin, plot_width, plot_height, x_min, x_max, y_min, y_max),
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

        let x_min = surface_result.x_min();
        let x_max = surface_result.x_max();
        let y_min = surface_result.y_min();
        let y_max = surface_result.y_max();
        let z_min = surface_result.z_min();
        let z_max = surface_result.z_max();

        let x_range = x_max - x_min;
        let y_range = y_max - y_min;
        let z_range = z_max - z_min;

        if x_range.abs() < f64::EPSILON || y_range.abs() < f64::EPSILON {
            return;
        }

        // Create isometric 3D visualization
        let bounds = SurfaceBounds::from_surface(surface_result);
        
        for (y_idx, z_row) in surface_result.data.iter().enumerate() {
            for (x_idx, &z_val) in z_row.iter().enumerate() {
                // Get the actual 3D coordinates
                let x_val = surface_result.x_values[x_idx];
                let y_val = surface_result.y_values[y_idx];
                
                // Create 3D point and project to isometric coordinates
                let point_3d = Point3D::new(x_val, y_val, z_val);
                let (iso_x, iso_y) = point_3d.to_isometric(plot_width, plot_height, &bounds);
                
                // Apply margin offset
                let screen_x = margin + iso_x;
                let screen_y = margin + iso_y;
                
                // Use Z value to determine color/intensity
                let color = if z_range.abs() > f64::EPSILON {
                    let normalized_z = (z_val - z_min) / z_range;
                    if normalized_z < 0.2 { 1 } 
                    else if normalized_z < 0.4 { 2 }
                    else if normalized_z < 0.6 { 3 }
                    else if normalized_z < 0.8 { 4 }
                    else { 1 }
                } else {
                    1
                };
                
                // Draw a small point for each surface point
                for dx in 0..2 {
                    for dy in 0..2 {
                        let px = screen_x.saturating_add(dx).min(self.width - 1);
                        let py = screen_y.saturating_add(dy).min(self.height - 1);
                        self.set_pixel(px, py, color);
                    }
                }
            }
        }

        self.draw_3d_axes(margin, plot_width, plot_height, &bounds);

        self.add_surface3d_value_labels(
            Label3DConfig::new((margin, plot_width, plot_height), (x_min, x_max), (y_min, y_max), (z_min, z_max)),
        );
    }

    fn add_surface3d_value_labels(
        &mut self,
        config: Label3DConfig,
    ) {
        // X-axis labels
        for i in 0..=5 {
            let x = config.margin + (i * config.plot_width) / 5;
            let value = config.x_min + (i as f64 / 5.0) * (config.x_max - config.x_min);
            let text = format!("{value:.1}");
            let text_x = x.saturating_sub(text.len() * 3);
            self.render_text(&text, text_x, config.margin + config.plot_height + 5, 4);
        }

        // Y-axis labels
        for i in 0..=5 {
            let y = config.margin + (i * config.plot_height) / 5;
            let value = config.y_max - (i as f64 / 5.0) * (config.y_max - config.y_min);
            let text = format!("{value:.1}");
            self.render_text(&text, config.margin.saturating_sub(40), y.saturating_sub(3), 4);
        }

        // Z-range info
        let z_text = format!("Z:[{:.1}, {:.1}]", config.z_min, config.z_max);
        self.render_text(&z_text, config.margin + 5, config.margin - 15, 4);
    }

    fn add_parametric_value_labels(
        &mut self,
        config: LabelConfig,
    ) {
        for i in 0..=5 {
            let y = config.margin + (i * config.plot_height) / 5;
            let value = config.y_max - (i as f64 / 5.0) * (config.y_max - config.y_min);
            let text = format!("{value:.1}");
            self.render_text(&text, config.margin.saturating_sub(40), y.saturating_sub(3), 4);
        }

        for i in 0..=5 {
            let x = config.margin + (i * config.plot_width) / 5;
            let value = config.x_min + (i as f64 / 5.0) * (config.x_max - config.x_min);
            let text = format!("{value:.1}");
            let text_x = x.saturating_sub(text.len() * 3);
            self.render_text(&text, text_x, config.margin + config.plot_height + 5, 4);
        }
    }

    fn draw_line(&mut self, x0: usize, y0: usize, x1: usize, y1: usize, color: u8) {
        let dx = x1.abs_diff(x0);
        let dy = y1.abs_diff(y0);
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
        layout: (usize, usize, usize), // (margin, plot_width, plot_height)
        y_range: (f64, f64), // (min_val, max_val)
        x_range: (f64, f64), // (x_min, x_max)
    ) {
        let (margin, plot_width, plot_height) = layout;
        let (min_val, max_val) = y_range;
        let (x_min, x_max) = x_range;
        for i in 0..=5 {
            let y = margin + (i * plot_height) / 5;
            let value = max_val - (i as f64 / 5.0) * (max_val - min_val);
            let text = format!("{value:.1}");
            self.render_text(&text, margin.saturating_sub(40), y.saturating_sub(3), 4);
        }

        for i in 0..=5 {
            let x = margin + (i * plot_width) / 5;
            let value = x_min + (i as f64 / 5.0) * (x_max - x_min);
            let text = format!("{value:.1}");
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

    pub fn draw_3d_axes(&mut self, margin: usize, plot_width: usize, plot_height: usize, bounds: &SurfaceBounds) {
        // Draw simple, prominent 3D axes that are clearly visible
        
        // Start from the corner that will be most visible and draw axes outward
        let origin = Point3D::new(bounds.x_min, bounds.y_min, bounds.z_min);
        
        // Draw thick, obvious axes lines
        // X-axis - horizontal line in red
        let x_end = Point3D::new(bounds.x_max, bounds.y_min, bounds.z_min);
        self.draw_thick_3d_line(&origin, &x_end, bounds, (margin, plot_width, plot_height), 3);
        
        // Y-axis - depth line in green  
        let y_end = Point3D::new(bounds.x_min, bounds.y_max, bounds.z_min);
        self.draw_thick_3d_line(&origin, &y_end, bounds, (margin, plot_width, plot_height), 4);
        
        // Z-axis - vertical line in blue
        let z_end = Point3D::new(bounds.x_min, bounds.y_min, bounds.z_max);
        self.draw_thick_3d_line(&origin, &z_end, bounds, (margin, plot_width, plot_height), 2);
    }
    
    fn draw_thick_3d_line(&mut self, start: &Point3D, end: &Point3D, bounds: &SurfaceBounds, 
                         layout: (usize, usize, usize), color: u8) {
        let (margin, plot_width, plot_height) = layout;
        let (start_x, start_y) = start.to_isometric(plot_width, plot_height, bounds);
        let (end_x, end_y) = end.to_isometric(plot_width, plot_height, bounds);
        
        // Draw a thick line by drawing multiple parallel lines
        for offset_x in -1..=1 {
            for offset_y in -1..=1 {
                let adj_start_x = (margin as i32 + start_x as i32 + offset_x).max(0) as usize;
                let adj_start_y = (margin as i32 + start_y as i32 + offset_y).max(0) as usize;
                let adj_end_x = (margin as i32 + end_x as i32 + offset_x).max(0) as usize;
                let adj_end_y = (margin as i32 + end_y as i32 + offset_y).max(0) as usize;
                
                if adj_start_x < self.width && adj_start_y < self.height &&
                   adj_end_x < self.width && adj_end_y < self.height {
                    self.draw_line(adj_start_x, adj_start_y, adj_end_x, adj_end_y, color);
                }
            }
        }
    }
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