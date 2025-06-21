use std::{error::Error, io::Write};

use crate::{
    values::{ExpressionRange1dResult, Expression3dResult},
    parametric2d::Parametric2DResult,
};

use super::super::{OutputWriter, Point3D, SurfaceBounds};

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
            r#"<svg width="{width}" height="{height}" xmlns="http://www.w3.org/2000/svg">"#
        )?;
        writeln!(
            file,
            r#"<rect width="{width}" height="{height}" fill="black"/>"#
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
                        path_data.push_str(&format!("M {x_svg} {y_svg}"));
                    } else {
                        path_data.push_str(&format!(" L {x_svg} {y_svg}"));
                    }

                    // Draw data points
                    writeln!(
                        file,
                        r#"<circle cx="{x_svg}" cy="{y_svg}" r="2" fill="cyan"/>"#
                    )?;
                }
                writeln!(
                    file,
                    r#"<path d="{path_data}" fill="none" stroke="cyan" stroke-width="2"/>"#
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
            r#"<svg width="{width}" height="{height}" xmlns="http://www.w3.org/2000/svg">"#
        )?;
        writeln!(
            file,
            r#"<rect width="{width}" height="{height}" fill="black"/>"#
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
                        path_data.push_str(&format!("M {x_svg} {y_svg}"));
                    } else {
                        path_data.push_str(&format!(" L {x_svg} {y_svg}"));
                    }

                    writeln!(
                        file,
                        r#"<circle cx="{x_svg}" cy="{y_svg}" r="2" fill="cyan"/>"#
                    )?;
                }
                writeln!(
                    file,
                    r#"<path d="{path_data}" fill="none" stroke="cyan" stroke-width="2"/>"#
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
            r#"<svg width="{width}" height="{height}" xmlns="http://www.w3.org/2000/svg">"#
        )?;
        writeln!(
            file,
            r#"<rect width="{width}" height="{height}" fill="black"/>"#
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

            // Create isometric 3D projection using proper 3D coordinates
            if x_range > f64::EPSILON && y_range > f64::EPSILON {
                let bounds = SurfaceBounds::from_surface(result);
                
                for (y_idx, &y_val) in result.y_values.iter().enumerate() {
                    for (x_idx, &x_val) in result.x_values.iter().enumerate() {
                        if let Some(z_val) = result.get_z(x_idx, y_idx) {
                            // Create 3D point and project to isometric coordinates
                            let point_3d = Point3D::new(x_val, y_val, z_val);
                            let (iso_x, iso_y) = point_3d.to_isometric(plot_width, plot_height, &bounds);
                            
                            // Apply margin offset
                            let x_svg = margin + iso_x;
                            let y_svg = margin + iso_y;
                            
                            // Color intensity based on z value
                            let intensity = if z_range > f64::EPSILON {
                                ((z_val - z_min) / z_range * 255.0) as u8
                            } else {
                                128
                            };
                            
                            writeln!(
                                file,
                                r#"<rect x="{}" y="{}" width="2" height="2" fill="rgb({},{},{})" opacity="0.8"/>"#,
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