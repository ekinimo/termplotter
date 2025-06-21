use crate::{
    values::{ExpressionRange1dResult, Expression3dResult},
    parametric2d::Parametric2DResult,
};

use super::super::{DisplayRenderer, Point3D, SurfaceBounds, PlotConfig, Plot3DConfig};

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
            PlotConfig::new(width, height, result.0.len(), x_min, x_max, min_val, max_val),
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
                "Parametric plot: X=[{x_min:.3}, {x_max:.3}], Y=[{y_min:.3}, {y_max:.3}] (constant values)"
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
            return format!("3D Surface: constant Z = {z_min:.3}");
        }
        
        // Create isometric projection for ASCII 3D visualization
        let bounds = SurfaceBounds::from_surface(result);
        let data_width = width.saturating_sub(6);
        let data_height = height.saturating_sub(4);
        
        for (y_idx, z_row) in result.data.iter().enumerate() {
            for (x_idx, &z_val) in z_row.iter().enumerate() {
                // Get the actual 3D coordinates
                let x_val = result.x_values[x_idx];
                let y_val = result.y_values[y_idx];
                
                // Create 3D point and project to isometric coordinates
                let point_3d = Point3D::new(x_val, y_val, z_val);
                let (iso_x, iso_y) = point_3d.to_isometric(data_width, data_height, &bounds);
                
                // Apply margin offset
                let screen_x = 5 + iso_x;
                let screen_y = 2 + iso_y;
                
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
        
        // Draw 3D axes
        draw_ascii_3d_axes(&mut grid, width, height, &bounds);
        
        format_ascii_surface3d_output(
            grid,
            Plot3DConfig::new(
                (width, height),
                result.x_len() * result.y_len(),
                (result.x_min(), result.x_max()),
                (result.y_min(), result.y_max()),
                (z_min, z_max),
            ),
        )
    }
}

fn add_ascii_axes(
    grid: &mut [Vec<char>],
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
    for row in grid.iter_mut().take(height) {
        if row[5] == ' ' {
            row[5] = '|';
        }
    }

    // Y-axis labels
    for i in 0..5 {
        let y = i * (height - 1) / 4;
        let value = plot_max - (i as f64 / 4.0) * (plot_max - plot_min);
        let label = format!("{value:4.1}");
        for (j, ch) in label.chars().enumerate() {
            if j < 4 && y < height {
                grid[y][j] = ch;
            }
        }
    }
}

fn format_ascii_output(
    grid: Vec<Vec<char>>,
    config: PlotConfig,
    x_range: &ExpressionRange1dResult,
) -> String {
    let mut output = format!(
        "┌─ ASCII Plot: {} points, X:[{:.2}, {:.2}], Y:[{:.2}, {:.2}] ─┐\n",
        config.data_points, config.x_min, config.x_max, config.y_min, config.y_max
    );

    for row in grid {
        output.push('│');
        output.push_str(&row.into_iter().collect::<String>());
        output.push_str("│\n");
    }

    output.push('└');
    output.push_str(&"─".repeat(config.width + 1));
    output.push_str("┘\n");

    // X-axis labels
    output.push_str("X: ");
    let num_markers = 5.min(config.data_points);
    let data_width = config.width.saturating_sub(6);

    for i in 0..num_markers {
        let x_index = if num_markers > 1 {
            i * (config.data_points - 1) / (num_markers - 1)
        } else {
            0
        };

        let marker_pos = if num_markers > 1 {
            5 + (i * data_width / (num_markers - 1))
        } else {
            config.width / 2
        };

        let x_value = if x_index < x_range.0.len() {
            x_range.0[x_index]
        } else {
            config.x_max
        };

        if i == 0 {
            output.push_str(&" ".repeat(marker_pos.saturating_sub(3)));
        } else {
            let prev_pos = if num_markers > 1 {
                5 + ((i - 1) * data_width / (num_markers - 1))
            } else {
                config.width / 2
            };
            let spacing = marker_pos.saturating_sub(prev_pos).saturating_sub(4);
            output.push_str(&" ".repeat(spacing));
        }

        output.push_str(&format!("{x_value:.1}"));
    }
    output.push('\n');

    output
}

fn add_ascii_parametric_axes(
    grid: &mut [Vec<char>],
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
            for row in grid.iter_mut().take(height) {
                if row[zero_x] == ' ' {
                    row[zero_x] = '|';
                }
            }
        }
    }

    // Y-axis labels on the left
    for i in 0..5 {
        let y = i * (height - 1) / 4;
        let value = plot_y_max - (i as f64 / 4.0) * (plot_y_max - plot_y_min);
        let label = format!("{value:4.1}");
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
        "┌─ ASCII Parametric Plot: {data_points} points, X:[{x_min:.2}, {x_max:.2}], Y:[{y_min:.2}, {y_max:.2}] ─┐\n"
    );

    for row in grid {
        output.push('│');
        output.push_str(&row.into_iter().collect::<String>());
        output.push_str("│\n");
    }

    output.push('└');
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
        output.push_str(&format!("{x_value:.1}"));
    }
    output.push('\n');

    output
}

fn format_ascii_surface3d_output(
    grid: Vec<Vec<char>>,
    config: Plot3DConfig,
) -> String {
    let mut output = format!(
        "┌─ ASCII 3D Surface: {} points, X:[{:.2}, {:.2}], Y:[{:.2}, {:.2}], Z:[{:.2}, {:.2}] ─┐\n",
        config.data_points, config.x_min, config.x_max, config.y_min, config.y_max, config.z_min, config.z_max
    );

    for row in grid {
        output.push('│');
        output.push_str(&row.into_iter().collect::<String>());
        output.push_str("│\n");
    }

    output.push('└');
    output.push_str(&"─".repeat(config.width + 1));
    output.push_str("┘\n");
    output.push_str("Legend: . : + * # (low to high Z)\n");

    output
}

fn draw_ascii_3d_axes(grid: &mut [Vec<char>], width: usize, height: usize, bounds: &SurfaceBounds) {
    let data_width = width.saturating_sub(6);
    let data_height = height.saturating_sub(4);
    
    // Draw a 3D coordinate box with ASCII characters
    
    // Define the 8 corners of the 3D box
    let corners = [
        Point3D::new(bounds.x_min, bounds.y_min, bounds.z_min), // 0: origin
        Point3D::new(bounds.x_max, bounds.y_min, bounds.z_min), // 1: +X
        Point3D::new(bounds.x_min, bounds.y_max, bounds.z_min), // 2: +Y  
        Point3D::new(bounds.x_max, bounds.y_max, bounds.z_min), // 3: +X+Y
        Point3D::new(bounds.x_min, bounds.y_min, bounds.z_max), // 4: +Z
        Point3D::new(bounds.x_max, bounds.y_min, bounds.z_max), // 5: +X+Z
        Point3D::new(bounds.x_min, bounds.y_max, bounds.z_max), // 6: +Y+Z
        Point3D::new(bounds.x_max, bounds.y_max, bounds.z_max), // 7: +X+Y+Z
    ];
    
    // Draw the wireframe box with '.' for lighter lines
    // Bottom face
    draw_ascii_3d_line(grid, &corners[0], &corners[1], bounds, data_width, data_height, '.');
    draw_ascii_3d_line(grid, &corners[1], &corners[3], bounds, data_width, data_height, '.');
    draw_ascii_3d_line(grid, &corners[3], &corners[2], bounds, data_width, data_height, '.');
    draw_ascii_3d_line(grid, &corners[2], &corners[0], bounds, data_width, data_height, '.');
    
    // Top face
    draw_ascii_3d_line(grid, &corners[4], &corners[5], bounds, data_width, data_height, '.');
    draw_ascii_3d_line(grid, &corners[5], &corners[7], bounds, data_width, data_height, '.');
    draw_ascii_3d_line(grid, &corners[7], &corners[6], bounds, data_width, data_height, '.');
    draw_ascii_3d_line(grid, &corners[6], &corners[4], bounds, data_width, data_height, '.');
    
    // Vertical edges
    draw_ascii_3d_line(grid, &corners[0], &corners[4], bounds, data_width, data_height, '.');
    draw_ascii_3d_line(grid, &corners[1], &corners[5], bounds, data_width, data_height, '.');
    draw_ascii_3d_line(grid, &corners[2], &corners[6], bounds, data_width, data_height, '.');
    draw_ascii_3d_line(grid, &corners[3], &corners[7], bounds, data_width, data_height, '.');
    
    // Draw main coordinate axes with distinct characters
    // X-axis - use '=' for emphasis
    draw_ascii_3d_line(grid, &corners[0], &corners[1], bounds, data_width, data_height, '=');
    
    // Y-axis - use '|' character  
    draw_ascii_3d_line(grid, &corners[0], &corners[2], bounds, data_width, data_height, '|');
    
    // Z-axis - use '^' character for upward direction
    draw_ascii_3d_line(grid, &corners[0], &corners[4], bounds, data_width, data_height, '^');
}

fn draw_ascii_3d_line(grid: &mut [Vec<char>], start: &Point3D, end: &Point3D, bounds: &SurfaceBounds,
                      plot_width: usize, plot_height: usize, axis_char: char) {
    let (start_x, start_y) = start.to_isometric(plot_width, plot_height, bounds);
    let (end_x, end_y) = end.to_isometric(plot_width, plot_height, bounds);
    
    // Apply margin offset
    let start_screen_x = 5 + start_x;
    let start_screen_y = 2 + start_y;
    let end_screen_x = 5 + end_x;
    let end_screen_y = 2 + end_y;
    
    // Simple line drawing using Bresenham-like algorithm
    let dx = end_screen_x.abs_diff(start_screen_x);
    let dy = end_screen_y.abs_diff(start_screen_y);
    let x_step = if start_screen_x < end_screen_x { 1i32 } else { -1i32 };
    let y_step = if start_screen_y < end_screen_y { 1i32 } else { -1i32 };

    let mut error = dx as i32 - dy as i32;
    let mut x = start_screen_x as i32;
    let mut y = start_screen_y as i32;

    loop {
        if x >= 0 && x < grid[0].len() as i32 && y >= 0 && y < grid.len() as i32 {
            grid[y as usize][x as usize] = axis_char;
        }

        if x == end_screen_x as i32 && y == end_screen_y as i32 {
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