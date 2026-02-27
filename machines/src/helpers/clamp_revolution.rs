use units::angle::revolution;
use units::f64::*;

use super::{Clamp, ClampedValue};

pub fn clamp_revolution_uom(value: Angle, min: Angle, max: Angle) -> ClampedValue<Angle>
{
    let value = value.get::<revolution>();
    let min = min.get::<revolution>();
    let max = max.get::<revolution>();

    // clamp
    let clamped_value = clamp_revolution(value, min, max);

    // convert back to uom
    ClampedValue { 
        value: Angle::new::<revolution>(clamped_value.value), 
        clamp: clamped_value.clamp 
    }
}

pub fn clamp_revolution(value: f64, min: f64, max: f64) -> ClampedValue<f64>
{
    // normalize value from 0..1
    let value = wrap_revolution(value);
    let min = wrap_revolution(min);
    let max = wrap_revolution(max);

    // check if in acceptable range
    if revolution_in_range(value, min, max) {
        return ClampedValue::new(value, Clamp::None);
    }

    // calculates the distance between min and max
    let (clamp_to_min_min, clamp_to_min_max, clamp_to_max_min, clamp_to_max_max) =
        clamping_ranges(min, max);

    // check if in min clamping  range
    if revolution_in_range(value, clamp_to_min_min, clamp_to_min_max) {
        return ClampedValue::new(min, Clamp::Min);
    }
    // check if in max clamping  range
    if revolution_in_range(value, clamp_to_max_min, clamp_to_max_max) {
        return ClampedValue::new(max, Clamp::Max);
    }

    // at this point our input value should be either retured (cause in spec) or clamped to min or max
    // so this point should never be reached
    // in case it does we just clamp to min
    ClampedValue::new(min, Clamp::Min)
    //TODO: consider using unreachable!() if truly unreachable
}

pub fn scale_revolution_to_range(value: f64, min: f64, max: f64) -> f64 
{
    // we calculate the distance between min and max
    let distance = revolution_distance(min, max);

    // we scale the value to the distance

    (value - min) / distance
}

fn clamping_ranges(min: f64, max: f64) -> (f64, f64, f64, f64) 
{
    // normalize min and max
    let min = wrap_revolution(min);
    let max = wrap_revolution(max);

    // calculates the distance between min and max (distance A in the test)
    let in_spec_distance = revolution_distance(min, max);

    // calculate distance B and clamping distance as per the test comment
    let out_spec_distance = 1.0 - in_spec_distance;
    let clamping_distance = out_spec_distance / 2.0;

    let clamp_to_min_min = wrap_revolution(min - clamping_distance);
    let clamp_to_min_max = min;
    let clamp_to_max_min = max;
    let clamp_to_max_max = wrap_revolution(max + clamping_distance);

    (
        clamp_to_min_min,
        clamp_to_min_max,
        clamp_to_max_min,
        clamp_to_max_max,
    )
}

fn revolution_distance(min: f64, max: f64) -> f64 
{
    // Normalize the values to ensure they're in the [0, 1) range
    let normalized_min = wrap_revolution(min);
    let normalized_max = wrap_revolution(max);

    // Check if the range crosses zero
    if normalized_min > normalized_max {
        // For ranges that cross zero (e.g., min = 0.9, max = 0.1)
        // The distance is (1 - min) + max
        1.0 - normalized_min + normalized_max
    } else {
        // For normal ranges (e.g., min = 0.1, max = 0.3)
        // The distance is simply max - min
        normalized_max - normalized_min
    }
}

fn wrap_revolution(value: f64) -> f64 
{
    debug_assert!(value.is_finite(), "value must be finite");

    match value % 1.0 
    {
        0.0 if value >= 1.0 => 1.0,
        n if n < 0.0 => n + 1.0,
        n => n,
    }
}

fn revolution_in_range(value: f64, min: f64, max: f64) -> bool 
{
    debug_assert!(value.is_finite(), "value must be finite");
    debug_assert!(min.is_finite(), "min must be finite");
    debug_assert!(max.is_finite(), "max must be finite");

    debug_assert!(value >= 0.0 && value < 1.0, "value out of [0,1)");
    debug_assert!(min   >= 0.0 && min   < 1.0, "min out of [0,1)");
    debug_assert!(max   >= 0.0 && max   < 1.0, "max out of [0,1)");

    let wraps = min > max;

    if wraps {
        // Range wraps across boundary (e.g., 0.9 → 0.1)
        min <= value || value <= max
    } else {
        // Standard interval
        min <= value && value <= max
    }
}