use std::sync::Arc;

use smol::{block_on, lock::RwLock};

use crate::devices::wago_modules::wago_750_671::{ControlByteC1, ControlByteC2, Wago750_671};

/*
 * Wago Stepper Velocity Wrapper around Stepper Controller
 *
 */
#[derive(Debug)]
pub struct StepperVelocityWago750671 {
    pub device: Arc<RwLock<Wago750_671>>,
    pub state: SpeedControlState,
    pub target_velocity: i16,
    pub target_acceleration: u16,
    pub enabled: bool,
}

impl StepperVelocityWago750671 {
    pub fn new(device: Arc<RwLock<Wago750_671>>) -> Self {
        Self {
            device,
            state: SpeedControlState::Init,
            target_velocity: 0,
            target_acceleration: 10,
            enabled: false,
        }
    }
    pub fn tick(&mut self) {
        let mut dev = block_on(self.device.write());

        match self.state {
            SpeedControlState::Init => {
                dev.write_control_bits(ControlByteC1::ENABLE | ControlByteC1::STOP2_N, 0, 0);
                self.state = SpeedControlState::WaitReady;
            }
            SpeedControlState::WaitReady => {
                if dev.ready() && dev.stop_n_ack() {
                    self.state = SpeedControlState::SelectMode;
                }
            }
            SpeedControlState::SelectMode => {
                dev.write_control_bits(ControlByteC1::ENABLE | ControlByteC1::M_SPEED_CONTROL, 0, 0);
                if dev.speed_mode_ack() {
                    self.state = SpeedControlState::StartPulse;
                }
            }
            SpeedControlState::StartPulse => {
                dev.write_control_bits(ControlByteC1::ENABLE | ControlByteC1::M_SPEED_CONTROL | ControlByteC1::START, 0, 0);
                self.state = SpeedControlState::Running;
            }
            SpeedControlState::Running => {
                dev.set_speed_setpoint(self.target_velocity, self.target_acceleration);
            }
            SpeedControlState::ErrorAck => {
                dev.write_control_bits(ControlByteC1::ENABLE, ControlByteC2::ERROR_QUIT, 0);
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

