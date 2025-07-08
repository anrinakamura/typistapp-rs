pub struct Color {}

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

        // Self::luminance(r, g, b)
        let yuv = Self::convert_rgb_to_yuv(r, g, b);
        Self::luminance_from_yuv(&yuv)
    }

    pub fn luminance_from_rgba(rgba: &[u8; 4]) -> f64 {
        let r = rgba[0] as f64 / 255.0;
        let g = rgba[1] as f64 / 255.0;
        let b = rgba[2] as f64 / 255.0;

        // Self::luminance(r, g, b)
        let yuv = Self::convert_rgb_to_yuv(r, g, b); 
        Self::luminance_from_yuv(&yuv)
    }

    fn luminance(r: f64, g: f64, b: f64) -> f64 {
        // Using the formula for relative luminance
        (0.2126 * r) + (0.7152 * g) + (0.0722 * b)
    }

    fn convert_rgb_to_yuv(r: f64, g: f64, b: f64) -> [f64; 3] {
        let y = 0.299 * r + 0.587 * g + 0.114 * b;
        let u = -0.169 * r - 0.331 * g + 0.500 * b;
        let v = 0.500 * r - 0.419 * g - 0.081 * b; 
        [y, u, v]
    }

    fn luminance_from_yuv(yuv: &[f64; 3]) -> f64 {
        yuv[0] // Y component represents luminance
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
