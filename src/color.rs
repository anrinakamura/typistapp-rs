/// A utility struct for color-related operations.
pub struct Color {}

impl Color {
    /// Calculates the luminance of an RGBA color.
    ///
    /// # Arguments
    ///
    /// * `rgba` - A reference to a 4-element array representing a color in RGBA format (0–255 range).
    ///
    /// # Returns
    ///
    /// * A `f64` value representing the luminance (brightness) of the color, normalized to the 0.0–1.0 range.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use typistapp::color::Color;
    ///
    /// let luminance = Color::luminance_from_rgba(&[255, 255, 255, 255]);
    /// assert!(luminance > 0.9);
    /// ```
    pub fn luminance_from_rgba(rgba: &[u8; 4]) -> f64 {
        let r = rgba[0] as f64 / 255.0;
        let g = rgba[1] as f64 / 255.0;
        let b = rgba[2] as f64 / 255.0;

        let yuv = Self::convert_rgb_to_yuv(r, g, b);
        Self::luminance_from_yuv(&yuv)
    }

    /// Converts an RGB color to YUV color space.
    ///
    /// # Arguments
    ///
    /// * `r` - Red component (0.0–1.0).
    /// * `g` - Green component (0.0–1.0).
    /// * `b` - Blue component (0.0–1.0).
    ///
    /// # Returns
    ///
    /// * A `[f64; 3]` array where the elements represent the Y (luminance), U, and V components respectively.
    pub fn convert_rgb_to_yuv(r: f64, g: f64, b: f64) -> [f64; 3] {
        let y = 0.299 * r + 0.587 * g + 0.114 * b;
        let u = -0.169 * r - 0.331 * g + 0.500 * b;
        let v = 0.500 * r - 0.419 * g - 0.081 * b;
        [y, u, v]
    }

    /// Extracts the luminance component (Y) from a YUV color.
    ///
    /// # Arguments
    ///
    /// * `yuv` - A reference to a 3-element array representing a YUV color.
    ///
    /// # Returns
    ///
    /// * A `f64` value representing the luminance.
    pub fn luminance_from_yuv(yuv: &[f64; 3]) -> f64 {
        yuv[0] // Y component represents luminance
    }
}

#[cfg(test)]
mod tests {
    use super::Color;

    #[test]
    fn luminance_black() {
        let rgba = [0, 0, 0, 255]; // black
        let lum = Color::luminance_from_rgba(&rgba);
        assert!((lum - 0.0).abs() < 1e-6);
    }

    #[test]
    fn luminance_white() {
        let rgba = [255, 255, 255, 255]; // white
        let lum = Color::luminance_from_rgba(&rgba);
        assert!((lum - 1.0).abs() < 1e-6);
    }

    #[test]
    fn luminance_gray() {
        let rgba = [128, 128, 128, 255];
        let lum = Color::luminance_from_rgba(&rgba);
        // Expected ~0.502, allow small margin
        assert!((lum - 0.502).abs() < 0.01);
    }

    #[test]
    fn luminance_red() {
        let rgba = [255, 0, 0, 255];
        let lum = Color::luminance_from_rgba(&rgba);
        // Expected ~0.299
        assert!((lum - 0.299).abs() < 0.01);
    }

    #[test]
    fn luminance_green() {
        let rgba = [0, 255, 0, 255];
        let lum = Color::luminance_from_rgba(&rgba);
        // Expected ~0.587
        assert!((lum - 0.587).abs() < 0.01);
    }

    #[test]
    fn luminance_blue() {
        let rgba = [0, 0, 255, 255];
        let lum = Color::luminance_from_rgba(&rgba);
        // Expected ~0.114
        assert!((lum - 0.114).abs() < 0.01);
    }

    #[test]
    fn convert_rgb_to_yuv() {
        let yuv = Color::convert_rgb_to_yuv(1.0, 0.0, 0.0); // pure red
        // Y ≈ 0.299
        assert!((yuv[0] - 0.299).abs() < 0.01);
        // U and V can vary; check rough expected signs
        assert!(yuv[1] < 0.0);
        assert!(yuv[2] > 0.0);
    }

    #[test]
    fn luminance_from_yuv_direct() {
        let yuv = [0.42, 0.1, -0.1];
        let lum = Color::luminance_from_yuv(&yuv);
        assert!((lum - 0.42).abs() < 1e-6);
    }
}
