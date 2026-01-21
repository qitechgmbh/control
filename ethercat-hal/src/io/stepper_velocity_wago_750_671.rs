use std::sync::Arc;

use smol::{block_on, lock::RwLock};

use crate::devices::wago_modules::wago_750_671::Wago750_671;

/*
 * Wago Stepper Velocity Wrapper around Stepper Controller
 *
 */
pub struct StepperVelocityWago750671 {
    device: Arc<RwLock<Wago750_671>>,
    state: SpeedControlState,
    desired_velocity: i16,
    desired_acceleration: u16,
    enabled: bool,
}

impl StepperVelocityWago750671 {
    pub fn tick(&mut self) {
        let mut dev = block_on(self.device.write());

        let status = dev.status();

        match self.state {
            SpeedControlState::Init => {
                dev.write_control_bits(ENABLE | STOP2_N, 0, 0);
                self.state = SpeedControlState::WaitReady;
            }
            SpeedControlState::WaitReady => {
                if status.ready && status.stop_n_ack {
                    self.state = SpeedControlState::SelectMode;
                }
            }
            SpeedControlState::SelectMode => {
                dev.write_control_bits(ENABLE | SPEED_MODE, 0, 0);
                if status.speed_mode_ack {
                    self.state = SpeedControlState::StartPulse;
                }
            }
            SpeedControlState::StartPulse => {
                dev.write_control_bits(ENABLE | SPEED_MODE | START, 0, 0);
                self.state = SpeedControlState::Running;
            }
            SpeedControlState::Running => {
                dev.write_raw_velocity(self.desired_velocity, self.desired_acceleration);
            }
            SpeedControlState::ErrorAck => {
                dev.write_control_bits(ENABLE, ERROR_QUIT, 0);
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum SpeedControlState {
    Init,
    WaitReady,
    SelectMode,
    StartPulse,
    Running,
    ErrorAck,
}

