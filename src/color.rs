

struct Color {}

impl Color {

    /// Converts RGB components to a luminance value.
    /// The RGB components should be in the range [0.0, 1.0].
    /// Returns the luminance as a floating-point value.
    /// # Arguments
    /// * `rgb` - A slice of three f64 values representing the RGB components.
    /// # Returns
    /// * `f64` - The calculated luminance value.
    /// # Example
    /// ```
    /// use typistapp::color::Color; 
    /// let rgb = [0.5, 0.5, 0.5];
    /// let luminance = Color::luminance_from_rgb_components(&rgb);
    /// assert_eq!(luminance, 0.5);
    /// ```
    pub fn luminance_from_rgb_components(rgb: &[f64; 3]) -> f64 {
        let r = rgb[0].clamp(0.0, 1.0);
        let g = rgb[1].clamp(0.0, 1.0);
        let b = rgb[2].clamp(0.0, 1.0);

        // Using the formula for relative luminance
        (0.2126 * r) + (0.7152 * g) + (0.0722 * b) 
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn luminance_from_rgb_red() {
        let rgb = [1.0, 0.0, 0.0];
        let luminance = Color::luminance_from_rgb_components(&rgb);
        assert_eq!(luminance, 0.2126);
    }

    #[test]
    fn luminance_from_rgb_green() {
        let rgb = [0.0, 1.0, 0.0];
        let luminance = Color::luminance_from_rgb_components(&rgb);
        assert_eq!(luminance, 0.7152);
    }

    #[test]
    fn luminance_from_rgb_blue() {
        let rgb = [0.0, 0.0, 1.0];
        let luminance = Color::luminance_from_rgb_components(&rgb);
        assert_eq!(luminance, 0.0722);
    }

    #[test]
    fn luminance_from_rgb_black() {
        let rgb = [0.0, 0.0, 0.0];
        let luminance = Color::luminance_from_rgb_components(&rgb);
        assert_eq!(luminance, 0.0);
    }

    #[test]
    fn luminance_from_rgb_white() {
        let rgb = [1.0, 1.0, 1.0];
        let luminance = Color::luminance_from_rgb_components(&rgb);
        assert_eq!(luminance, 1.0);
    }
}