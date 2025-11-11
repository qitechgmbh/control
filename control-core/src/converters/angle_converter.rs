use units::{angle::radian, f64::Angle};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AngleConverter {
    pub flip_x: bool, // true to flip X-axis (mirror horizontally)
    pub flip_y: bool, // true to flip Y-axis (mirror vertically)
    pub is_cw: bool,  // true for clockwise, false for counter-clockwise
}

impl AngleConverter {
    /// Create a custom coordinate system
    pub const fn new(flip_x: bool, flip_y: bool, is_cw: bool) -> Self {
        Self {
            flip_x,
            flip_y,
            is_cw,
        }
    }

    /// Standard mathematical coordinate system (CCW positive, 0° at positive X)
    pub const fn mathematical() -> Self {
        Self::new(false, false, false)
    }

    /// Screen/graphics coordinate system (CW positive, 0° at positive X, Y-flipped)
    pub const fn screen() -> Self {
        Self::new(false, true, false)
    }

    /// System where 0° points up, clockwise positive
    pub const fn y_up_cw() -> Self {
        Self::new(false, false, true)
    }

    /// System where 0° points down, counter-clockwise positive
    pub const fn y_down_ccw() -> Self {
        Self::new(false, true, true)
    }

    /// System where 0° points left, clockwise positive
    pub const fn x_left_cw() -> Self {
        Self::new(true, false, false)
    }

    /// System where 0° points right, counter-clockwise positive (same as mathematical)
    pub const fn x_right_ccw() -> Self {
        Self::mathematical()
    }

    /// System where 0° points up, counter-clockwise positive
    pub const fn y_up_ccw() -> Self {
        Self::new(false, false, false)
    }

    /// System where 0° points down, clockwise positive
    pub const fn y_down_cw() -> Self {
        Self::new(false, true, false)
    }

    /// System where 0° points left, counter-clockwise positive
    pub const fn x_left_ccw() -> Self {
        Self::new(true, false, true)
    }

    /// Convert angle from mathematical system to this system (f32)
    pub fn degrees_encode(&self, math_angle_degrees: f64) -> f64 {
        let normalized_input = self.normalize_angle(math_angle_degrees);

        let mut result = normalized_input;

        // Apply axis flips
        match (self.flip_x, self.flip_y) {
            (false, false) => {}                      // No change
            (true, false) => result = 180.0 - result, // X-flip: mirror across Y-axis
            (false, true) => result = -result,        // Y-flip: mirror across X-axis
            (true, true) => result += 180.0,          // Both flips: 180° rotation
        }

        // Handle rotation direction
        if self.is_cw {
            result = -result;
        }

        self.normalize_angle(result)
    }

    /// Convert angle from this system to mathematical system (f32)
    pub fn degrees_decode(&self, system_angle_degrees: f64) -> f64 {
        let normalized_input = self.normalize_angle(system_angle_degrees);

        let mut result = normalized_input;

        // Handle rotation direction
        if self.is_cw {
            result = -result;
        }

        // Apply inverse axis flips
        match (self.flip_x, self.flip_y) {
            (false, false) => {}                      // No change
            (true, false) => result = 180.0 - result, // X-flip: mirror across Y-axis
            (false, true) => result = -result,        // Y-flip: mirror across X-axis
            (true, true) => result -= 180.0,          // Both flips: inverse 180° rotation
        }

        self.normalize_angle(result)
    }

    /// Convert angle in radians from mathematical system to this system (f32)
    pub fn radians_encode(&self, math_angle_radians: f64) -> f64 {
        let degrees = math_angle_radians.to_degrees();
        let converted_degrees = self.degrees_encode(degrees);
        converted_degrees.to_radians()
    }

    /// Convert angle in radians from this system to mathematical system (f32)
    pub fn radians_decode(&self, system_angle_radians: f64) -> f64 {
        let degrees = system_angle_radians.to_degrees();
        let converted_degrees = self.degrees_decode(degrees);
        converted_degrees.to_radians()
    }

    /// Convert angle from mathematical system to this system (f64)
    pub fn degrees_encode_f64(&self, math_angle_degrees: f64) -> f64 {
        let normalized_input = self.normalize_angle_f64(math_angle_degrees);

        let mut result = normalized_input;

        // Apply axis flips
        match (self.flip_x, self.flip_y) {
            (false, false) => {}                      // No change
            (true, false) => result = 180.0 - result, // X-flip: mirror across Y-axis
            (false, true) => result = -result,        // Y-flip: mirror across X-axis
            (true, true) => result += 180.0,          // Both flips: 180° rotation
        }

        // Handle rotation direction
        if self.is_cw {
            result = -result;
        }

        self.normalize_angle_f64(result)
    }

    /// Convert angle from this system to mathematical system (f64)
    pub fn degrees_decode_f64(&self, system_angle_degrees: f64) -> f64 {
        let normalized_input = self.normalize_angle_f64(system_angle_degrees);

        let mut result = normalized_input;

        // Handle rotation direction
        if self.is_cw {
            result = -result;
        }

        // Apply inverse axis flips
        match (self.flip_x, self.flip_y) {
            (false, false) => {}                      // No change
            (true, false) => result = 180.0 - result, // X-flip: mirror across Y-axis
            (false, true) => result = -result,        // Y-flip: mirror across X-axis
            (true, true) => result -= 180.0,          // Both flips: inverse 180° rotation
        }

        self.normalize_angle_f64(result)
    }

    /// Convert angle in radians from mathematical system to this system (f64)
    pub fn radians_encode_f64(&self, math_angle_radians: f64) -> f64 {
        let degrees = math_angle_radians.to_degrees();
        let converted_degrees = self.degrees_encode_f64(degrees);
        converted_degrees.to_radians()
    }

    /// Convert angle in radians from this system to mathematical system (f64)
    pub fn radians_decode_f64(&self, system_angle_radians: f64) -> f64 {
        let degrees = system_angle_radians.to_degrees();
        let converted_degrees = self.degrees_decode_f64(degrees);
        converted_degrees.to_radians()
    }

    /// Normalize angle to [0, 360) range (f32)
    fn normalize_angle(&self, angle: f64) -> f64 {
        let mut normalized = angle % 360.0;
        if normalized < 0.0 {
            normalized += 360.0;
        }
        normalized
    }

    /// Normalize angle to [0, 360) range (f64)
    fn normalize_angle_f64(&self, angle: f64) -> f64 {
        let mut normalized = angle % 360.0;
        if normalized < 0.0 {
            normalized += 360.0;
        }
        normalized
    }
}

#[derive(Debug, Clone)]
pub struct AngleConverterUom {
    angle_converter: AngleConverter,
}

impl AngleConverterUom {
    pub const fn new(angle_converter: AngleConverter) -> Self {
        Self { angle_converter }
    }

    pub fn encode(&self, angle: Angle) -> Angle {
        Angle::new::<radian>(self.angle_converter.radians_encode(angle.get::<radian>()))
    }

    pub fn decode(&self, angle: Angle) -> Angle {
        Angle::new::<radian>(self.angle_converter.radians_decode(angle.get::<radian>()))
    }
}

#[cfg(test)]
mod tests {
    use std::f32::consts::PI;

    use super::*;

    #[test]
    fn test_constructors() {
        let math = AngleConverter::mathematical();
        assert_eq!(math.flip_x, false);
        assert_eq!(math.flip_y, false);
        assert_eq!(math.is_cw, false);

        let screen = AngleConverter::screen();
        assert_eq!(screen.flip_x, false);
        assert_eq!(screen.flip_y, true);
        assert_eq!(screen.is_cw, false);
    }

    #[test]
    fn test_screen_converter_bidirectional() {
        let converter = AngleConverter::screen();

        // Math to screen (Y-flip only)
        assert_eq!(converter.degrees_encode(0.0), 0.0); // Right stays right
        assert_eq!(converter.degrees_encode(90.0), 270.0); // Up becomes down
        assert_eq!(converter.degrees_encode(270.0), 90.0); // Down becomes up
        assert_eq!(converter.degrees_encode(180.0), 180.0); // Left stays left

        // Screen to math (should be inverse)
        assert_eq!(converter.degrees_decode(0.0), 0.0); // Right stays right
        assert_eq!(converter.degrees_decode(270.0), 90.0); // Down becomes up
        assert_eq!(converter.degrees_decode(90.0), 270.0); // Up becomes down
        assert_eq!(converter.degrees_decode(180.0), 180.0); // Left stays left
    }

    #[test]
    fn test_x_flip_converter() {
        let converter = AngleConverter::x_left_cw();

        // Math to x-flipped system
        assert_eq!(converter.degrees_encode(0.0), 180.0); // Right becomes left
        assert_eq!(converter.degrees_encode(90.0), 90.0); // Up stays up
        assert_eq!(converter.degrees_encode(180.0), 0.0); // Left becomes right
        assert_eq!(converter.degrees_encode(270.0), 270.0); // Down stays down
    }

    #[test]
    fn test_both_flips_converter() {
        let converter = AngleConverter::new(true, true, false);

        // Math to both-flipped system (180° rotation)
        assert_eq!(converter.degrees_encode(0.0), 180.0); // Right becomes left
        assert_eq!(converter.degrees_encode(90.0), 270.0); // Up becomes down
        assert_eq!(converter.degrees_encode(180.0), 0.0); // Left becomes right
        assert_eq!(converter.degrees_encode(270.0), 90.0); // Down becomes up
    }

    #[test]
    fn test_radians_conversion() {
        let converter = AngleConverter::screen();

        // Test radians_encode
        let result = converter.radians_encode(PI as f64 / 2.0) as f64; // 90° in radians
        let expected: f64 = 270.0f64 * PI as f64 / 180.0f64; // 270° in radians
        assert!((result - expected).abs() < 1e-6);
    }

    #[test]
    fn test_roundtrip_conversion() {
        let converter = AngleConverter::new(true, false, true); // X-flip + CW

        let original = 75.0_f64;
        let converted = converter.degrees_encode(original);
        let roundtrip = converter.degrees_decode(converted);

        assert!((original - roundtrip as f64).abs() < 1e-6);
    }
}
