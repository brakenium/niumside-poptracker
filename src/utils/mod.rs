pub fn safe_percentage<T: Into<f64>>(numerator: T, denominator: T) -> f64 {
    let numerator = numerator.into();
    let denominator = denominator.into();
    if denominator == 0.0 {
        0.0
    } else {
        (numerator / denominator) * 100.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ALLOWED_ERROR: f64 = 0.0001;

    #[test]
    fn test_safe_percentage() {
        assert!((safe_percentage(0, 0) - 0.0).abs() < ALLOWED_ERROR);
        assert!((safe_percentage(0, 1) - 0.0).abs() < ALLOWED_ERROR);
        assert!((safe_percentage(1, 0) - 0.0).abs() < ALLOWED_ERROR);
        assert!((safe_percentage(1, 1) - 100.0).abs() < ALLOWED_ERROR);
        assert!((safe_percentage(1, 2) - 50.0).abs() < ALLOWED_ERROR);
        assert!((safe_percentage(1, 200) - 0.5).abs() < ALLOWED_ERROR);
    }
}
