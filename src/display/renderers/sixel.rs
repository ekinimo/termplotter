use crate::{
    values::{ExpressionRange1dResult, Expression3dResult},
    parametric2d::Parametric2DResult,
};

use super::super::{DisplayRenderer, Bitmap};

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

fn bitmap_to_sixel(bitmap: &Bitmap, result: &ExpressionRange1dResult) -> String {
    let mut output = String::new();

    output.push_str("\x1bPq");
    output.push_str("\"1;1;");
    output.push_str(&format!("{};{}", bitmap.width, bitmap.height));
    output.push('\n');

    //Colormap
    output.push_str("#0;2;0;0;0");
    output.push_str("#1;2;0;100;0");
    output.push_str("#2;2;25;25;25");
    output.push_str("#3;2;100;100;0");
    output.push_str("#4;2;75;75;75");

    for row_chunk in (0..bitmap.height).step_by(6) {
        for color in 1..=4 {
            output.push_str(&format!("#{color}"));

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
                            output.push_str(&format!("!{repeat_count}{prev_char}"));
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
                    output.push_str(&format!("!{repeat_count}{prev_char}"));
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