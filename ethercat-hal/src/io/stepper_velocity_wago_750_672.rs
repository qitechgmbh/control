use std::sync::Arc;

use smol::{block_on, lock::RwLock};

use crate::devices::wago_modules::wago_750_672::Wago750_672;

/*
 * Wago Stepper Velocity Wrapper around Stepper Controller
 *
 */
#[derive(Debug)]
pub struct StepperVelocityWago750672 {
    pub device: Arc<RwLock<Wago750_672>>,
    pub state: SpeedControlState,
    pub target_velocity: i16,
    pub target_acceleration: u16,
    pub enabled: bool,
}

impl StepperVelocityWago750672 {
    pub fn new(device: Arc<RwLock<Wago750_672>>) -> Self {
        Self {
            device,
            state: SpeedControlState::Init,
            target_velocity: 1000,
            target_acceleration: 10000,
            enabled: false,
        }
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;

        let mut dev = block_on(self.device.write());

        if enabled {
            dev.rxpdo.c1 = 0b00000011;
            tracing::info!("Enabling!");
            if dev.txpdo.s1 & 0b00000011 != 0 {
                tracing::info!("Enabled!");
                tracing::info!("Setting Mode");
                dev.rxpdo.c1 = 0b00011011;
            }
            if dev.txpdo.s1 & 0b00011011 != 0 {
                tracing::info!("Mode Set");
                tracing::info!("Starting up...");
                dev.rxpdo.c1 = 0b00011111;
            }
        } else {
            tracing::info!("Disabled."); dev.rxpdo.c1 = 0b00000000;
        }
    }

    pub fn set_velocity(&mut self, velocity: i16) {
        self.target_velocity = velocity;

        let mut dev = block_on(self.device.write());

        dev.rxpdo.velocity = velocity;
        dev.rxpdo.acceleration = 10000; // hardcoded for now
    }

    pub fn clear_errors(&self, clear: bool) {
        let mut dev = block_on(self.device.write());
        let _ = clear;

        dev.rxpdo.c2 = 0b10000000;
        tracing::info!("S1{:08b}, S2{:08b}, S3{:08b}", dev.txpdo.s1, dev.txpdo.s2, dev.txpdo.s3);
    }

    pub fn clear_reset(&self, clear: bool) {
        let mut dev = block_on(self.device.write());
        let _ = clear;

        dev.rxpdo.c3 = 0b10000000;
    }

    pub fn stop_clear_errors(&self, clear: bool) {
        let mut dev = block_on(self.device.write());
        let _ = clear;

        dev.rxpdo.c2 = 0b00000000;
        tracing::info!("C1{:08b}, C2{:08b}, C3{:08b}", dev.rxpdo.c1, dev.rxpdo.c2, dev.rxpdo.c3);
    }

    pub fn stop_clear_reset(&self, clear: bool) {
        let mut dev = block_on(self.device.write());
        let _ = clear;

        dev.rxpdo.c3 = 0b00000000;
    }


    fn write_control_byte(&self, control_byte: ControlByte, value: u8) {
        let mut dev = block_on(self.device.write());

        match control_byte {
            ControlByte::C0 => dev.rxpdo.c0 = value,
            ControlByte::C1 => dev.rxpdo.c1 = value,
            ControlByte::C2 => dev.rxpdo.c2 = value,
            ControlByte::C3 => dev.rxpdo.c3 = value,
        }
    }

    fn read_status_byte(&self, status_byte: StatusByte) -> u8 {

        let dev = block_on(self.device.write());

        match status_byte {
            StatusByte::S0 => return dev.txpdo.s0,
            StatusByte::S1 => return dev.txpdo.s1,
            StatusByte::S2 => return dev.txpdo.s2,
            StatusByte::S3 => return dev.txpdo.s3,
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

    pub const COMMAND_MASK: u8 = 0xF8;

    pub const CMD_IDLE: u8 = 0x00;
    pub const CMD_SINGLE_POSITION: u8 = 0x08;
    pub const CMD_RUN_PROGRAM: u8 = 0x10;
    pub const CMD_SPEED_CONTROL: u8 = 0x18;
    pub const CMD_REFERENCE: u8 = 0x20;
    pub const CMD_JOG_MODE: u8 = 0x40;
    pub const CMD_MAILBOX: u8 = 0x80;
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

#[derive(Debug, Clone, Copy)]
pub enum ControlByte {
    C0,
    C1,
    C2,
    C3,
}

#[derive(Debug, Clone, Copy)]
pub enum StatusByte {
    S0,
    S1,
    S2,
    S3,
}
