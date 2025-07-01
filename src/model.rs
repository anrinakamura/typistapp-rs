use std::sync::LazyLock;

use ab_glyph::{Font, FontArc, PxScale};
use anyhow::{Result, anyhow};
use image::DynamicImage;
use log;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

const F64_ALMOST_ZERO: f64 = 1e-12;
const NUM_OF_CANDIDATES: usize = 16;

static GLYPH_SCALE: LazyLock<PxScale> = LazyLock::new(|| PxScale::from(16.0));

#[allow(dead_code)]
#[derive(Debug, Clone, Default, PartialEq)]
struct Element {
    characteristics: Vec<f64>,
    luminance: f32,
    character: Option<char>,
    image: Option<DynamicImage>,
}

#[allow(dead_code)]
impl Element {
    pub fn characteristics(&self) -> &[f64] {
        &self.characteristics
    }

    pub fn luminance(&self) -> f32 {
        self.luminance
    }

    pub fn character(&self) -> Option<char> {
        self.character
    }

    pub fn image(&self) -> Option<&DynamicImage> {
        self.image.as_ref()
    }

    pub fn from_char(font: &FontArc, character: char, scale: PxScale) -> Result<Self> {
        let glyph = font.glyph_id(character).with_scale(scale);
        let outlined_glyph = match font.outline_glyph(glyph) {
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
            return Err(anyhow!(
                "Glyph for character '{}' has zero width or height.",
                character
            ));
        }

        let mut characteristics: Vec<f64> = vec![];
        let mut total_luminance = 0.0;
        outlined_glyph.draw(|_, _, c| {
            total_luminance += c;
            characteristics.push(c as f64);
        });

        let luminance = total_luminance / (width * height) as f32;

        Ok(Element {
            characteristics,
            luminance,
            character: Some(character),
            image: None,
        })
    }
}

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
    fn typeset_elements(&self, characters: &[char]) -> Result<Vec<Element>> {
        let elements: Vec<Element> = characters
            .par_iter()
            .map(|c| Element::from_char(&self.font, *c, *GLYPH_SCALE))
            .collect::<Result<Vec<_>>>()?;

        Ok(elements)
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

        let mut numerator = 0.0;
        let mut den_x = 0.0;
        let mut den_y = 0.0;

        for (x, y) in x_values.iter().zip(y_values.iter()) {
            let diff_x = x - mean_x;
            let diff_y = y - mean_y;
            numerator += diff_x * diff_y;
            den_x += diff_x * diff_x;
            den_y += diff_y * diff_y;
        }

        let denominator = den_x.sqrt() * den_y.sqrt();
        if denominator.abs() < F64_ALMOST_ZERO {
            let is_den_x_zero = den_x.abs() < F64_ALMOST_ZERO;
            let is_den_y_zero = den_y.abs() < F64_ALMOST_ZERO;
            let are_means_equal = (mean_x - mean_y).abs() < F64_ALMOST_ZERO;

            return match (is_den_x_zero, is_den_y_zero, are_means_equal) {
                (true, true, true) => Some(1.0),
                _ => Some(0.0),
            };
        }

        log::debug!("numerator: {}, denominator: {}", numerator, denominator);
        Some(numerator / denominator)
    }

    #[allow(dead_code)]
    fn normalized(values: &[f64], min: f64, max: f64) -> Option<Vec<f64>> {
        if min >= max {
            return None;
        }

        let range = max - min;
        let result: Vec<f64> = if range.abs() < F64_ALMOST_ZERO {
            vec![0.0; values.len()]
        } else {
            values
                .iter()
                .map(|&v| (v - min) / range)
                .collect::<Vec<_>>()
        };

        Some(result)
    }

    fn closest_luminance_index(target: f32, typeset_elements: &[Element]) -> usize {
        let result = typeset_elements.binary_search_by(|prove| {
            prove
                .luminance()
                .partial_cmp(&target)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        match result {
            Ok(i) => i,
            Err(i) => {
                if i == 0 {
                    0
                } else if i >= typeset_elements.len() {
                    typeset_elements.len() - 1
                } else {
                    let diff1 = (typeset_elements[i - 1].luminance() - target).abs();
                    let diff2 = (typeset_elements[i].luminance() - target).abs();
                    if diff1 < diff2 { i - 1 } else { i }
                }
            }
        }
    }

    fn best_match_element<'a>(target: &Element, candidates: &'a [Element]) -> Option<&'a Element> {
        let mut max = -1.0;
        let mut best: Option<&Element> = None;
        for candidate in candidates {
            if let Some(result) =
                Self::correlation(target.characteristics(), candidate.characteristics())
            {
                if result > max {
                    max = result;
                    best = Some(candidate);
                }
            }
        }

        best
    }

    fn search_typeset_element<'a>(
        picture_element: &'a Element,
        typeset_elements: &'a [Element],
    ) -> Option<&'a Element> {
        if typeset_elements.is_empty() {
            return None;
        }

        // STEP 1: find the index of the character with the most similar average luminance.
        let index = Self::closest_luminance_index(picture_element.luminance(), typeset_elements);

        // STEP 2: create a slice of candidates around that index for a more detailed search.
        // NOTE: use saturating_sub to avoid underflow when index is less than NUM_OF_CANDIDATES / 2.
        let from = index.saturating_sub(NUM_OF_CANDIDATES / 2);
        let to = std::cmp::min(typeset_elements.len(), from + NUM_OF_CANDIDATES);
        let candidates = &typeset_elements[from..to];

        if candidates.is_empty() {
            return Some(&typeset_elements[index]);
        }

        // STEP 3: from the candidates, find the best match using pixel-by-pixel correlation.
        Self::best_match_element(picture_element, candidates)
    }

    #[allow(dead_code)]
    fn convert(picture_elements: &[Element], typeset_elements: &[Element]) -> Vec<Element> {
        let default = Element::default();
        let typist_art_elements: Vec<Element> = picture_elements
            .par_iter()
            .map(|e| Self::search_typeset_element(e, typeset_elements).unwrap_or(&default))
            .cloned()
            .collect();

        typist_art_elements
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
    fn correlation_valid_data_returns_some() {
        let x_values = [1.0, 2.0, 3.0];
        let y_values = [4.0, 5.0, 6.0];
        let result = Model::correlation(&x_values, &y_values);
        assert!(result.is_some());
        assert!((result.unwrap() - 1.0).abs() < 1e-9);
    }

    #[test]
    fn normalized_invalid_range_returns_none() {
        assert_eq!(Model::normalized(&[1.0, 2.0, 3.0], 3.0, 1.0), None);
    }

    #[test]
    fn normalized_empty_values_returns_some() {
        assert_eq!(Model::normalized(&vec![], 0.0, 1.0), Some(vec![]));
    }

    #[test]
    fn normalized_valid_range_returns_some() {
        let values = [1.0, 2.0, 3.0];
        let min = 1.0;
        let max = 3.0;
        let expected = vec![0.0, 0.5, 1.0];
        let result = Model::normalized(&values, min, max);
        assert!(result.is_some());
        assert_eq!(result.unwrap(), expected);
    }

    #[test]
    fn closest_luminance_index_empty_elements() {
        let elements: Vec<Element> = vec![];
        assert_eq!(Model::closest_luminance_index(0.5, &elements), 0);
    }

    #[test]
    fn closest_luminance_index_single_element() {
        let elements = vec![Element {
            characteristics: vec![0.0; 10],
            luminance: 0.5,
            character: Some('A'),
            image: None,
        }];
        assert_eq!(Model::closest_luminance_index(0.5, &elements), 0);
    }

    #[test]
    fn closest_luminance_index_multiple_elements() {
        let elements = vec![
            Element {
                characteristics: vec![0.0; 10],
                luminance: 0.1,
                character: None,
                image: None,
            },
            Element {
                characteristics: vec![0.0; 10],
                luminance: 0.5,
                character: None,
                image: None,
            },
            Element {
                characteristics: vec![0.0; 10],
                luminance: 0.9,
                character: None,
                image: None,
            },
        ];
        assert_eq!(Model::closest_luminance_index(0.5, &elements), 1);
        assert_eq!(Model::closest_luminance_index(0.2, &elements), 0);
        assert_eq!(Model::closest_luminance_index(0.8, &elements), 2);
    }

    #[test]
    fn best_match_element_empty_candidates() {
        let target = Element {
            characteristics: vec![0.5; 10],
            luminance: 0.5,
            character: Some('A'),
            image: None,
        };
        let candidates: Vec<Element> = vec![];
        assert!(Model::best_match_element(&target, &candidates).is_none());
    }

    #[test]
    fn best_match_element_valid_candidates() {
        let target = Element {
            characteristics: vec![0.5; 10],
            luminance: 0.5,
            character: Some('A'),
            image: None,
        };
        let candidates = vec![
            Element {
                characteristics: vec![0.2; 10],
                luminance: 0.2,
                character: Some('B'),
                image: None,
            },
            Element {
                characteristics: vec![0.5; 10],
                luminance: 0.5,
                character: Some('C'),
                image: None,
            },
            Element {
                characteristics: vec![0.7; 10],
                luminance: 0.7,
                character: Some('D'),
                image: None,
            },
        ];
        let best = Model::best_match_element(&target, &candidates);
        assert!(best.is_some());
        assert_eq!(best.unwrap().characteristics(), &vec![0.5; 10]);
    }

    #[test]
    fn search_typeset_element_empty_typeset_returns_none() {
        let picture_element = Element {
            characteristics: vec![0.0; 10],
            luminance: 0.5,
            character: Some('A'),
            image: None,
        };
        let typeset_elements: Vec<Element> = vec![];
        assert!(Model::search_typeset_element(&picture_element, &typeset_elements).is_none());
    }

    #[test]
    fn search_typeset_element_valid_typeset_returns_some() {
        let picture_element = Element {
            characteristics: vec![0.5; 10],
            luminance: 0.5,
            character: Some('A'),
            image: None,
        };
        let typeset_elements = vec![
            Element {
                characteristics: vec![0.2; 10],
                luminance: 0.2,
                character: Some('B'),
                image: None,
            },
            Element {
                characteristics: vec![0.5; 10],
                luminance: 0.5,
                character: Some('C'),
                image: None,
            },
            Element {
                characteristics: vec![0.7; 10],
                luminance: 0.7,
                character: Some('D'),
                image: None,
            },
        ];
        let result = Model::search_typeset_element(&picture_element, &typeset_elements);
        assert!(result.is_some());
        let best_match = result.unwrap();
        assert_eq!(best_match.characteristics(), &vec![0.5; 10]);
        assert_eq!(best_match.character(), Some('C'));
    }
}
