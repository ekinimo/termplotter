use crate::{
    values::{ExpressionRange1dResult, Expression3dResult},
    parametric2d::Parametric2DResult,
};

use super::super::{DisplayRenderer, Point3D, SurfaceBounds, PlotConfig, Plot3DConfig, PlotBounds, Plot3DStyle};

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
                "\x1b[33mParametric plot: X=[{x_min:.3}, {x_max:.3}], Y=[{y_min:.3}, {y_max:.3}] (constant values)\x1b[0m"
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
        add_ansi_parametric_axes(&mut grid, &mut colors, width, height, PlotBounds::new(plot_x_min, plot_x_max, plot_y_min, plot_y_max));
        
        format_ansi_parametric_output(
            grid,
            colors,
            PlotConfig::new(width, height, result.len(), x_min, x_max, y_min, y_max),
        )
    }

    fn render_surface3d(
        &self,
        result: &Expression3dResult,
        width: usize,
        height: usize,
    ) -> String {
        if result.is_empty() {
            return "\x1b[31mNo 3D surface data to plot\x1b[0m".to_string();
        }

        let mut grid = vec![vec![' '; width]; height];
        let mut colors = vec![vec![0u8; width]; height];
        
        // Create a colorized ANSI 3D projection
        let z_min = result.z_min();
        let z_max = result.z_max();
        let z_range = z_max - z_min;
        
        if z_range.abs() < f64::EPSILON {
            return format!("\x1b[33m3D Surface: constant Z = {z_min:.3}\x1b[0m");
        }
        
        // Create isometric projection for ANSI 3D visualization
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
        
        // Draw 3D axes
        draw_ansi_3d_axes(&mut grid, &mut colors, width, height, &bounds);
        
        format_ansi_surface3d_output(
            grid,
            colors,
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

fn add_ansi_axes(
    grid: &mut [Vec<char>],
    colors: &mut [Vec<u8>],
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
        let label = format!("{value:4.1}");
        for (j, ch) in label.chars().enumerate() {
            if j < 4 && y < height {
                grid[y][j] = ch;
                colors[y][j] = 2;
            }
        }
    }
}

fn format_ansi_output(
    grid: Vec<Vec<char>>,
    colors: Vec<Vec<u8>>,
    config: PlotConfig,
    x_range: &ExpressionRange1dResult,
) -> String {
    let mut output = format!(
        "\x1b[36m┌─ ANSI Plot: {} points, X:[{:.2}, {:.2}], Y:[{:.2}, {:.2}] ─┐\x1b[0m\n",
        config.data_points, config.x_min, config.x_max, config.y_min, config.y_max
    );

    for (row, color_row) in grid.into_iter().zip(colors.into_iter()) {
        output.push_str("\x1b[36m│\x1b[0m");
        for (ch, color) in row.into_iter().zip(color_row.into_iter()) {
            match color {
                1 => output.push_str(&format!("\x1b[92m{ch}\x1b[0m")), // Green data
                2 => output.push_str(&format!("\x1b[37m{ch}\x1b[0m")), // White axes
                3 => output.push_str(&format!("\x1b[93m{ch}\x1b[0m")), // Yellow zero line
                _ => output.push(ch),
            }
        }
        output.push_str("\x1b[36m│\x1b[0m\n");
    }

    output.push_str("\x1b[36m└");
    output.push_str(&"─".repeat(config.width + 1));
    output.push_str("┘\x1b[0m\n");

    // X-axis labels
    output.push_str("\x1b[37mX: \x1b[0m");
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

        output.push_str(&format!("\x1b[93m{x_value:.1}\x1b[0m"));
    }
    output.push('\n');

    output
}

fn add_ansi_parametric_axes(
    grid: &mut [Vec<char>],
    colors: &mut [Vec<u8>],
    width: usize,
    height: usize,
    bounds: PlotBounds,
) {
    // X-axis (check if 0 is in range)
    if bounds.y_min <= 0.0 && bounds.y_max >= 0.0 {
        let plot_y_range = bounds.y_max - bounds.y_min;
        let zero_y = ((bounds.y_max - 0.0) / plot_y_range * (height - 1) as f64) as usize;
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
    if bounds.x_min <= 0.0 && bounds.x_max >= 0.0 {
        let plot_x_range = bounds.x_max - bounds.x_min;
        let zero_x = 5 + ((0.0 - bounds.x_min) / plot_x_range * (width - 6) as f64) as usize;
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
        let value = bounds.y_max - (i as f64 / 4.0) * (bounds.y_max - bounds.y_min);
        let label = format!("{value:4.1}");
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
    config: PlotConfig,
) -> String {
    let mut output = format!(
        "\x1b[36m┌─ ANSI Parametric Plot: {} points, X:[{:.2}, {:.2}], Y:[{:.2}, {:.2}] ─┐\x1b[0m\n",
        config.data_points, config.x_min, config.x_max, config.y_min, config.y_max
    );

    for (row, color_row) in grid.into_iter().zip(colors.into_iter()) {
        output.push_str("\x1b[36m│\x1b[0m");
        for (ch, color) in row.into_iter().zip(color_row.into_iter()) {
            match color {
                1 => output.push_str(&format!("\x1b[92m{ch}\x1b[0m")), // Green data
                2 => output.push_str(&format!("\x1b[37m{ch}\x1b[0m")), // White axes
                3 => output.push_str(&format!("\x1b[93m{ch}\x1b[0m")), // Yellow zero line
                _ => output.push(ch),
            }
        }
        output.push_str("\x1b[36m│\x1b[0m\n");
    }

    output.push_str("\x1b[36m└");
    output.push_str(&"─".repeat(config.width + 1));
    output.push_str("┘\x1b[0m\n");

    // X-axis labels
    output.push_str("\x1b[37mX: \x1b[0m");
    for i in 0..5 {
        let x_value = config.x_min + (i as f64 / 4.0) * (config.x_max - config.x_min);
        if i == 0 {
            output.push_str("  ");
        } else {
            output.push_str("              ");
        }
        output.push_str(&format!("\x1b[93m{x_value:.1}\x1b[0m"));
    }
    output.push('\n');

    output
}

fn format_ansi_surface3d_output(
    grid: Vec<Vec<char>>,
    colors: Vec<Vec<u8>>,
    config: Plot3DConfig,
) -> String {
    let mut output = format!(
        "\x1b[36m┌─ ANSI 3D Surface: {} points, X:[{:.2}, {:.2}], Y:[{:.2}, {:.2}], Z:[{:.2}, {:.2}] ─┐\x1b[0m\n",
        config.data_points, config.x_min, config.x_max, config.y_min, config.y_max, config.z_min, config.z_max
    );

    for (row, color_row) in grid.into_iter().zip(colors.into_iter()) {
        output.push_str("\x1b[36m│\x1b[0m");
        for (ch, color) in row.into_iter().zip(color_row.into_iter()) {
            match color {
                1 => output.push_str(&format!("\x1b[92m{ch}\x1b[0m")), // Green
                2 => output.push_str(&format!("\x1b[94m{ch}\x1b[0m")), // Blue
                3 => output.push_str(&format!("\x1b[93m{ch}\x1b[0m")), // Yellow
                4 => output.push_str(&format!("\x1b[90m{ch}\x1b[0m")), // Dim
                5 => output.push_str(&format!("\x1b[91m{ch}\x1b[0m")), // Red
                6 => output.push_str(&format!("\x1b[91m{ch}\x1b[0m")), // Red X-axis
                7 => output.push_str(&format!("\x1b[92m{ch}\x1b[0m")), // Green Y-axis
                8 => output.push_str(&format!("\x1b[96m{ch}\x1b[0m")), // Cyan Z-axis
                _ => output.push(ch),
            }
        }
        output.push_str("\x1b[36m│\x1b[0m\n");
    }

    output.push_str("\x1b[36m└");
    output.push_str(&"─".repeat(config.width + 1));
    output.push_str("\x1b[36m┘\x1b[0m\n");

    output
}

fn draw_ansi_3d_axes(grid: &mut [Vec<char>], colors: &mut [Vec<u8>], width: usize, height: usize, bounds: &SurfaceBounds) {
    let data_width = width.saturating_sub(6);
    let data_height = height.saturating_sub(4);
    
    // Draw a 3D coordinate box with colored ANSI characters
    
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
    
    let box_color = 2; // Gray for wireframe
    let box_style = Plot3DStyle::new(data_width, data_height, '·', box_color);
    
    // Draw the wireframe box
    // Bottom face
    draw_ansi_3d_line(grid, colors, &corners[0], &corners[1], bounds, Plot3DStyle::new(data_width, data_height, '·', box_color));
    draw_ansi_3d_line(grid, colors, &corners[1], &corners[3], bounds, box_style);
    draw_ansi_3d_line(grid, colors, &corners[3], &corners[2], bounds, box_style);
    draw_ansi_3d_line(grid, colors, &corners[2], &corners[0], bounds, box_style);
    
    // Top face
    draw_ansi_3d_line(grid, colors, &corners[4], &corners[5], bounds, box_style);
    draw_ansi_3d_line(grid, colors, &corners[5], &corners[7], bounds, box_style);
    draw_ansi_3d_line(grid, colors, &corners[7], &corners[6], bounds, box_style);
    draw_ansi_3d_line(grid, colors, &corners[6], &corners[4], bounds, box_style);
    
    // Vertical edges
    draw_ansi_3d_line(grid, colors, &corners[0], &corners[4], bounds, box_style);
    draw_ansi_3d_line(grid, colors, &corners[1], &corners[5], bounds, box_style);
    draw_ansi_3d_line(grid, colors, &corners[2], &corners[6], bounds, box_style);
    draw_ansi_3d_line(grid, colors, &corners[3], &corners[7], bounds, box_style);
    
    // Draw main coordinate axes with distinct colored characters
    // X-axis - use '━' character with red color (6)
    draw_ansi_3d_line(grid, colors, &corners[0], &corners[1], bounds, Plot3DStyle::new(data_width, data_height, '━', 6));
    
    // Y-axis - use '┃' character with green color (7)
    draw_ansi_3d_line(grid, colors, &corners[0], &corners[2], bounds, Plot3DStyle::new(data_width, data_height, '┃', 7));
    
    // Z-axis - use '▲' character with cyan color (8) for upward direction
    draw_ansi_3d_line(grid, colors, &corners[0], &corners[4], bounds, Plot3DStyle::new(data_width, data_height, '▲', 8));
}

fn draw_ansi_3d_line(grid: &mut [Vec<char>], colors: &mut [Vec<u8>], start: &Point3D, end: &Point3D, bounds: &SurfaceBounds,
                     style: Plot3DStyle) {
    let (start_x, start_y) = start.to_isometric(style.plot_width, style.plot_height, bounds);
    let (end_x, end_y) = end.to_isometric(style.plot_width, style.plot_height, bounds);
    
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
            grid[y as usize][x as usize] = style.axis_char;
            colors[y as usize][x as usize] = style.axis_color;
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