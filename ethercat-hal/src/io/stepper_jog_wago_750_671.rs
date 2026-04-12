use smol::block_on;

use crate::devices::wago_modules::wago_750_671::InitState;
use crate::io::stepper_positioning_wago_750_671::StepperPositioningWago750671;
use crate::io::stepper_velocity_wago_750_671::{C1Mode, C3Flag};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Wago750671JogDirection {
    Positive,
    Negative,
}

pub struct StepperJogWago750671<'a> {
    stepper: &'a mut StepperPositioningWago750671,
}

impl<'a> StepperJogWago750671<'a> {
    pub fn new(stepper: &'a mut StepperPositioningWago750671) -> Self {
        Self { stepper }
    }

    pub fn activate(&mut self, timeout_ms: u16) {
        {
            let mut dev = block_on(self.stepper.device.write());
            dev.rxpdo.acceleration = timeout_ms;
        }
        let status = self.stepper.status();
        let start_requested = !status.jog_mode_ack || !status.start_ack;
        self.request_mode(C1Mode::JogMode, 0, start_requested);
    }

    pub fn jog(&mut self, direction: Wago750671JogDirection, timeout_ms: u16) {
        {
            let mut dev = block_on(self.stepper.device.write());
            dev.rxpdo.acceleration = timeout_ms;
        }

        let control_byte3 = match direction {
            Wago750671JogDirection::Positive => C3Flag::DirectionPos as u8,
            Wago750671JogDirection::Negative => C3Flag::DirectionNeg as u8,
        };

        let status = self.stepper.status();
        let start_requested = !status.jog_mode_ack || !status.start_ack;
        self.request_mode(C1Mode::JogMode, control_byte3, start_requested);
    }

    pub fn stop(&mut self) {
        let mut dev = block_on(self.stepper.device.write());
        dev.desired_mode = C1Mode::JogMode;
        dev.desired_stop2_n = true;
        dev.desired_control_byte3 = 0;
        dev.start_requested = false;
    }

    pub fn request_fast_stop(&mut self) {
        self.stepper.request_fast_stop();
    }

    pub fn clear_fast_stop(&mut self) {
        let mut dev = block_on(self.stepper.device.write());
        dev.desired_stop2_n = true;
    }

    fn request_mode(&mut self, mode: C1Mode, control_byte3: u8, start_requested: bool) {
        let mut dev = block_on(self.stepper.device.write());
        let sanitized_control_byte3 = control_byte3 & !(C3Flag::ResetQuit as u8);
        let unchanged = dev.desired_mode as u8 == mode as u8
            && dev.desired_stop2_n
            && dev.desired_control_byte3 == sanitized_control_byte3
            && dev.start_requested == start_requested;
        if unchanged {
            return;
        }

        dev.desired_mode = mode;
        dev.desired_stop2_n = true;
        dev.desired_control_byte3 = sanitized_control_byte3;
        dev.start_requested = start_requested;
        if self.stepper.enabled {
            dev.initialized = false;
            dev.state = InitState::SetMode;
            self.stepper.state = InitState::SetMode;
        }
    }
}
