/// Normalize a tension-arm angle for control/graph display.
///
/// Treats 360° wraparound as a small negative angle around the zero point.
pub fn normalize_angle_deg(angle_deg: f64) -> f64 {
    if angle_deg >= 270.0 {
        angle_deg - 360.0
    } else {
        angle_deg
    }
}

#[cfg(test)]
mod tests {
    use super::normalize_angle_deg;

    #[test]
    fn keeps_low_angles_unchanged() {
        assert_eq!(normalize_angle_deg(0.0), 0.0);
        assert_eq!(normalize_angle_deg(120.0), 120.0);
        assert_eq!(normalize_angle_deg(269.99), 269.99);
    }

    #[test]
    fn wraps_upper_quadrant() {
        assert_eq!(normalize_angle_deg(270.0), -90.0);
        assert_eq!(normalize_angle_deg(315.0), -45.0);
        assert_eq!(normalize_angle_deg(359.0), -1.0);
    }
}
