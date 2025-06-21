use std::{error::Error, io::Write};

use crate::{
    values::{ExpressionRange1dResult, Expression3dResult},
    parametric2d::Parametric2DResult,
};

use super::super::OutputWriter;

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
            writeln!(file, "    ({x_val:.6}, {y_val:.6})")?;
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

        writeln!(file, r"    xmin={x_min:.3}, xmax={x_max:.3},")?;
        writeln!(file, r"    ymin={y_min:.3}, ymax={y_max:.3},")?;
        writeln!(file, r"    grid=major")?;
        writeln!(file, r"]")?;

        writeln!(
            file,
            r"\addplot[red, mark=*, mark size=1pt] coordinates {{"
        )?;
        for (&x_val, &y_val) in result.x_values.iter().zip(result.y_values.iter()) {
            writeln!(file, "    ({x_val:.6}, {y_val:.6})")?;
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
                    writeln!(file, "    ({x_val:.6}, {y_val:.6}, {z_val:.6})")?;
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