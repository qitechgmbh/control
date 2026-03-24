use ethercat_hal::io::analog_input::{AnalogInput, physical::AnalogInputValue};
use units::angle::revolution;
use units::electric_potential::volt;
use units::f64::*;

#[derive(Debug)]
pub struct TensionArm {
    pub analog_input: AnalogInput,
    pub zero: Angle,
    /// was zeroed at least once
    pub zeroed: bool,
}

impl TensionArm {
    pub fn new(analog_input: AnalogInput) -> Self {
        Self {
            analog_input,
            zero: Angle::new::<revolution>(0.0),
            zeroed: false,
        }
    }

    fn volts_to_angle(&self, volts: f64) -> Angle {
        // 0V = 0deg 5V = 3600deg
        // always wrap into 0..1 revolution
        Angle::new::<revolution>(volts / 5.0) % Angle::new::<revolution>(1.0)
    }

    fn get_volts(&self) -> f64 {
        // get the normalized value from the analog input
        let value = self.analog_input.get_physical();

        match value {
            AnalogInputValue::Potential(v) => v.get::<volt>(),
            _ => panic!("Expected a potential value"),
        }
    }

    fn raw_angle(&self) -> Angle {
        // get volts
        let volts = self.get_volts();

        // 0V = 0deg 5V = 3600deg
        self.volts_to_angle(volts)
    }

    pub fn get_angle(&self) -> Angle {
        // revolution is maping -1/1 to 0/1
        let raw = self.raw_angle();

        // Handle the wraparound case
        if raw < self.zero {
            // We've wrapped around, so add a full revolution
            (raw + Angle::new::<revolution>(1.0)) - self.zero
        } else {
            // Normal case
            raw - self.zero
        }
    }

    pub fn zero(&mut self) {
        self.zero = self.raw_angle();
        self.zeroed = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;
    use core::f64;
    use ethercat_hal::io::{
        analog_input::{AnalogInputInput, physical::AnalogInputRange},
        analog_input_dummy::AnalogInputDummy,
    };
    use std::i16;

    #[test]
    fn volts_to_angle() {
        let tension_arm = TensionArm::new(
            AnalogInputDummy::new(AnalogInputRange::Potential {
                min: ElectricPotential::new::<volt>(0.0),
                max: ElectricPotential::new::<volt>(10.0),
                min_raw: 0,
                max_raw: i16::MAX,
            })
            .analog_input(),
        );

        // 0V = 0deg
        assert_relative_eq!(
            tension_arm.volts_to_angle(0.0).get::<revolution>(),
            0.0,
            epsilon = f64::EPSILON
        );

        // 5V = 1 revolution
        assert_relative_eq!(
            tension_arm.volts_to_angle(5.0).get::<revolution>(),
            0.0,
            epsilon = f64::EPSILON
        );

        // 10V = 2 revolution
        assert_relative_eq!(
            tension_arm.volts_to_angle(10.0).get::<revolution>(),
            0.0,
            epsilon = f64::EPSILON
        );
    }

    #[test]
    fn get_volts() {
        let mut analog_input_dummy = AnalogInputDummy::new(AnalogInputRange::Potential {
            min: ElectricPotential::new::<volt>(0.0),
            max: ElectricPotential::new::<volt>(10.0),
            min_raw: 0,
            max_raw: i16::MAX,
        });
        let analog_input = analog_input_dummy.analog_input();
        let tension_arm = TensionArm::new(analog_input);

        // 0.0 normalized = 0V
        let volts = tension_arm.get_volts();
        assert_relative_eq!(volts, 0.0, epsilon = f64::EPSILON);

        //  0.5 normalized = 5V
        analog_input_dummy.set_input(AnalogInputInput {
            // 5V of 10V in positive range
            normalized: (5.0 / 10.0),
            wiring_error: false,
        });
        let physical = tension_arm.analog_input.get_physical();
        match physical {
            AnalogInputValue::Potential(v) => {
                assert_relative_eq!(v.get::<volt>(), 5.0, epsilon = f64::EPSILON);
            }
            _ => panic!("Expected a potential value"),
        }
    }

    #[test]
    fn test_tension_arm() {
        let mut analog_input_dummy = AnalogInputDummy::new(AnalogInputRange::Potential {
            min: ElectricPotential::new::<volt>(0.0),
            max: ElectricPotential::new::<volt>(10.0),
            min_raw: 0,
            max_raw: i16::MAX,
        });
        let analog_input = analog_input_dummy.analog_input();
        let tension_arm = TensionArm::new(analog_input);

        // 0V = 0.25 = 0 revolution
        let angle = tension_arm.get_angle();
        assert_relative_eq!(angle.get::<revolution>(), 0.0, epsilon = f64::EPSILON);

        // 1.25V = 0.25 revolution
        analog_input_dummy.set_input(AnalogInputInput {
            normalized: (1.25 / 10.0),
            wiring_error: false,
        });
        let angle = tension_arm.raw_angle();
        assert_relative_eq!(angle.get::<revolution>(), 0.25, epsilon = f64::EPSILON);

        // 2.5V = 0.5 revolution
        analog_input_dummy.set_input(AnalogInputInput {
            normalized: (2.5 / 10.0),
            wiring_error: false,
        });
        let angle = tension_arm.raw_angle();
        assert_relative_eq!(angle.get::<revolution>(), 0.5, epsilon = f64::EPSILON);

        // 3.75V = 0.75 revolution
        analog_input_dummy.set_input(AnalogInputInput {
            normalized: (3.75 / 10.0),
            wiring_error: false,
        });
        let angle = tension_arm.raw_angle();
        assert_relative_eq!(angle.get::<revolution>(), 0.75, epsilon = f64::EPSILON);

        // 5V = 1 revolution
        analog_input_dummy.set_input(AnalogInputInput {
            normalized: (5.0 / 10.0),
            wiring_error: false,
        });
        let angle = tension_arm.raw_angle();
        assert_relative_eq!(angle.get::<revolution>(), 0.0, epsilon = f64::EPSILON);

        // 6.25V = 0.25 revolution
        analog_input_dummy.set_input(AnalogInputInput {
            normalized: (6.25 / 10.0),
            wiring_error: false,
        });
        let angle = tension_arm.raw_angle();
        assert_relative_eq!(angle.get::<revolution>(), 0.25, epsilon = f64::EPSILON);
    }
}
