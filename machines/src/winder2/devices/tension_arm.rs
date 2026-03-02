use units::{Angle, ConstZero, ElectricPotential, angle::revolution, electric_potential::volt};
use ethercat_hal::io::{analog_input::{AnalogInput, physical::AnalogInputValue}};

#[derive(Debug)]
pub struct TensionArm
{
    input:       AnalogInput,
    zero_offset: Option<Angle>,
}

impl TensionArm
{
    const VOLTS_PER_REVOLUTION: f64 = 5.0;

    pub fn new(input: AnalogInput) -> Self 
    {
        Self { input, zero_offset: None }
    }

    pub fn zero(&mut self) 
    {
        self.zero_offset = Some(self.raw_angle());
    }

    pub fn is_zeroed(&self) -> bool
    {
        self.zero_offset.is_some()
    }

    pub fn angle(&self) -> Angle
    {
        let Some(zero_offset) = self.zero_offset else { return Angle::ZERO };

        let mut raw = self.raw_angle();

        // wrap guard
        if raw < zero_offset 
        {
            raw += Angle::new::<revolution>(1.0);
        }

        raw - zero_offset
    }
}

impl TensionArm
{
    fn raw_angle(&self) -> Angle 
    {
        let volts = self.read_volts();
        Self::volts_to_angle(volts.get::<volt>())
    }

    fn volts_to_angle(volts: f64) -> Angle 
    {
        let revolutions = volts / Self::VOLTS_PER_REVOLUTION;
        
        // Wrap into 0..1 revolution
        Angle::new::<revolution>(revolutions) % Angle::new::<revolution>(1.0)
    }

    fn read_volts(&self) -> ElectricPotential
    {
        use AnalogInputValue::*;

        match self.input.get_physical() 
        {
            Potential(v) => v,
            _ => panic!("Expected voltage, got current"),
        }
    }
}

#[cfg(test)]
mod tests
{
    use approx::assert_relative_eq;
    use units::{Angle, ElectricPotential, angle::revolution, electric_potential::volt};
    use core::f64;
    use ethercat_hal::io::{
        analog_input::{AnalogInputInput, physical::AnalogInputRange},
        analog_input_dummy::AnalogInputDummy,
    };

    use super::*;

    #[test]
    fn volts_to_angle() 
    {
        // 0V = 0 revolutions
        validate_angle(TensionArm::volts_to_angle(0.0), 0.0);

        // 5V = 1 revolution -> wrap to 0
        validate_angle(TensionArm::volts_to_angle(5.0), 0.0);

        // 10V = 2 revolutions -> wrap to 0
        validate_angle(TensionArm::volts_to_angle(10.0), 0.0);
    }

    #[test]
    fn test_tension_arm() 
    {
        let mut dummy_sensor = new_analog_input_dummy();
        let input = dummy_sensor.analog_input();
        let tension_arm = TensionArm::new(input);

        // 0V = 0.25 = 0 revolution
        validate_angle(tension_arm.raw_angle(), 0.0);

        // 1.25V = 0.25 revolution
        dummy_sensor_set_volt(&mut dummy_sensor, 1.25);
        validate_angle(tension_arm.raw_angle(), 0.25);

        // 2.5V = 0.5 revolution
        dummy_sensor_set_volt(&mut dummy_sensor, 2.5);
        validate_angle(tension_arm.raw_angle(), 0.5);

        // 3.75V = 0.75 revolution
        dummy_sensor_set_volt(&mut dummy_sensor, 3.75);
        validate_angle(tension_arm.raw_angle(), 0.75);

        // 5V = 1 revolution
        dummy_sensor_set_volt(&mut dummy_sensor, 5.0);
        validate_angle(tension_arm.raw_angle(), 0.0);

        // 6.25V = 0.25 revolution
        dummy_sensor_set_volt(&mut dummy_sensor, 6.25);
        validate_angle(tension_arm.raw_angle(), 0.25);
    }

    // utils
    fn new_analog_input_dummy() -> AnalogInputDummy
    {
        let range = AnalogInputRange::Potential {
            min: ElectricPotential::new::<volt>(0.0),
            max: ElectricPotential::new::<volt>(1.0),
            min_raw: 0,
            max_raw: i16::MAX,
        };

        AnalogInputDummy::new(range)
    }

    fn dummy_sensor_set_volt(dummy: &mut AnalogInputDummy, value: f64)
    {
        let input = AnalogInputInput {
            normalized: value as f32 / 10.0,
            wiring_error: false,
        };

        dummy.set_input(input);
    }

    fn validate_float(lhs: f64, rhs: f64)
    {
        assert_relative_eq!(lhs, rhs, epsilon = f64::EPSILON);
    }

    fn validate_angle(angle: Angle, value: f64)
    {
        validate_float(angle.get::<revolution>(), value);
    }
}