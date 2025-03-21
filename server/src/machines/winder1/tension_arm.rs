use ethercat_hal::actors::analog_input_getter::AnalogInputGetter;
use uom::si::{angle::revolution, f32::Angle};

#[derive(Debug)]
pub struct TensionArm {
    pub analog_input_getter: AnalogInputGetter,
    pub zero: Angle,
}

impl TensionArm {
    pub fn new(analog_input_getter: AnalogInputGetter) -> Self {
        Self {
            analog_input_getter,
            zero: Angle::new::<revolution>(0.0),
        }
    }

    fn raw_angle(&self) -> Angle {
        // get the normalized value from the analog input
        let normalized = self.analog_input_getter.get_normalized().unwrap_or(0.0);
        // to angle
        Angle::new::<revolution>(normalized / 2.0 + 0.5)
    }

    pub fn get_angle(&self) -> Angle {
        // revolution is maping -1/1 to 0/1
        let raw = self.raw_angle();

        // modulo 1 revolution
        (raw - self.zero) % Angle::new::<revolution>(1.0)
    }

    pub fn zero(&mut self) {
        self.zero = self.raw_angle();
    }
}
