use std::sync::Arc;

use smol::{block_on, lock::RwLock};

use crate::devices::wago_modules::wago_750_671::Wago750_671;

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
            target_velocity: 1000,
            target_acceleration: 10000,
            enabled: false,
        }
    }
    pub fn tick(&mut self) {
        let mut dev = block_on(self.device.write());

        match self.state {
            SpeedControlState::Init => {
                dev.write_control_bits(
                    ControlByteC1::ENABLE | ControlByteC1::STOP2_N,
                    ControlByteC2::ERROR_QUIT,
                    ControlByteC3::RESET_QUIT,
                );
                self.state = SpeedControlState::WaitReady;
            }
            SpeedControlState::WaitReady => {
                dev.write_control_bits(ControlByteC1::ENABLE | ControlByteC1::STOP2_N, 0, 0);
                if dev.ready() && dev.stop_n_ack() {
                    self.state = SpeedControlState::SelectMode;
                }
            }
            SpeedControlState::SelectMode => {
                dev.write_control_bits(
                    ControlByteC1::ENABLE | ControlByteC1::M_SPEED_CONTROL | ControlByteC1::STOP2_N,
                    0,
                    0,
                );
                if dev.speed_mode_ack() {
                    self.state = SpeedControlState::StartPulse;
                }
            }
            SpeedControlState::StartPulse => {
                dev.write_control_bits(
                    ControlByteC1::ENABLE
                        | ControlByteC1::M_SPEED_CONTROL
                        | ControlByteC1::STOP2_N
                        | ControlByteC1::START,
                    0,
                    0,
                );
                self.state = SpeedControlState::Running;
            }
            SpeedControlState::Running => {
                dev.write_control_bits(
                    ControlByteC1::ENABLE | ControlByteC1::M_SPEED_CONTROL | ControlByteC1::STOP2_N,
                    0,
                    0,
                );
                dev.set_speed_setpoint(self.target_velocity, self.target_acceleration);
            }
            SpeedControlState::ErrorAck => {
                dev.write_control_bits(ControlByteC1::ENABLE, ControlByteC2::ERROR_QUIT, 0);
            }
        }
    }
}

/// Control Byte C1
/// Bits 0 - 7
pub struct ControlByteC1;
impl ControlByteC1 {
    pub const ENABLE: u8 = 0x01;
    pub const STOP2_N: u8 = 0x02;
    pub const START: u8 = 0x04;
    pub const M_SPEED_CONTROL: u8 = 0x08;
    pub const M_PROGRAM: u8 = 0x10;
    pub const M_REFERENCE: u8 = 0x20;
    pub const M_JOG: u8 = 0x40;
    pub const M_DRIVE_BYMBX: u8 = 0x80;
}

/// Control Byte C2
/// Bits 0 - 7
pub struct ControlByteC2;
impl ControlByteC2 {
    pub const FREQ_RANGE_SEL_LSB: u8 = 0x01;
    pub const FREQ_RANGE_SEL_MSB: u8 = 0x02;
    pub const ACCELERATION_RANGE_SEL_LSB: u8 = 0x04;
    pub const ACCELERATION_RANGE_SEL_MSB: u8 = 0x08;
    // RESERVED
    // RESERVED
    pub const PRE_CALC: u8 = 0x40;
    pub const ERROR_QUIT: u8 = 0x80;
}

/// Control Byte C3
/// Bits 0 - 7
pub struct ControlByteC3;
impl ControlByteC3 {
    // RESERVED
    // RESERVED
    // RESERVED
    // RESERVED
    pub const LIMIT_SWITCH_POS: u8 = 0x10;
    pub const LIMIT_SWITCH_NEG: u8 = 0x20;
    pub const SETUP_SPEED_ACTIVE: u8 = 0x40;
    pub const RESET_QUIT: u8 = 0x80;
}

/// Status Byte S1
/// Bits 0 - 7
pub struct StatusByteS1;
impl StatusByteS1 {
    pub const READY: u8 = 0x01;
    pub const STOP_N_ACK: u8 = 0x02;
    pub const START_ACK: u8 = 0x04;
    pub const M_SPEED_CONTROL_ACK: u8 = 0x08;
    pub const M_PROGRAM_ACK: u8 = 0x10;
    pub const M_REFERENCE_ACK: u8 = 0x20;
    pub const M_JOG_ACK: u8 = 0x40;
    pub const M_DRIVE_BYMBX_ACK: u8 = 0x80;
}

/// Status Byte S2
/// Bits 0 - 7
pub struct StatusByteS2;
impl StatusByteS2 {
    // RESERVED
    pub const BUSY: u8 = 0x02;
    pub const STAND_STILL: u8 = 0x04;
    pub const ON_SPEED: u8 = 0x08;
    pub const DIRECTION: u8 = 0x10;
    // RESERVED
    pub const PRE_CALC_ACK: u8 = 0x40;
    pub const ERROR: u8 = 0x80;
}

/// Status Byte S3
/// Bits 0 - 7
pub struct StatusByteS3;
impl StatusByteS3 {
    pub const INPUT1: u8 = 0x01;
    // RESERVED
    // RESERVED
    // RESERVED
    // RESERVED
    // RESERVED
    pub const SETUP_SPEED_ACTIVE_ACK: u8 = 0x40;
    pub const RESET: u8 = 0x80;
}

#[derive(Debug, Clone, Copy)]
pub enum SpeedControlState {
    Init,
    WaitReady,
    SelectMode,
    StartPulse,
    Running,
    ErrorAck,
}
