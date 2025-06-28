use ab_glyph::{Font, FontArc, PxScale};
use anyhow::{Result, anyhow};

#[allow(dead_code)]
pub struct Model {
    font: FontArc,
}

impl Model {
    pub fn from_bytes(data: &[u8]) -> Result<Self> {
        let font = FontArc::try_from_vec(data.to_vec())?;
        Ok(Model { font })
    }

    pub fn from_vec(data: Vec<u8>) -> Result<Self> {
        let font = FontArc::try_from_vec(data)?;
        Ok(Model { font })
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
        let (width, height) = (bounds.width() as u32, bounds.height() as u32);

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
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    const FONT_PATH: &str = "fonts/NotoSansJP-Regular.otf";

    #[test]
    fn model_from_bytes() {
        let font_data = fs::read(FONT_PATH).unwrap();
        let model = Model::from_bytes(&font_data);
        assert!(model.is_ok());
    }

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
}
