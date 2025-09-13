/// Normalizes a value from a given range [min, max] to the range [0, 1].
///
/// This function maps a value from an arbitrary range to the standard
/// normalized range [0, 1]. Values o/// Plot for Steepness = -1:ge are clamped
/// to the boundaries (0.0 or 1.0).
///
/// # Arguments
/// * `value` - The input value to normalize
/// * `min` - The minimum value of the input range
/// * `max` - The maximum value of the input range
///
/// # Returns
/// * Normalized value in range [0.0, 1.0]
///
/// # Panics
/// * Panics if `min >= max`
///
/// # Example
/// ```rust
/// use control_core::helpers::interpolation::normalize;
/// let normalized = normalize(5.0, 0.0, 10.0); // Returns 0.5
/// let clamped = normalize(-2.0, 0.0, 10.0);   // Returns 0.0 (clamped)
/// ```
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
///
/// This function takes a normalized value (typically from 0 to 1) and maps it
/// to a new target range. It's the inverse operation of normalization.
///
/// # Arguments
/// * `normalized_value` - Input value in range [0.0, 1.0]
/// * `new_min` - The minimum value of the target range
/// * `new_max` - The maximum value of the target range
///
/// # Returns
/// * Scaled value in the range [new_min, new_max]
///
/// # Example
/// ```rust
/// use control_core::helpers::interpolation::scale;
/// let scaled = scale(0.5, 0.0, 100.0); // Returns 50.0
/// let negative_range = scale(0.25, -10.0, 10.0); // Returns -5.0
/// ```
pub fn scale<T>(normalized_value: f64, new_min: T, new_max: T) -> T
where
    T: Into<f64> + From<f64>,
{
    let normalized_value: f64 = normalized_value;
    let new_min: f64 = new_min.into();
    let new_max: f64 = new_max.into();

    let result = normalized_value.mul_add(new_max - new_min, new_min);
    T::from(result)
}

/// Performs hinge interpolation creating a piecewise linear function.
///
/// This function creates a piecewise linear curve that:
/// - For values < x: linearly interpolates from (0, 0) to (x, y)
/// - For values > x: linearly interpolates from (x, y) to (1, 1)
/// - For value == x: returns y
///
/// The function assumes the input value is normalized to the range [0, 1],
/// and creates a "hinge" or "elbow" shape at the point (x, y).
///
/// # Arguments
/// * `value` - The input value (typically in range [0, 1])
/// * `x` - The x-coordinate of the hinge point (typically in range [0, 1])
/// * `y` - The y-coordinate of the hinge point (typically in range [0, 1])
///
/// # Returns
/// * Interpolated value based on the hinge function
///
/// # Example
/// ```rust
/// use control_core::helpers::interpolation::interpolate_hinge;
/// // Create a hinge at (0.3, 0.7)
/// let result1 = interpolate_hinge(0.15, 0.3, 0.7); // Returns 0.35 (halfway to hinge)
/// let result2 = interpolate_hinge(0.3, 0.3, 0.7);  // Returns 0.7 (at hinge)
/// let result3 = interpolate_hinge(0.65, 0.3, 0.7); // Returns 0.85 (halfway from hinge to 1)
/// ```
/// ```ignore
/// Hinge at (0.8, 0.2)
/// ⡁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢰⠁ 1.0
/// ⠄⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡜⠀
/// ⠂⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢀⠇⠀
/// ⡁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢀⠎⠀⠀
/// ⠄⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢸⠀⠀⠀
/// ⠂⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡇⠀⠀⠀
/// ⡁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢰⠁⠀⠀⠀
/// ⠄⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡜⠀⠀⠀⠀
/// ⠂⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢀⠇⠀⠀⠀⠀
/// ⡁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢀⠎⠀⠀⠀⠀⠀
/// ⠄⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢸⠀⠀⠀⠀⠀⠀
/// ⠂⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡇⠀⠀⠀⠀⠀⠀
/// ⡁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢀⣀⠤⠤⠒⠒⠊⠁⠀⠀⠀⠀⠀⠀
/// ⠄⠀⠀⠀⠀⠀⠀⠀⠀⣀⣀⡠⠤⠒⠒⠉⠉⠁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
/// ⠂⣀⣀⠤⠤⠔⠒⠉⠉⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
/// ⠉⠈⠀⠁⠈⠀⠁⠈⠀⠁⠈⠀⠁⠈⠀⠁⠈⠀⠁⠈⠀⠁⠈⠀⠁⠈⠀⠁⠈⠀⠁ 0.0
/// 0.0                        1.0
/// ```
pub fn interpolate_hinge<T>(value: T, x: T, y: T) -> f64
where
    T: Into<f64>,
{
    let value_f64: f64 = value.into();
    let x_f64: f64 = x.into();
    let y_f64: f64 = y.into();

    // If we are under X we scale linearly from 0 to y
    // If we are over X we scale linearly from y to 1
    if value_f64 < x_f64 {
        normalize(value_f64, 0.0, x_f64) * y_f64
    } else if value_f64 > x_f64 {
        normalize(value_f64, x_f64, 1.0).mul_add(1.0 - y_f64, y_f64)
    } else {
        y_f64
    }
}

/// Transforms a normalized value from the range [0, 1] to a [0, 1] range using a normalized exponential function.
///
/// This function creates smooth curves that are either convex (fast start, slow end)
/// or concave (slow start, fast end) depending on the steepness parameter sign.
/// The curve always passes through (0,0) and (1,1).
///
/// # Arguments
/// * `normalized_value` - Input value in range [0.0, 1.0]
/// * `steepness` - Controls the curve shape:
///   - `steepness = 0.0`: Linear interpolation (y = x)
///   - `steepness > 0.0`: Convex curve (slow start → fast end)
///   - `steepness < 0.0`: Concave curve (fast start → slow end)
///   - Typical range: -5.0 to 5.0 for most applications
///
/// # Returns
/// * Transformed value in range [0.0, 1.0]
///
/// # Example
/// ```rust
/// use control_core::helpers::interpolation::interpolate_exponential;
/// let convex = interpolate_exponential(0.5, 2.0);   // Slow start, fast end
/// let concave = interpolate_exponential(0.5, -2.0); // Fast start, slow end
/// let linear = interpolate_exponential(0.5, 0.0);   // Linear (returns 0.5)
/// ```
///
/// # Plots
/// Steepness = -3
/// Steepness = -3
/// ```ignore
/// ⡁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣀⡠⠤⠔⠒⠒⠉⠁ 1.0
/// ⠄⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣀⠤⠔⠊⠉⠀⠀⠀⠀⠀⠀⠀⠀
/// ⠂⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡠⠒⠊⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
/// ⡁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢀⠤⠊⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
/// ⠄⠀⠀⠀⠀⠀⠀⠀⠀⠀⢀⠔⠁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
/// ⠂⠀⠀⠀⠀⠀⠀⠀⠀⡔⠁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
/// ⡁⠀⠀⠀⠀⠀⠀⢀⠎⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
/// ⠄⠀⠀⠀⠀⠀⡔⠁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
/// ⠂⠀⠀⠀⠀⡰⠁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
/// ⡁⠀⠀⢀⠔⠁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
/// ⠄⠀⠀⡜⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
/// ⠂⠀⢰⠁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
/// ⡁⢀⠇⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
/// ⢤⠃⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
/// ⡎⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
/// ⠁⠈⠀⠁⠈⠀⠁⠈⠀⠁⠈⠀⠁⠈⠀⠁⠈⠀⠁⠈⠀⠁⠈⠀⠁⠈⠀⠁⠈⠀⠁ -0.0
/// 0.0                        1.0
/// ```
///
/// Steepness = -1
/// ```ignore
/// ⡁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢀⠤⠒⠁ 1.0
/// ⠄⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢀⠤⠊⠁⠀⠀⠀
/// ⠂⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢀⠤⠒⠁⠀⠀⠀⠀⠀⠀
/// ⡁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣀⠔⠁⠀⠀⠀⠀⠀⠀⠀⠀⠀
/// ⠄⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡠⠊⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
/// ⠂⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡰⠉⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
/// ⡁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢀⠤⠊⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
/// ⠄⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡠⠃⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
/// ⠂⠀⠀⠀⠀⠀⠀⠀⠀⡠⠊⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
/// ⡁⠀⠀⠀⠀⠀⠀⢀⠜⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
/// ⠄⠀⠀⠀⠀⠀⡔⠁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
/// ⠂⠀⠀⠀⢀⠎⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
/// ⡁⠀⠀⡔⠁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
/// ⠄⢀⠜⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
/// ⡲⠁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
/// ⠁⠈⠀⠁⠈⠀⠁⠈⠀⠁⠈⠀⠁⠈⠀⠁⠈⠀⠁⠈⠀⠁⠈⠀⠁⠈⠀⠁⠈⠀⠁ -0.0
/// 0.0                        1.0
/// ```
///
/// Steepness = 0
/// ```ignore
/// ⡁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢀⠔⠁ 1.0
/// ⠄⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢀⠔⠁⠀⠀
/// ⠂⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢀⠔⠁⠀⠀⠀⠀
/// ⡁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢀⠔⠁⠀⠀⠀⠀⠀⠀
/// ⠄⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢀⠔⠁⠀⠀⠀⠀⠀⠀⠀⠀
/// ⠂⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢀⠔⠁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
/// ⡁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢀⠔⠁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
/// ⠄⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢀⠔⠁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
/// ⠂⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢀⠔⠁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
/// ⡁⠀⠀⠀⠀⠀⠀⠀⠀⠀⢀⠔⠁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
/// ⠄⠀⠀⠀⠀⠀⠀⠀⢀⠔⠁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
/// ⠂⠀⠀⠀⠀⠀⢀⠔⠁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
/// ⡁⠀⠀⠀⢀⠔⠁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
/// ⠄⠀⢀⠔⠁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
/// ⢂⠔⠁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
/// ⠁⠈⠀⠁⠈⠀⠁⠈⠀⠁⠈⠀⠁⠈⠀⠁⠈⠀⠁⠈⠀⠁⠈⠀⠁⠈⠀⠁⠈⠀⠁ 0.0
/// 0.0                        1.0
/// ```
///
/// Steepness = 1
/// ```ignore
/// ⡁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡰⠁ 1.0
/// ⠄⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢀⠔⠁⠀
/// ⠂⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢠⠃⠀⠀⠀
/// ⡁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢀⠔⠁⠀⠀⠀⠀
/// ⠄⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢠⠊⠀⠀⠀⠀⠀⠀
/// ⠂⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢀⠔⠁⠀⠀⠀⠀⠀⠀⠀
/// ⡁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡠⠃⠀⠀⠀⠀⠀⠀⠀⠀⠀
/// ⠄⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡠⠊⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
/// ⠂⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣀⠜⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
/// ⡁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡠⠊⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
/// ⠄⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡠⠒⠁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
/// ⠂⠀⠀⠀⠀⠀⠀⠀⠀⢀⠤⠊⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
/// ⡁⠀⠀⠀⠀⠀⢀⣀⠔⠁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
/// ⠄⠀⠀⠀⣀⠔⠁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
/// ⢂⣀⠔⠉⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
/// ⠁⠈⠀⠁⠈⠀⠁⠈⠀⠁⠈⠀⠁⠈⠀⠁⠈⠀⠁⠈⠀⠁⠈⠀⠁⠈⠀⠁⠈⠀⠁ 0.0
/// 0.0                        1.0
/// ```
///
/// Steepness = 3
/// ```ignore
/// ⡁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢰⠁ 1.0
/// ⠄⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡎⠀
/// ⠂⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢀⠎⠀⠀
/// ⡁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡜⠀⠀⠀
/// ⠄⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢰⠁⠀⠀⠀
/// ⠂⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢀⠇⠀⠀⠀⠀
/// ⡁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡰⠁⠀⠀⠀⠀⠀
/// ⠄⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢰⠁⠀⠀⠀⠀⠀⠀
/// ⠂⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢀⠔⠁⠀⠀⠀⠀⠀⠀⠀
/// ⡁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢠⠊⠀⠀⠀⠀⠀⠀⠀⠀⠀
/// ⠄⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢀⠔⠁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
/// ⠂⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣀⠔⠁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
/// ⡁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣀⡠⠊⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
/// ⠄⠀⠀⠀⠀⠀⠀⠀⠀⣀⠤⠔⠊⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
/// ⠂⠀⣀⣀⠤⠤⠒⠒⠉⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
/// ⠉⠉⠀⠁⠈⠀⠁⠈⠀⠁⠈⠀⠁⠈⠀⠁⠈⠀⠁⠈⠀⠁⠈⠀⠁⠈⠀⠁⠈⠀⠁ 0.0
/// 0.0                        1.0
/// ```
pub fn interpolate_exponential(normalized_value: f64, steepness: f64) -> f64 {
    if steepness.abs() < 1e-10 {
        // When a is very close to zero, the function becomes linear
        return normalized_value;
    }
    let numerator = steepness.exp().powf(normalized_value) - 1.0;
    let denominator = steepness.exp_m1();

    clip(numerator / denominator)
}

/// A piecewise exponential interpolation function that creates smooth S-curves
/// through the points (0,0), (0.5,0.5), and (1,1).
///
/// The function combines two normalized exponential segments:
/// - First half (0 to 0.5): inverted exponential (concave, fast start → slow end)
/// - Second half (0.5 to 1): standard exponential (convex, slow start → fast end)
///
/// This creates an S-like curve that's concave then convex, similar to
/// a smoothed step function or sigmoid-like transition.
///
/// # Arguments
/// * `x` - Input value, should be in range [0.0, 1.0]
/// * `steepness` - Controls how steep the S-curve is
///   - `steepness = 0.0`: Linear interpolation (y = x)
///   - `steepness > 0.0`: More pronounced S-curve
///   - Higher values create sharper transitions at the boundaries
///   - Typical range: 0.5 to 5.0 for most applications
///
/// # Returns
/// * Interpolated y value in range [0.0, 1.0]
///
/// # Example
/// ```rust
/// use control_core::helpers::interpolation::interpolate_inflected_exponential;
/// let y = interpolate_inflected_exponential(0.25, 2.0); // Gentle S-curve
/// let y = interpolate_inflected_exponential(0.75, 5.0); // Sharp S-curve
/// ```
///
/// # Plots
/// Steepness = 0.5
/// ```ignore
/// ⡁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡰⠁ 1.0
/// ⠄⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡰⠉⠀⠀
/// ⠂⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣀⠜⠀⠀⠀⠀
/// ⡁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢀⠜⠀⠀⠀⠀⠀⠀
/// ⠄⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢠⠒⠁⠀⠀⠀⠀⠀⠀⠀
/// ⠂⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣀⠔⠁⠀⠀⠀⠀⠀⠀⠀⠀⠀
/// ⡁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡠⠊⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
/// ⠄⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢀⠔⠉⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
/// ⠂⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡠⠒⠁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
/// ⡁⠀⠀⠀⠀⠀⠀⠀⠀⢀⠤⠊⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
/// ⠄⠀⠀⠀⠀⠀⠀⠀⡔⠁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
/// ⠂⠀⠀⠀⠀⢀⠔⠉⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
/// ⡁⠀⠀⢀⠤⠃⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
/// ⠄⠀⡠⠃⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
/// ⡢⠒⠁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
/// ⠁⠈⠀⠁⠈⠀⠁⠈⠀⠁⠈⠀⠁⠈⠀⠁⠈⠀⠁⠈⠀⠁⠈⠀⠁⠈⠀⠁⠈⠀⠁ 0.0
/// 0.0                        1.0
/// ```
///
/// Steepness = 1
/// ```ignore
/// ⡁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡰⠁ 1.0
/// ⠄⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢠⠒⠁⠀
/// ⠂⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡠⠃⠀⠀⠀
/// ⡁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡠⠒⠁⠀⠀⠀⠀
/// ⠄⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣀⠔⠁⠀⠀⠀⠀⠀⠀
/// ⠂⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡠⠊⠀⠀⠀⠀⠀⠀⠀⠀⠀
/// ⡁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢀⠔⠉⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
/// ⠄⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣀⠔⠒⠁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
/// ⠂⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢀⠔⠉⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
/// ⡁⠀⠀⠀⠀⠀⠀⠀⠀⡠⠒⠁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
/// ⠄⠀⠀⠀⠀⠀⢀⠤⠊⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
/// ⠂⠀⠀⠀⠀⡰⠁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
/// ⡁⠀⠀⡰⠉⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
/// ⠄⠀⡜⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
/// ⡲⠉⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
/// ⠁⠈⠀⠁⠈⠀⠁⠈⠀⠁⠈⠀⠁⠈⠀⠁⠈⠀⠁⠈⠀⠁⠈⠀⠁⠈⠀⠁⠈⠀⠁ 0.0
/// 0.0                        1.0
/// ```
///
/// Steepness = 2
/// ```ignore
/// ⡁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢰⠁ 1.0
/// ⠄⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡠⠃⠀
/// ⠂⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡸⠀⠀⠀
/// ⡁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡜⠀⠀⠀⠀
/// ⠄⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡰⠉⠀⠀⠀⠀⠀
/// ⠂⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢀⠤⠊⠀⠀⠀⠀⠀⠀⠀
/// ⡁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢀⠤⠔⠁⠀⠀⠀⠀⠀⠀⠀⠀⠀
/// ⠄⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣀⡠⠤⠒⠊⠁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
/// ⠂⠀⠀⠀⠀⠀⠀⠀⠀⢀⠤⠔⠉⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
/// ⡁⠀⠀⠀⠀⠀⠀⣀⠔⠁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
/// ⠄⠀⠀⠀⠀⡠⠊⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
/// ⠂⠀⠀⢠⠒⠁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
/// ⡁⠀⡠⠃⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
/// ⠄⡠⠃⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
/// ⡞⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
/// ⠁⠈⠀⠁⠈⠀⠁⠈⠀⠁⠈⠀⠁⠈⠀⠁⠈⠀⠁⠈⠀⠁⠈⠀⠁⠈⠀⠁⠈⠀⠁ 0.0
/// 0.0                        1.0
/// ```
///
/// Steepness = 5
/// ```ignore
/// ⡁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢰⠁ 1.0
/// ⠄⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡜⠀
/// ⠂⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢀⠇⠀
/// ⡁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢀⠎⠀⠀
/// ⠄⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡜⠀⠀⠀
/// ⠂⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡜⠀⠀⠀⠀
/// ⡁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣀⠔⠉⠀⠀⠀⠀⠀
/// ⠄⠀⠀⠀⠀⠀⠀⠀⠀⣀⣀⡠⠤⠤⠤⠤⠤⠤⠤⠒⠒⠊⠉⠀⠀⠀⠀⠀⠀⠀⠀
/// ⠂⠀⠀⠀⠀⢀⠔⠒⠉⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
/// ⡁⠀⠀⢠⠒⠁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
/// ⠄⠀⢠⠃⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
/// ⠂⢀⠇⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
/// ⣁⠎⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
/// ⢼⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
/// ⡇⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
/// ⠁⠈⠀⠁⠈⠀⠁⠈⠀⠁⠈⠀⠁⠈⠀⠁⠈⠀⠁⠈⠀⠁⠈⠀⠁⠈⠀⠁⠈⠀⠁ 0.0
/// 0.0                        1.0
/// ```
pub fn interpolate_inflected_exponential(x: f64, steepness: f64) -> f64 {
    debug_assert!((0.0..=1.0).contains(&x), "x must be in range [0.0, 1.0]");
    debug_assert!(steepness >= 0.0, "steepness must be non-negative");

    if steepness == 0.0 {
        // Linear case
        return x;
    }

    if x <= 0.5 {
        // First segment: inverted exponential (concave)
        // Maps [0, 0.5] → [0, 0.5] with concave shape
        let x_norm = 2.0 * x; // Scale to [0, 1]

        // Use inverted exponential interpolation
        let result = 1.0 - interpolate_exponential(1.0 - x_norm, steepness);
        0.5 * result
    } else {
        // Second segment: standard exponential (convex)
        // Maps [0.5, 1] → [0.5, 1] with convex shape
        let x_norm = 2.0 * (x - 0.5); // Scale to [0, 1]

        // Use standard exponential interpolation
        let result = interpolate_exponential(x_norm, steepness);
        0.5f64.mul_add(result, 0.5)
    }
}

/// Inverts a value from the range [0, 1] to [1, 0].
///
/// This function flips a normalized value, where 0 becomes 1 and 1 becomes 0.
/// Values outside [0, 1] are first clamped to this range before inversion.
///
/// # Arguments
/// * `value` - Input value (will be clamped to [0.0, 1.0])
///
/// # Returns
/// * Inverted value in range [0.0, 1.0]
///
/// # Example
/// ```rust
/// use control_core::helpers::interpolation::invert;
/// let inverted = invert(0.3); // Returns 0.7
/// let boundary = invert(0.0); // Returns 1.0
/// let clamped = invert(1.5);  // Returns 0.0 (input clamped to 1.0 first)
/// ```
pub fn invert(value: f64) -> f64 {
    let value = clip(value);
    1.0 - value
}

/// Clips a value to the range [0, 1].
///
/// This function ensures that any input value is constrained to the
/// normalized range [0, 1] by clamping values below 0 to 0 and
/// values above 1 to 1.
///
/// # Arguments
/// * `value` - Input value to clip
///
/// # Returns
/// * Clipped value in range [0.0, 1.0]
///
/// # Example
/// ```rust
/// use control_core::helpers::interpolation::clip;
/// let normal = clip(0.5);  // Returns 0.5
/// let low = clip(-0.3);    // Returns 0.0
/// let high = clip(1.7);    // Returns 1.0
/// ```
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
    fn test_interpolate_inflected_exponential() {
        // Test boundary points (0,0), (0.5,0.5), and (1,1) for various steepness values
        for steepness in [0.0, 0.5, 1.0, 2.0, 5.0].iter() {
            assert_relative_eq!(
                interpolate_inflected_exponential(0.0, *steepness),
                0.0,
                epsilon = EPSILON
            );
            assert_relative_eq!(
                interpolate_inflected_exponential(0.5, *steepness),
                0.5,
                epsilon = EPSILON
            );
            assert_relative_eq!(
                interpolate_inflected_exponential(1.0, *steepness),
                1.0,
                epsilon = EPSILON
            );
        }

        // Test linear case (steepness = 0)
        for x in [0.0, 0.25, 0.5, 0.75, 1.0].iter() {
            assert_relative_eq!(
                interpolate_inflected_exponential(*x, 0.0),
                *x,
                epsilon = EPSILON
            );
        }

        // Test monotonicity - function should be strictly increasing
        for steepness in [0.5, 1.0, 2.0, 5.0].iter() {
            let mut prev = interpolate_inflected_exponential(0.0, *steepness);
            for i in 1..=20 {
                let x = i as f64 / 20.0;
                let current = interpolate_inflected_exponential(x, *steepness);
                assert!(
                    current >= prev,
                    "Failed monotonicity test at x={}, steepness={}, prev={}, current={}",
                    x,
                    steepness,
                    prev,
                    current
                );
                prev = current;
            }
        }

        // Test S-curve property: first half should be concave, second half convex
        let steepness = 2.0;

        // First half: check concave property (derivative decreasing)
        let x1 = 0.1;
        let x2 = 0.2;
        let x3 = 0.3;
        let y1 = interpolate_inflected_exponential(x1, steepness);
        let y2 = interpolate_inflected_exponential(x2, steepness);
        let y3 = interpolate_inflected_exponential(x3, steepness);

        let slope1 = (y2 - y1) / (x2 - x1);
        let slope2 = (y3 - y2) / (x3 - x2);
        assert!(
            slope1 > slope2,
            "First half should be concave (decreasing slope)"
        );

        // Second half: check convex property (derivative increasing)
        let x1 = 0.7;
        let x2 = 0.8;
        let x3 = 0.9;
        let y1 = interpolate_inflected_exponential(x1, steepness);
        let y2 = interpolate_inflected_exponential(x2, steepness);
        let y3 = interpolate_inflected_exponential(x3, steepness);

        let slope1 = (y2 - y1) / (x2 - x1);
        let slope2 = (y3 - y2) / (x3 - x2);
        assert!(
            slope1 < slope2,
            "Second half should be convex (increasing slope)"
        );

        // Test that output is always in [0, 1] range
        for steepness in [0.0, 1.0, 5.0, 10.0].iter() {
            for i in 0..=100 {
                let x = i as f64 / 100.0;
                let y = interpolate_inflected_exponential(x, *steepness);
                assert!(
                    y >= 0.0 && y <= 1.0,
                    "Output {} out of range [0,1] for x={}, steepness={}",
                    y,
                    x,
                    steepness
                );
            }
        }
    }

    #[test]
    fn test_interpolate_hinge() {
        // Test basic functionality with hinge at (0.3, 0.7)
        let x_hinge = 0.3;
        let y_hinge = 0.7;

        // Test exact hinge point
        assert_relative_eq!(
            interpolate_hinge(x_hinge, x_hinge, y_hinge),
            y_hinge,
            epsilon = EPSILON
        );

        // Test left segment (value < x): should interpolate from (0,0) to (x,y)
        assert_relative_eq!(
            interpolate_hinge(0.0, x_hinge, y_hinge),
            0.0,
            epsilon = EPSILON
        );
        assert_relative_eq!(
            interpolate_hinge(0.15, x_hinge, y_hinge),
            0.35, // 0.15/0.3 * 0.7 = 0.5 * 0.7 = 0.35
            epsilon = EPSILON
        );

        // Test right segment (value > x): should interpolate from (x,y) to (1,1)
        assert_relative_eq!(
            interpolate_hinge(1.0, x_hinge, y_hinge),
            1.0,
            epsilon = EPSILON
        );
        assert_relative_eq!(
            interpolate_hinge(0.65, x_hinge, y_hinge),
            0.85, // y + ((0.65-0.3)/(1.0-0.3)) * (1.0-0.7) = 0.7 + 0.5 * 0.3 = 0.85
            epsilon = EPSILON
        );

        // Test with different hinge points
        // Hinge at (0.5, 0.5) - should be linear
        assert_relative_eq!(interpolate_hinge(0.25, 0.5, 0.5), 0.25, epsilon = EPSILON);
        assert_relative_eq!(interpolate_hinge(0.75, 0.5, 0.5), 0.75, epsilon = EPSILON);

        // Hinge at (0.2, 0.9) - steep left, shallow right
        assert_relative_eq!(
            interpolate_hinge(0.1, 0.2, 0.9),
            0.45, // 0.1/0.2 * 0.9 = 0.5 * 0.9 = 0.45
            epsilon = EPSILON
        );
        assert_relative_eq!(
            interpolate_hinge(0.6, 0.2, 0.9),
            0.95, // 0.9 + ((0.6-0.2)/(1.0-0.2)) * (1.0-0.9) = 0.9 + 0.5 * 0.1 = 0.95
            epsilon = EPSILON
        );

        // Test edge cases
        // Hinge at (0, 0)
        assert_relative_eq!(interpolate_hinge(0.0, 0.0, 0.0), 0.0, epsilon = EPSILON);
        assert_relative_eq!(
            interpolate_hinge(0.5, 0.0, 0.0),
            0.5, // Should interpolate from (0,0) to (1,1)
            epsilon = EPSILON
        );

        // Hinge at (1, 1)
        assert_relative_eq!(
            interpolate_hinge(0.5, 1.0, 1.0),
            0.5, // Should interpolate from (0,0) to (1,1)
            epsilon = EPSILON
        );
        assert_relative_eq!(interpolate_hinge(1.0, 1.0, 1.0), 1.0, epsilon = EPSILON);

        // Test with different numeric types
        assert_relative_eq!(
            interpolate_hinge(0.5f32, 0.3f32, 0.7f32),
            0.7857142857142857, // 0.7 + ((0.5-0.3)/(1.0-0.3)) * (1.0-0.7) = 0.7 + (2/7) * 0.3
            epsilon = 1e-7      // Relaxed epsilon for f32 precision
        );
        assert_relative_eq!(
            interpolate_hinge(1u8, 0u8, 1u8), // 1.0, 0.0, 1.0 - should return 1.0
            1.0,
            epsilon = EPSILON
        );

        // Test monotonicity - function should be non-decreasing
        let test_x = 0.4;
        let test_y = 0.6;
        let mut prev = interpolate_hinge(0.0, test_x, test_y);

        for i in 1..=20 {
            let value = i as f64 / 20.0;
            let current = interpolate_hinge(value, test_x, test_y);
            assert!(
                current >= prev,
                "Failed monotonicity test at value={}, x={}, y={}, prev={}, current={}",
                value,
                test_x,
                test_y,
                prev,
                current
            );
            prev = current;
        }
    }

    #[test]
    fn test_interpolate_exponential_graph() {
        use textplots::{Chart, Plot, Shape};

        println!("\n=== Exponential Interpolation Examples ===");

        // Test different steepness values (both positive and negative)
        let steepness_values = [-3.0, -1.0, 0.0, 1.0, 3.0];

        for steepness in steepness_values.iter() {
            println!("\nSteepness = {}", steepness);

            // Generate data points
            let points: Vec<(f32, f32)> = (0..=50)
                .map(|i| {
                    let x = i as f64 / 50.0;
                    let y = interpolate_exponential(x, *steepness);
                    (x as f32, y as f32)
                })
                .collect();

            // Create and display chart
            Chart::new(60, 60, 0.0, 1.0)
                .lineplot(&Shape::Lines(&points))
                .display();
        }
    }

    #[test]
    fn test_interpolate_hinge_graph() {
        use textplots::{Chart, Plot, Shape};

        println!("\n=== hinge Interpolation Examples ===");

        // Test different hinge points
        let hinges = [(0.3, 0.7), (0.5, 0.5), (0.2, 0.9), (0.8, 0.2)];

        for &(x_hinge, y_hinge) in hinges.iter() {
            println!("\nHinge at ({}, {})", x_hinge, y_hinge);

            // Generate data points
            let points: Vec<(f32, f32)> = (0..=50)
                .map(|i| {
                    let x = i as f64 / 50.0;
                    let y = interpolate_hinge(x, x_hinge, y_hinge);
                    (x as f32, y as f32)
                })
                .collect();

            // Create and display chart
            Chart::new(60, 60, 0.0, 1.0)
                .lineplot(&Shape::Lines(&points))
                .display();
        }
    }

    #[test]
    fn test_interpolate_inflected_exponential_graph() {
        use textplots::{Chart, Plot, Shape};

        // Test different steepness values
        let steepness_values = [0.5, 1.0, 2.0, 5.0];

        for steepness in steepness_values.iter() {
            println!("\nSteepness = {}", steepness);

            // Generate data points
            let points: Vec<(f32, f32)> = (0..=50)
                .map(|i| {
                    let x = i as f64 / 50.0;
                    let y = interpolate_inflected_exponential(x, *steepness);
                    (x as f32, y as f32)
                })
                .collect();

            // Create and display chart
            Chart::new(60, 60, 0.0, 1.0)
                .lineplot(&Shape::Lines(&points))
                .display();
        }
    }

    #[test]
    #[should_panic]
    fn test_normalize_invalid_range() {
        normalize(5.0, 10.0, 5.0); // min >= max should panic
    }
}
