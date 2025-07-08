use ab_glyph::{Font, FontArc, PxScale};
use anyhow::{Result, anyhow};
use image::{DynamicImage, GenericImageView};
use log;

use crate::color::Color;
use crate::{F64_ALMOST_ZERO, FULL_WIDTH_SPACE, IMAGE_SIZE};

/// Represents either a character or image tile, along with its
/// luminance and pixel characteristics used for comparison and matching.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Element {
    characteristics: Vec<f64>,
    luminance: f64,
    character: Option<char>,
    image: Option<DynamicImage>,
}

impl Element {
    /// Constructs a new Element with the given characteristics and metadata.
    pub fn new(
        characteristics: Vec<f64>,
        luminance: f64,
        character: Option<char>,
        image: Option<DynamicImage>,
    ) -> Self {
        Element {
            characteristics,
            luminance,
            character,
            image,
        }
    }

    /// Returns the pixel intensity values of the element.
    pub fn characteristics(&self) -> &[f64] {
        &self.characteristics
    }

    /// Returns the average luminance of the element.
    pub fn luminance(&self) -> f64 {
        self.luminance
    }

    /// Returns the character associated with this element, if any.
    pub fn character(&self) -> Option<char> {
        self.character
    }

    /// Returns a reference to the image of this element, if available.
    #[allow(dead_code)]
    pub fn image(&self) -> Option<&DynamicImage> {
        self.image.as_ref()
    }

    /// Creates an element by rendering a character into an image using the provided font and scale,
    /// then converting it into luminance data.
    pub fn from_char(font: &FontArc, character: char, scale: PxScale) -> Result<Self> {
        let (width, height) = (IMAGE_SIZE, IMAGE_SIZE);
        let mut characteristics = vec![1.0; (width * height) as usize];

        let glyph = font.glyph_id(character).with_scale(scale);
        let outline = match font.outline_glyph(glyph) {
            Some(g) => g,
            None => {
                if character == FULL_WIDTH_SPACE {
                    return Ok(Element {
                        characteristics,
                        luminance: 1.0,
                        character: Some('　'),
                        image: None,
                    });
                }
                return Err(anyhow!(
                    "Failed to outline glyph for character: {}",
                    character
                ));
            }
        };

        let bounds = outline.px_bounds();

        // canvas center - glyph center
        let glyph_center_x = bounds.min.x + bounds.width() / 2.0;
        let glyph_center_y = bounds.min.y + bounds.height() / 2.0;

        let canvas_center_x = width as f32 / 2.0;
        let canvas_center_y = height as f32 / 2.0;

        let offset_x = canvas_center_x - glyph_center_x;
        let offset_y = canvas_center_y - glyph_center_y;

        outline.draw(|x, y, c| {
            let canvas_x = x as f32 + offset_x;
            let canvas_y = y as f32 + offset_y;

            if canvas_x >= 0.0
                && canvas_x < width as f32
                && canvas_y >= 0.0
                && canvas_y < height as f32
            {
                let index = (canvas_y as u32 * width + canvas_x as u32) as usize;
                characteristics[index] = 1.0 - (c as f64);
            }
        });

        let luminance = characteristics.iter().sum::<f64>() / (width * height) as f64;

        log::debug!(
            "Character: '{}', Width: {}, Height: {}, Luminance: {}",
            character,
            width,
            height,
            luminance
        );

        Ok(Element {
            characteristics,
            luminance,
            character: Some(character),
            image: None,
        })
    }

    /// Creates an element from an image tile by calculating its luminance characteristics.
    pub fn from_image(image: DynamicImage) -> Result<Self> {
        let (width, height) = image.dimensions();
        log::trace!("Image dimensions: {}x{}", width, height);
        if width == 0 || height == 0 {
            return Err(anyhow!("Image has zero width or height."));
        }

        let mut characteristics: Vec<f64> = vec![];
        let mut total_luminance: f64 = 0.0;

        for (_, _, rgba) in image.pixels() {
            let l = Color::luminance_from_rgba(&rgba.0);
            total_luminance += l;
            characteristics.push(l);
        }

        let luminance = total_luminance / (width * height) as f64;

        Ok(Element {
            characteristics,
            luminance,
            character: None,
            image: Some(image),
        })
    }

    /// Normalizes the element's pixel characteristics and luminance
    /// to fall within the given luminance range.
    pub fn normalized(&mut self, min: f64, max: f64) -> Result<()> {
        if min >= max {
            return Err(anyhow!(
                "Invalid range: min ({}) must be less than max ({})",
                min,
                max
            ));
        }

        log::trace!(
            "Normalizing element: character: {:?}, luminance: {}",
            self.character,
            self.luminance,
        );

        for value in &mut self.characteristics {
            *value = Self::normalize(*value, min, max);
        }
        self.luminance = Self::normalize(self.luminance, min, max);

        log::trace!(
            "Normalized element: character: {:?}, luminance: {}",
            self.character,
            self.luminance,
        );

        Ok(())
    }

    /// Normalizes a single luminance value into the given range.
    fn normalize(value: f64, min: f64, max: f64) -> f64 {
        if max - min < F64_ALMOST_ZERO {
            0.0
        } else {
            (value - min) / (max - min)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    const FONT_PATH: &str = "resources/NotoSansJP-Regular.otf";

    #[test]
    fn element_from_char() {
        let font_data = fs::read(FONT_PATH).unwrap();
        let font = FontArc::try_from_vec(font_data).unwrap();
        let scale = PxScale::from(16.0);
        let element = Element::from_char(&font, 'A', scale);
        assert!(element.is_ok());
        let element = element.unwrap();
        assert_eq!(element.character, Some('A'));
        assert!(!element.characteristics.is_empty());
    }

    #[test]
    fn normalized_invalid_range_returns_err() {
        let mut element = Element::new(vec![0.5, 0.6, 0.7], 0.6, Some('A'), None);
        let result = element.normalized(0.7, 0.5);
        assert!(result.is_err());
    }

    #[test]
    fn normalized_valid_range_returns_ok() {
        let mut element = Element::new(vec![0.5, 0.6, 0.7], 0.6, Some('A'), None);
        let result = element.normalized(0.5, 0.7);
        assert!(result.is_ok());
        assert_eq!(element.characteristics, vec![0.0, 0.5, 1.0]);
        assert_eq!(element.luminance, 0.5);
    }
}
