use crate::{
    values::{ExpressionRange1dResult, Expression3dResult},
    parametric2d::Parametric2DResult,
};

use super::super::{DisplayRenderer, Point3D, SurfaceBounds};

#[derive(Clone, Debug)]
pub struct RegisRenderer;

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

fn regis_init(width: usize, height: usize) -> String {
    let mut init = format!("\x1bP0p\nS(A[0,0][{width},{height}])\nS(E)\n");
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
        output.push_str(&format!("P[{x},0]\nV[{x},{height}]\n"));
    }
    for i in 1..8 {
        let y = (i * height) / 8;
        output.push_str(&format!("P[0,{y}]\nV[{width},{y}]\n"));
    }

    // Draw axes
    output.push_str("W(P0)\nS(C1)\n");
    let x_axis_y = if plot_y_min <= 0.0 && plot_y_max >= 0.0 {
        data_to_screen_y(0.0)
    } else {
        height - 1
    };
    output.push_str(&format!("P[0,{x_axis_y}]\nV[{width},{x_axis_y}]\n"));
    let y_axis_x = if x_min <= 0.0 && x_max >= 0.0 {
        data_to_screen_x(0.0)
    } else {
        0
    };
    output.push_str(&format!("P[{y_axis_x},0]\nV[{y_axis_x},{height}]\n"));

    // Add axis labels
    output.push_str("W(P2)\nS(C1)\n");

    // X-axis labels
    for i in 0..=5 {
        let x_screen = (i * width) / 5;
        let x_data = x_min + (i as f64 / 5.0) * x_range_val;
        let label_y = (x_axis_y + 20).min(height - 10);

        let label = if x_data.abs() < 0.01 {
            "0".to_string()
        } else if x_data.abs() >= 1000.0 || x_data.fract() == 0.0 {
            format!("{x_data:.0}")
        } else {
            format!("{x_data:.1}")
        };

        let text_x = x_screen.saturating_sub(label.len() * 3);
        output.push_str(&format!("P[{text_x},{label_y}]\nT(S1)'{label}'\n"));
    }

    // Y-axis labels
    for i in 0..=5 {
        let y_screen = (i * height) / 5;
        let y_data = plot_y_max - (i as f64 / 5.0) * plot_y_range;
        let label_x = y_axis_x.saturating_sub(30).max(5);

        let label = if y_data.abs() < 0.01 {
            "0".to_string()
        } else if y_data.abs() >= 1000.0 {
            format!("{y_data:.0}")
        } else {
            format!("{y_data:.2}")
        };

        let text_y = y_screen.saturating_sub(5);
        output.push_str(&format!("P[{label_x},{text_y}]\nT(S1)'{label}'\n"));
    }

    // Add Info
    output.push_str(&format!("P[5,15]\nT(S2)'Max: {y_max:.2}'\n"));
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
        output.push_str(&format!("P[{x0},{y0}]\n"));

        for (i, &y_val) in result.0.iter().enumerate().skip(1) {
            let x = data_to_screen_x(i);
            let y = data_to_screen_y(y_val);
            output.push_str(&format!("V[{x},{y}]\n"));
        }

        let x0 = data_to_screen_x(0);
        let y0 = data_to_screen_y(result.0[0])
            .saturating_add(1)
            .min(height - 1);
        output.push_str(&format!("P[{x0},{y0}]\n"));

        for (i, &y_val) in result.0.iter().enumerate().skip(1) {
            let x = data_to_screen_x(i);
            let y = data_to_screen_y(y_val).saturating_add(1).min(height - 1);
            output.push_str(&format!("V[{x},{y}]\n"));
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
                output.push_str(&format!("P[{px},{py}]\nV[{px},{py}]\n"));
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
        output.push_str(&format!("P[{x},0]\nV[{x},{height}]\n"));
    }
    for i in 1..8 {
        let y = (i * height) / 8;
        output.push_str(&format!("P[0,{y}]\nV[{width},{y}]\n"));
    }

    // Draw axes
    output.push_str("W(P0)\nS(C1)\n");
    
    // X-axis (horizontal line at y=0 if 0 is in range)
    let x_axis_y = if plot_y_min <= 0.0 && plot_y_max >= 0.0 {
        data_to_screen_y(0.0)
    } else {
        height - 1
    };
    output.push_str(&format!("P[0,{x_axis_y}]\nV[{width},{x_axis_y}]\n"));
    
    // Y-axis (vertical line at x=0 if 0 is in range)
    let y_axis_x = if plot_x_min <= 0.0 && plot_x_max >= 0.0 {
        data_to_screen_x(0.0)
    } else {
        0
    };
    output.push_str(&format!("P[{y_axis_x},0]\nV[{y_axis_x},{height}]\n"));

    // Add axis labels
    output.push_str("W(P2)\nS(C1)\n");

    // X-axis labels
    for i in 0..=5 {
        let x_screen = (i * width) / 5;
        let x_data = plot_x_min + (i as f64 / 5.0) * plot_x_range;
        let label_y = (x_axis_y + 20).min(height - 10);

        let label = if x_data.abs() < 0.01 {
            "0".to_string()
        } else if x_data.abs() >= 1000.0 || x_data.fract() == 0.0 {
            format!("{x_data:.0}")
        } else {
            format!("{x_data:.1}")
        };

        let text_x = x_screen.saturating_sub(label.len() * 3);
        output.push_str(&format!("P[{text_x},{label_y}]\nT(S1)'{label}'\n"));
    }

    // Y-axis labels
    for i in 0..=5 {
        let y_screen = (i * height) / 5;
        let y_data = plot_y_max - (i as f64 / 5.0) * plot_y_range;
        let label_x = y_axis_x.saturating_sub(30).max(5);

        let label = if y_data.abs() < 0.01 {
            "0".to_string()
        } else if y_data.abs() >= 1000.0 {
            format!("{y_data:.0}")
        } else {
            format!("{y_data:.2}")
        };

        let text_y = y_screen.saturating_sub(5);
        output.push_str(&format!("P[{label_x},{text_y}]\nT(S1)'{label}'\n"));
    }

    // Add plot info
    output.push_str(&format!("P[5,15]\nT(S2)'Parametric Plot: {} points'\n", result.len()));
    output.push_str(&format!("P[5,30]\nT(S2)'X: {x_min:.2} to {x_max:.2}'\n"));
    output.push_str(&format!("P[5,45]\nT(S2)'Y: {y_min:.2} to {y_max:.2}'\n"));

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
        output.push_str(&format!("P[{x0},{y0}]\n"));

        for (&x_val, &y_val) in result.x_values.iter().zip(result.y_values.iter()).skip(1) {
            let x = data_to_screen_x(x_val);
            let y = data_to_screen_y(y_val);
            output.push_str(&format!("V[{x},{y}]\n"));
        }
    }

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
    output.push_str("W(P3)\nS(C2)\n");
    for i in 1..10 {
        let x = (i * width) / 10;
        output.push_str(&format!("P[{x},0]\nV[{x},{height}]\n"));
    }
    for i in 1..8 {
        let y = (i * height) / 8;
        output.push_str(&format!("P[0,{y}]\nV[{width},{y}]\n"));
    }

    // Draw axes
    output.push_str("W(P0)\nS(C1)\n");
    output.push_str(&format!("P[0,{}]\nV[{},{}]\n", height/2, width, height/2));
    output.push_str(&format!("P[{},0]\nV[{},{}]\n", width/2, width/2, height));

    // Add axis labels
    output.push_str("W(P2)\nS(C1)\n");
    output.push_str(&format!("P[5,15]\nT(S2)'3D Surface: {}x{} points'\n", result.x_len(), result.y_len()));
    output.push_str(&format!("P[5,30]\nT(S2)'X: {x_min:.2} to {x_max:.2}'\n"));
    output.push_str(&format!("P[5,45]\nT(S2)'Y: {y_min:.2} to {y_max:.2}'\n"));
    output.push_str(&format!("P[5,60]\nT(S2)'Z: {z_min:.2} to {z_max:.2}'\n"));

    output
}

fn regis_surface3d_plot(result: &Expression3dResult, width: usize, height: usize) -> String {
    if result.is_empty() {
        return String::new();
    }

    let mut output = String::new();
    output.push_str("W(P1)\nS(C3)\n");

    let z_min = result.z_min();
    let z_max = result.z_max();
    let z_range = z_max - z_min;

    if z_range.abs() < f64::EPSILON {
        return output;
    }

    // Create isometric wireframe representation
    let bounds = SurfaceBounds::from_surface(result);
    
    for (y_idx, z_row) in result.data.iter().enumerate() {
        for (x_idx, &z_val) in z_row.iter().enumerate() {
            // Get the actual 3D coordinates
            let x_val = result.x_values[x_idx];
            let y_val = result.y_values[y_idx];
            
            // Create 3D point and project to isometric coordinates
            let point_3d = Point3D::new(x_val, y_val, z_val);
            let (iso_x, iso_y) = point_3d.to_isometric(width, height, &bounds);
            
            // Draw a small point for the surface
            output.push_str(&format!("P[{},{}]\nV[{},{}]\n", iso_x, iso_y, iso_x + 1, iso_y + 1));
        }
    }

    output
}