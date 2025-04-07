use ethercat_hal::shared_config::el70x1::EL70x1SpeedRange;

#[derive(Debug)]
pub struct EL70x1VelocityCalculator {
    max_steps_per_seconds: u16,
}

impl EL70x1VelocityCalculator {
    pub fn new(speed_range: &EL70x1SpeedRange) -> Self {
        let speed_range_value = match speed_range {
            EL70x1SpeedRange::Steps1000 => 1000,
            EL70x1SpeedRange::Steps2000 => 2000,
            EL70x1SpeedRange::Steps4000 => 4000,
            EL70x1SpeedRange::Steps8000 => 8000,
            EL70x1SpeedRange::Steps16000 => 16000,
            EL70x1SpeedRange::Steps32000 => 32000,
        };
        Self {
            max_steps_per_seconds: speed_range_value,
        }
    }

    /// Convert steps per second to the i16 velocity value used by the EL7031
    pub fn steps_to_velocity(&self, steps_per_second: i32) -> i16 {
        // Calculate the velocity value (10000 = 100% of max speed)
        let velocity_f64 =
            (steps_per_second as f64 / self.max_steps_per_seconds as f64) * i16::MAX as f64;

        // Clamp to i16 range to be safe
        velocity_f64.round() as i16
    }

    /// Convert i16 velocity value back to steps per second
    pub fn velocity_to_steps(&self, velocity: i16) -> i16 {
        // Calculate steps per second from velocity value
        let steps_per_second =
            (velocity as f64 / i16::MAX as f64) * self.max_steps_per_seconds as f64;

        steps_per_second.round() as i16
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_steps_to_velocity_conversion() {
        let calc = EL70x1VelocityCalculator::new(&EL70x1SpeedRange::Steps4000); // 4000 steps/s

        // 0 sps = 0% velocity
        assert_eq!(calc.steps_to_velocity(0), 0);
        // 1000 sps = 25% velocity = 8192
        let v1 = calc.steps_to_velocity(1000);
        assert!((v1 - 8192i16).abs() <= 1, "Expected 8192±1, got {}", v1);
        // 2000 sps = 50% velocity = 16384
        let v2 = calc.steps_to_velocity(2000);
        assert!((v2 - 16384i16).abs() <= 1, "Expected 16384±1, got {}", v2);
        // 3000 sps = 75% velocity = 24576
        let v3 = calc.steps_to_velocity(3000);
        assert!((v3 - 24576i16).abs() <= 1, "Expected 24576±1, got {}", v3);
        // 4000 sps = 100% velocity = 32767
        let v4 = calc.steps_to_velocity(4000);
        assert!((v4 - 32767i16).abs() <= 1, "Expected 32767±1, got {}", v4);
    }

    #[test]
    fn test_velocity_to_steps_conversion() {
        let calc = EL70x1VelocityCalculator::new(&EL70x1SpeedRange::Steps4000); // 4000 steps/s

        // 0% velocity = 0i16 = 0 sps
        assert_eq!(calc.velocity_to_steps(0), 0);
        // 25% velocity = 8192i16 = 1000 sps
        let s1 = calc.velocity_to_steps(8192);
        assert_eq!((s1 - 1000i16).abs(), 0, "Expected 1000±1, got {}", s1);
        // 50% velocity = 16384i16 = 2000 sps
        let s2 = calc.velocity_to_steps(16384);
        assert_eq!((s2 - 2000i16).abs(), 0, "Expected 2000±1, got {}", s2);
        // 75% velocity = 24576i16 = 3000 sps
        let s3 = calc.velocity_to_steps(24576);
        assert_eq!((s3 - 3000i16).abs(), 0, "Expected 3000±1, got {}", s3);
        // 100% velocity = 32767i16 = 4000 sps
        let s4 = calc.velocity_to_steps(32767);
        assert_eq!((s4 - 4000i16).abs(), 0, "Expected 4000±1, got {}", s4);
    }
}
