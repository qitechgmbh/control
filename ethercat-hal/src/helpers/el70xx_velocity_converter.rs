use crate::shared_config::el70x1::EL70x1SpeedRange;

#[derive(Debug)]
pub struct EL70x1VelocityConverter {
    max_steps_per_seconds: u16,
}

impl EL70x1VelocityConverter {
    pub const fn new(speed_range: &EL70x1SpeedRange) -> Self {
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
    pub fn steps_to_velocity(&self, steps_per_second: f64, propability_rounding: bool) -> i16 {
        // Calculate the velocity value (10000 = 100% of max speed)
        let velocity_f64 =
            (steps_per_second / self.max_steps_per_seconds as f64) * std::i16::MAX as f64;

        match propability_rounding {
            true => round_propabilistic(velocity_f64),
            false => velocity_f64.round() as i16,
        }
    }

    /// Convert i16 velocity value back to steps per second
    pub fn velocity_to_steps(&self, velocity: i16, propability_rounding: bool) -> i16 {
        // Calculate steps per second from velocity value
        let steps_per_second =
            (velocity as f64 / std::i16::MAX as f64) * self.max_steps_per_seconds as f64;

        match propability_rounding {
            true => round_propabilistic(steps_per_second),
            false => steps_per_second.round() as i16,
        }
    }
}

/// The [`EL70x1SpeedRange`] has a different maxmimum resoltion based on the range
/// For [`EL70x1SpeedRange::Steps1000`] it is 0.1 steps/second
/// For [`EL70x1SpeedRange::Steps2000`] it is 0.2 steps/second
/// For [`EL70x1SpeedRange::Steps4000`] it is 0.4 steps/second
/// For [`EL70x1SpeedRange::Steps8000`] it is 0.8 steps/second
/// For [`EL70x1SpeedRange::Steps16000`] it is 1.6 steps/second
/// For [`EL70x1SpeedRange::Steps32000`] it is 3.2 steps/second
///
/// For a stepper with 200 steps using the [`EL70x1SpeedRange::Steps32000`] the actual speed could be off up to 1.6 steps/second which is 0.8%
///
/// With probabilistic rounding the we rsometimes round up and sometimes down which can result in much greater accuracy given enough samples/cycles (thousands a second)
fn round_propabilistic(value: f64) -> i16 {
    let propability = value - value.floor();
    // make a random decision to add 1 based on the propability
    match rand::random_bool(propability) {
        true => value.ceil() as i16,
        false => value.floor() as i16,
    }
}

#[cfg(test)]
mod tests {
    use approx::assert_relative_eq;

    use super::*;

    #[test]
    fn test_steps_to_velocity_conversion() {
        let calc = EL70x1VelocityConverter::new(&EL70x1SpeedRange::Steps4000); // 4000 steps/s

        // 0 sps = 0% velocity
        assert_eq!(calc.steps_to_velocity(0.0, false), 0);
        // 1000 sps = 25% velocity = 8192 (25% of 32767)
        let v1 = calc.steps_to_velocity(1000.0, false);
        assert!((v1 - 8192i16).abs() <= 1, "Expected 8192±1, got {}", v1);
        // 2000 sps = 50% velocity = 16384 (50% of 32767)
        let v2 = calc.steps_to_velocity(2000.0, false);
        assert!((v2 - 16384i16).abs() <= 1, "Expected 16384±1, got {}", v2);
        // 3000 sps = 75% velocity = 24575 (75% of 32767)
        let v3 = calc.steps_to_velocity(3000.0, false);
        assert!((v3 - 24575i16).abs() <= 1, "Expected 24575±1, got {}", v3);
        // 4000 sps = 100% velocity = 32767 (100% of 32767)
        let v4 = calc.steps_to_velocity(4000.0, false);
        assert!((v4 - 32767i16).abs() <= 1, "Expected 32767±1, got {}", v4);
    }

    #[test]
    fn test_velocity_to_steps_conversion() {
        let calc = EL70x1VelocityConverter::new(&EL70x1SpeedRange::Steps4000); // 4000 steps/s

        // 0% velocity = 0i16 = 0 sps
        assert_eq!(calc.velocity_to_steps(0, false), 0);
        // 25% velocity = 8192i16 = 1000 sps
        let s1 = calc.velocity_to_steps(8192, false);
        assert_eq!((s1 - 1000i16).abs(), 0, "Expected 1000±0, got {}", s1);
        // 50% velocity = 16384i16 = 2000 sps
        let s2 = calc.velocity_to_steps(16384, false);
        assert_eq!((s2 - 2000i16).abs(), 0, "Expected 2000±0, got {}", s2);
        // 75% velocity = 24575i16 = 3000 sps
        let s3 = calc.velocity_to_steps(24575, false);
        assert_eq!((s3 - 3000i16).abs(), 0, "Expected 3000±0, got {}", s3);
        // 100% velocity = 32767i16 = 4000 sps
        let s4 = calc.velocity_to_steps(32767, false);
        assert_eq!((s4 - 4000i16).abs(), 0, "Expected 4000±0, got {}", s4);
    }

    #[test]
    fn test_round_propabilistic() {
        // Test the rounding function with a few values
        let mut sum = 0.0;
        for _ in 0..100000 {
            let rounded = round_propabilistic(0.5);
            sum += rounded as f64;
        }
        // Check that the average is close to 0.5
        let average = sum / 100000.0;
        assert_relative_eq!(average, 0.5, epsilon = 0.01);
    }
}
