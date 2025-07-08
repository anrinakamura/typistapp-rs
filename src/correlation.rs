use log;

use crate::F64_ALMOST_ZERO;

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

    let result = numerator / denominator;
    log::trace!("Correlation result: {}", result);
    Some(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn correlation_different_lengths_returns_none() {
        assert_eq!(correlation(&[1.0], &[1.0, 2.0]), None);
    }

    #[test]
    fn correlation_empty_slices_returns_none() {
        assert_eq!(correlation(&[], &[]), None);
    }

    #[test]
    fn correlation_valid_data_returns_some() {
        let x_values = [1.0, 2.0, 3.0];
        let y_values = [4.0, 5.0, 6.0];
        let result = correlation(&x_values, &y_values);
        assert!(result.is_some());
        assert!((result.unwrap() - 1.0).abs() < 1e-9);
    }
}
