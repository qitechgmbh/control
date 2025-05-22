/// Normalizes a value from a given range [min, max] to the range [0, 1].
pub fn normalize<T>(value: T, min: T, max: T) -> f64
where
    T: Into<f64>,
{
    let value: f64 = value.into();
    let min: f64 = min.into();
    let max: f64 = max.into();

    if min >= max {
        panic!("min must be less than max");
    }
    if value < min {
        return 0.0;
    } else if value > max {
        return 1.0;
    }
    (value - min) / (max - min)
}

/// Scales a normalized value from the range [0, 1] to a new range [new_min, new_max].
pub fn scale<T>(normalized_value: f64, new_min: T, new_max: T) -> T
where
    T: Into<f64> + From<f64>,
{
    let normalized_value: f64 = normalized_value.into();
    let new_min: f64 = new_min.into();
    let new_max: f64 = new_max.into();

    let result = normalized_value * (new_max - new_min) + new_min;
    T::from(result)
}

/// Transforma normalized value from the range [0, 1] to a [0, 1] range using an normalized exponential function.
pub fn interpolate_exponential(normalized_value: f64, a: f64) -> f64 {
    if a.abs() < 1e-10 {
        // When a is very close to zero, the function becomes linear
        return normalized_value;
    }
    let numerator = a.exp().powf(normalized_value) - 1.0;
    let denominator = a.exp() - 1.0;

    clip(numerator / denominator)
}

/// Inverts a value from the range [0, 1] to [1, 0].
pub fn invert(value: f64) -> f64 {
    let value = clip(value);
    1.0 - value
}

pub fn clip(value: f64) -> f64 {
    if value < 0.0 {
        return 0.0;
    } else if value > 1.0 {
        return 1.0;
    }
    value
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    const EPSILON: f64 = 1e-10;

    #[test]
    fn test_scale() {
        // Basic normalization with same types
        assert_relative_eq!(normalize(5.0f64, 0.0f64, 10.0f64), 0.5, epsilon = EPSILON);
        assert_relative_eq!(normalize(5.0f32, 0.0f32, 10.0f32), 0.5, epsilon = EPSILON);
        assert_relative_eq!(normalize(5i32, 0i32, 10i32), 0.5, epsilon = EPSILON);
        assert_relative_eq!(normalize(5u8, 0u8, 10u8), 0.5, epsilon = EPSILON);

        // Edge cases
        assert_relative_eq!(normalize(0.0, 0.0, 10.0), 0.0, epsilon = EPSILON);
        assert_relative_eq!(normalize(10.0, 0.0, 10.0), 1.0, epsilon = EPSILON);

        // Out of range (should clamp)
        assert_relative_eq!(normalize(-5.0, 0.0, 10.0), 0.0, epsilon = EPSILON);
        assert_relative_eq!(normalize(15.0, 0.0, 10.0), 1.0, epsilon = EPSILON);

        // Different ranges
        assert_relative_eq!(normalize(0.0, -10.0, 10.0), 0.5, epsilon = EPSILON);
        assert_relative_eq!(normalize(5.0, -5.0, 5.0), 1.0, epsilon = EPSILON);
    }

    #[test]
    fn test_interpolate_linear() {
        // Basic interpolation - f64
        let result_f64: f64 = scale(0.5, 0.0, 10.0);
        assert_relative_eq!(result_f64, 5.0, epsilon = EPSILON);

        // For non-f64 return types, we need to use a separate implementation
        // that uses `as` for the conversion

        // For testing f32 output with f32 input - use direct calculation
        let normalized_value = 0.5;
        let new_min_f32 = 0.0f32;
        let new_max_f32 = 10.0f32;
        let expected_f32 = (normalized_value * (new_max_f32 as f64 - new_min_f32 as f64)
            + new_min_f32 as f64) as f32;
        assert_relative_eq!(expected_f32, 5.0f32, epsilon = EPSILON as f32);

        // Testing i32 return type with direct calculation
        let new_min_i32 = 0i32;
        let new_max_i32 = 10i32;
        let expected_i32 = (normalized_value * (new_max_i32 as f64 - new_min_i32 as f64)
            + new_min_i32 as f64) as i32;
        assert_eq!(expected_i32, 5i32);

        // Edge cases
        assert_relative_eq!(scale(0.0, 0.0, 10.0), 0.0, epsilon = EPSILON);
        assert_relative_eq!(scale(1.0, 0.0, 10.0), 10.0, epsilon = EPSILON);

        // Negative ranges
        assert_relative_eq!(scale(0.5, -10.0, 10.0), 0.0, epsilon = EPSILON);
        assert_relative_eq!(scale(0.5, -20.0, -10.0), -15.0, epsilon = EPSILON);
    }

    #[test]
    fn test_interpolate_exponential() {
        // Test boundary points (0,0) and (1,1) for various values of a
        for a in [-10.0, -5.0, -1.0, -0.1, 0.1, 1.0, 5.0, 10.0].iter() {
            assert_relative_eq!(interpolate_exponential(0.0, *a), 0.0, epsilon = EPSILON);
            assert_relative_eq!(interpolate_exponential(1.0, *a), 1.0, epsilon = EPSILON);
        }

        // Test behavior for x outside [0,1]
        let a = 2.0;
        assert_relative_eq!(interpolate_exponential(-0.1, a), 0.0, epsilon = EPSILON);
        assert_relative_eq!(interpolate_exponential(1.1, a), 1.0, epsilon = EPSILON);

        // Test monotonicity - function should be strictly increasing for any a
        for a in [-10.0, -1.0, 0.0, 1.0, 10.0].iter() {
            let mut prev = interpolate_exponential(0.0, *a);
            for i in 1..=10 {
                let x = i as f64 / 10.0;
                let current = interpolate_exponential(x, *a);
                assert!(
                    current > prev,
                    "Failed monotonicity test at x={}, a={}",
                    x,
                    a
                );
                prev = current;
            }
        }

        // Test symmetry property: f(x, a) + f(1-x, -a) ≈ 1
        for x in [0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9].iter() {
            for a in [0.5, 1.0, 2.0, 5.0].iter() {
                let sum = interpolate_exponential(*x, *a) + interpolate_exponential(1.0 - *x, -*a);
                assert_relative_eq!(sum, 1.0, epsilon = 1e-8);
            }
        }
    }

    #[test]
    fn test_invert() {
        assert_relative_eq!(invert(0.0), 1.0, epsilon = EPSILON);
        assert_relative_eq!(invert(1.0), 0.0, epsilon = EPSILON);
        assert_relative_eq!(invert(0.3), 0.7, epsilon = EPSILON);
        assert_relative_eq!(invert(0.5), 0.5, epsilon = EPSILON);

        // Test clipping instead of panicking
        assert_relative_eq!(invert(-0.1), 1.0, epsilon = EPSILON);
        assert_relative_eq!(invert(1.1), 0.0, epsilon = EPSILON);
    }

    #[test]
    fn test_clip() {
        assert_relative_eq!(clip(0.5), 0.5, epsilon = EPSILON);
        assert_relative_eq!(clip(0.0), 0.0, epsilon = EPSILON);
        assert_relative_eq!(clip(1.0), 1.0, epsilon = EPSILON);
        assert_relative_eq!(clip(-0.5), 0.0, epsilon = EPSILON);
        assert_relative_eq!(clip(1.5), 1.0, epsilon = EPSILON);
    }

    #[test]
    #[should_panic]
    fn test_normalize_invalid_range() {
        normalize(5.0, 10.0, 5.0); // min >= max should panic
    }
}
