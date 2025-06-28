use ab_glyph::{Font, FontArc, PxScale};
use anyhow::{Result, anyhow};

#[allow(dead_code)]
pub struct Model {
    font: FontArc,
}

impl Model {
    pub fn from_vec(data: Vec<u8>) -> Result<Self> {
        let font = FontArc::try_from_vec(data)?;
        Ok(Model { font })
    }

    pub fn from_bytes(data: &[u8]) -> Result<Self> {
        Self::from_vec(data.to_vec())
    }

    #[allow(dead_code)]
    fn glyph_luminance(&self, character: char, scale: PxScale) -> Result<f32> {
        let glyph = self.font.glyph_id(character).with_scale(scale);
        let outlined_glyph = match self.font.outline_glyph(glyph) {
            Some(g) => g,
            None => {
                return Err(anyhow!(
                    "Failed to outline glyph for character: {}",
                    character
                ));
            }
        };

        let bounds = outlined_glyph.px_bounds();
        let (width, height) = (bounds.width().ceil() as u32, bounds.height().ceil() as u32);

        if width == 0 || height == 0 {
            return Ok(0.0);
        }

        let mut total_luminance = 0.0;
        outlined_glyph.draw(|_, _, c| {
            total_luminance += c;
        });

        let average_luminance = total_luminance / (width * height) as f32;
        Ok(average_luminance)
    }

    pub fn correlation(x_values: &[f64], y_values: &[f64]) -> Option<f64> {
        if x_values.len() != y_values.len() || x_values.is_empty() || y_values.is_empty() {
            return None;
        }

        let n = x_values.len();
        let mean_x = x_values.iter().sum::<f64>() / n as f64;
        let mean_y = y_values.iter().sum::<f64>() / n as f64;
        let (numerator, denom_x, denom_y) =
            x_values
                .iter()
                .zip(y_values)
                .fold((0.0, 0.0, 0.0), |(num, den_x, den_y), (&x, &y)| {
                    let diff_x = x - mean_x;
                    let diff_y = y - mean_y;
                    (
                        num + diff_x * diff_y,
                        den_x + diff_x.powi(2),
                        den_y + diff_y.powi(2),
                    )
                });

        let denominator = denom_x.sqrt() * denom_y.sqrt();
        if denominator == 0.0 {
            return None;
        }

        Some(numerator / denominator)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    const FONT_PATH: &str = "fonts/NotoSansJP-Regular.otf";

    #[test]
    fn model_from_vec() {
        let font_data = fs::read(FONT_PATH).unwrap();
        let model = Model::from_vec(font_data);
        assert!(model.is_ok());
    }

    #[test]
    fn glyph_luminance() {
        let font_data = fs::read(FONT_PATH).unwrap();
        let model = Model::from_bytes(&font_data).unwrap();
        let scale = PxScale::from(16.0);
        let result = model.glyph_luminance('A', scale);
        assert!(result.is_ok());
        let luminance = result.unwrap();
        assert!(luminance >= 0.0 && luminance <= 1.0);
    }

    #[test]
    fn correlation_different_lengths_returns_none() {
        assert_eq!(Model::correlation(&[1.0], &[1.0, 2.0]), None);
    }

    #[test]
    fn correlation_empty_slices_returns_none() {
        assert_eq!(Model::correlation(&[], &[]), None);
    }

    #[test]
    fn correlation_valid_data_returns_none() {
        assert_eq!(Model::correlation(&[1.0, 2.0], &[5.0, 5.0]), None,);
    }

    #[test]
    fn correlation_valid_data_returns_some() {
        let x_values = [1.0, 2.0, 3.0];
        let y_values = [4.0, 5.0, 6.0];
        let result = Model::correlation(&x_values, &y_values);
        assert!(result.is_some());
        assert!((result.unwrap() - 1.0).abs() < 1e-9);
    }
}
