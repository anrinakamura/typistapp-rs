use ab_glyph::FontArc;
use anyhow::Result;
use image::{DynamicImage, imageops};
use log;
use rayon::iter::{IntoParallelRefIterator, IntoParallelRefMutIterator, ParallelIterator};

use crate::correlation::correlation;
use crate::element::Element;
use crate::{FULL_WIDTH_SPACE, GLYPH_SCALE, IMAGE_SIZE, NUM_OF_CANDIDATES};

/// A struct that serves as the Model (M) in MVC. Specializes in data management.
/// Converts an image into typist-art using a set of full-width characters and a font.
#[derive(Debug, Clone)]
pub struct Model {
    /// The source image to be converted to typist-art.
    image: DynamicImage,

    /// A collection of full-width characters used for rendering the art.
    characters: Vec<char>,

    /// The font used to render each character.
    font: FontArc,

    /// The number of characters (columns) per line in the output art.
    columns: u32,

    /// The total number of lines (rows) in the output art.
    lines: u32,
}

impl Model {
    /// Creates a new Model instance with a resized image and initialized parameters.
    pub fn new(length: u32, image: &DynamicImage, characters: &[char], font: &[u8]) -> Self {
        let columns = length;
        let width = IMAGE_SIZE * columns;
        let height = image.height() * width / image.width();
        let img = image.resize(width, height, imageops::FilterType::Triangle);
        let lines = height / IMAGE_SIZE;
        log::info!(
            "Image dimensions: {}x{}, size: {}, columns: {}, lines: {}",
            width,
            height,
            IMAGE_SIZE,
            columns,
            lines
        );

        Model {
            image: img,
            characters: characters.to_vec(),
            font: FontArc::try_from_vec(font.to_vec()).unwrap(),
            columns,
            lines,
        }
    }

    /// Converts the input image into a vector of typist-art strings.
    pub fn convert(&mut self) -> Result<Vec<String>> {
        let typeset_elements = self.typeset_elements(&self.characters)?;
        let picture_elements =
            self.picture_elements(&self.image, IMAGE_SIZE, self.columns, self.lines)?;
        log::info!(
            "Typeset elements: {}, Picture elements: {}",
            typeset_elements.len(),
            picture_elements.len()
        );

        let typist_art_elements = Self::generate_typist_art(&picture_elements, &typeset_elements);
        log::info!("Converted picture elements to typist art.");

        let mut result = vec![];
        let mut v = vec![];
        for (i, e) in typist_art_elements.iter().enumerate() {
            if i % self.columns as usize == 0 && i != 0 {
                result.push(v.iter().collect());
                v.clear();
            }
            v.push(e.character().unwrap_or(FULL_WIDTH_SPACE));
        }

        Ok(result)
    }

    /// Divides the input image into a grid of picture elements (tiles),
    /// computes their luminance characteristics, and normalizes them.
    fn picture_elements(
        &self,
        image: &DynamicImage,
        size: u32,
        columns: u32,
        lines: u32,
    ) -> Result<Vec<Element>> {
        let mut elements = vec![];
        for y in 0..lines {
            for x in 0..columns {
                let block_image = image.crop_imm(x * size, y * size, size, size);
                elements.push(Element::from_image(block_image)?);
            }
        }

        // normalize the luminance of the picture elements.
        Self::normalize_elements(&mut elements)?;

        Ok(elements)
    }

    /// Renders each character into an image using the given font, converts
    /// them into elements, normalizes their luminance, and sorts them by brightness.
    fn typeset_elements(&self, characters: &[char]) -> Result<Vec<Element>> {
        let mut elements: Vec<Element> = characters
            .par_iter()
            .map(|c| Element::from_char(&self.font, *c, *GLYPH_SCALE))
            .collect::<Result<Vec<_>>>()?;

        // normalize the luminance of the typeset elements.
        Self::normalize_elements(&mut elements)?;

        // sort the typeset elements by luminance.
        elements.sort_by(|a, b| {
            a.luminance()
                .partial_cmp(&b.luminance())
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        log::debug!("Sorted typeset elements by luminance.");
        for e in &elements {
            log::debug!(
                "Character: {:?}, Luminance: {}",
                e.character(),
                e.luminance(),
            );
        }

        Ok(elements)
    }

    /// Normalizes the luminance and pixel characteristics of each element
    /// so that all values are within a common range.
    fn normalize_elements(elements: &mut [Element]) -> Result<()> {
        let mut min = f64::INFINITY;
        let mut max = f64::NEG_INFINITY;
        for e in elements.into_iter() {
            if e.luminance() < min {
                min = e.luminance();
            }
            if e.luminance() > max {
                max = e.luminance();
            }
        }
        log::info!("Luminance range: [{}, {}]", min, max,);

        elements
            .par_iter_mut()
            .for_each(|e| e.normalized(min, max).unwrap());
        log::info!("Normalized elements.");

        Ok(())
    }

    /// Finds the index of the element in the typeset list whose luminance is
    /// closest to the given target luminance value.
    fn closest_luminance_index(target: f64, typeset_elements: &[Element]) -> usize {
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

    /// Selects the best-matching element from the given candidates
    /// based on pixel-wise correlation similarity.
    fn best_match_element<'a>(target: &Element, candidates: &'a [Element]) -> Option<&'a Element> {
        let mut max = -1.0;
        let mut best: Option<&Element> = None;
        for candidate in candidates {
            if let Some(result) = correlation(target.characteristics(), candidate.characteristics())
            {
                if result > max {
                    max = result;
                    best = Some(candidate);
                }
            }
        }

        best
    }

    /// Finds the best-matching character element for a picture element
    /// by combining luminance-based preselection and pixel correlation.
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

    /// Converts the picture elements into their best-matching character elements
    /// to generate the final typist-art structure.
    fn generate_typist_art(
        picture_elements: &[Element],
        typeset_elements: &[Element],
    ) -> Vec<Element> {
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

    #[test]
    fn closest_luminance_index_empty_elements() {
        let elements: Vec<Element> = vec![];
        assert_eq!(Model::closest_luminance_index(0.5, &elements), 0);
    }

    #[test]
    fn closest_luminance_index_single_element() {
        let elements = vec![Element::new(vec![0.0; 10], 0.5, Some('A'), None)];
        assert_eq!(Model::closest_luminance_index(0.5, &elements), 0);
    }

    #[test]
    fn closest_luminance_index_multiple_elements() {
        let elements = vec![
            Element::new(vec![0.0; 10], 0.1, None, None),
            Element::new(vec![0.0; 10], 0.5, None, None),
            Element::new(vec![0.0; 10], 0.9, None, None),
        ];
        assert_eq!(Model::closest_luminance_index(0.5, &elements), 1);
        assert_eq!(Model::closest_luminance_index(0.2, &elements), 0);
        assert_eq!(Model::closest_luminance_index(0.8, &elements), 2);
    }

    #[test]
    fn best_match_element_empty_candidates() {
        let target = Element::new(vec![0.5; 10], 0.5, Some('A'), None);
        let candidates: Vec<Element> = vec![];
        assert!(Model::best_match_element(&target, &candidates).is_none());
    }

    #[test]
    fn best_match_element_valid_candidates() {
        let target = Element::new(vec![0.5; 10], 0.5, Some('A'), None);
        let candidates = vec![
            Element::new(vec![0.2; 10], 0.2, Some('B'), None),
            Element::new(vec![0.5; 10], 0.5, Some('C'), None),
            Element::new(vec![0.7; 10], 0.7, Some('D'), None),
        ];
        let best = Model::best_match_element(&target, &candidates);
        assert!(best.is_some());
        assert_eq!(best.unwrap().characteristics(), &vec![0.5; 10]);
    }

    #[test]
    fn search_typeset_element_empty_typeset_returns_none() {
        let picture_element = Element::new(vec![0.0; 10], 0.5, Some('A'), None);
        let typeset_elements: Vec<Element> = vec![];
        assert!(Model::search_typeset_element(&picture_element, &typeset_elements).is_none());
    }

    #[test]
    fn search_typeset_element_valid_typeset_returns_some() {
        let picture_element = Element::new(vec![0.5; 10], 0.5, Some('A'), None);
        let typeset_elements = vec![
            Element::new(vec![0.2; 10], 0.2, Some('B'), None),
            Element::new(vec![0.2; 10], 0.2, Some('B'), None),
            Element::new(vec![0.5; 10], 0.5, Some('C'), None),
            Element::new(vec![0.7; 10], 0.7, Some('D'), None),
        ];
        let result = Model::search_typeset_element(&picture_element, &typeset_elements);
        assert!(result.is_some());
        let best_match = result.unwrap();
        assert_eq!(best_match.characteristics(), &vec![0.5; 10]);
        assert_eq!(best_match.character(), Some('C'));
    }
}
