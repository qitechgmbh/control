use std::sync::Arc;

use smol::{block_on, lock::RwLock};

use crate::devices::wago_modules::wago_750_672::{InitState, Wago750_672};

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

        if enabled {
            self.change_init_state(InitState::Enable);
        } else {
            self.change_init_state(InitState::Off);
            self.write_control_byte(ControlByte::C1, 0b00000000);
        }
    }

    pub fn set_velocity(&mut self, velocity: i16) {
        self.target_velocity = velocity;

        let mut dev = block_on(self.device.write());

        dev.rxpdo.velocity = velocity;
        dev.rxpdo.acceleration = 10000; // hardcoded for now

        if dev.initialized {
            // This does not work because i can't block twice without a deadlock
            // self.change_init_state(InitState::StartPulse);
            dev.state = InitState::StartPulse;
        }
    }

    fn change_init_state(&self, state: InitState) {
        let mut dev = block_on(self.device.write());

        dev.state = state;
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
            StatusByte::S0 => dev.txpdo.s0,
            StatusByte::S1 => dev.txpdo.s1,
            StatusByte::S2 => dev.txpdo.s2,
            StatusByte::S3 => dev.txpdo.s3,
        }
    }
}

/// Control Byte C1
/// Bits 0 - 7
pub struct ControlByteC1;
impl ControlByteC1 {
    pub const ENABLE: u8 = 0b0000_0001;
    pub const STOP2_N: u8 = 0b0000_0010;
    pub const START: u8 = 0b0000_0100;
    pub const CMD_IDLE: u8 = 0b0000_0000;
    pub const CMD_SINGLE_POSITION: u8 = 0b0000_0000;
    pub const CMD_RUN_PROGRAM: u8 = 0b0000_0000;
    pub const CMD_SPEED_CONTROL: u8 = 0b0000_0000;
    pub const CMD_REFERENCE: u8 = 0b0000_0000;
    pub const CMD_JOG_MODE: u8 = 0b0000_0000;
    pub const CMD_MAILBOX: u8 = 0b0000_0000;
}

/// Control Byte C2
/// Bits 0 - 7
pub struct ControlByteC2;
impl ControlByteC2 {
    pub const FREQ_RANGE_SEL_L: u8 = 0b0000_0001;
    pub const FREQ_RANGE_SEL_H: u8 = 0b0000_0010;
    pub const ACCELERATION_RANGE_SEL_L: u8 = 0b0000_0100;
    pub const ACCELERATION_RANGE_SEL_H: u8 = 0b0000_1000;
    // RESERVED
    // RESERVED
    pub const PRE_CALC: u8 = 0b0100_0000;
    pub const ERROR_QUIT: u8 = 0b1000_0000;
}

/// Control Byte C3
/// Bits 0 - 7
pub struct ControlByteC3;
impl ControlByteC3 {
    pub const SET_ACTUAL_POS: u8 = 0b0000_0001;
    // RESERVED
    pub const DIRECTION_POS: u8 = 0b0000_0010;
    pub const DIRCTION_NEG: u8 = 0b0000_0100;
    // RESERVED
    // RESERVED
    // RESERVED
    pub const RESET_QUIT: u8 = 0b1000_0000;
}

/// Status Byte S1
/// Bits 0 - 7
pub struct StatusByteS1;
impl StatusByteS1 {
    pub const READY: u8 = 0b0000_0001;
    pub const STOP2_N_ACK: u8 = 0b0000_0010;
    pub const START_ACK: u8 = 0b0000_0100;
    pub const CMD_IDLE_ACK: u8 = 0b0000_0000;
    pub const CMD_SINGLE_POSITION_ACK: u8 = 0b0000_0000;
    pub const CMD_RUN_PROGRAM_ACK: u8 = 0b0000_0000;
    pub const CMD_SPEED_CONTROL_ACK: u8 = 0b0000_0000;
    pub const CMD_REFERENCE_ACK: u8 = 0b0000_0000;
    pub const CMD_JOG_MODE_ACK: u8 = 0b0000_0000;
    pub const CMD_MAILBOX_ACK: u8 = 0b0000_0000;
}

/// Status Byte S2
/// Bits 0 - 7
pub struct StatusByteS2;
impl StatusByteS2 {
    pub const ON_TARGET: u8 = 0b0000_0001;
    pub const BUSY: u8 = 0b0000_0010;
    pub const STAND_STILL: u8 = 0b0000_0100;
    pub const ON_SPEED: u8 = 0b0000_1000;
    pub const DIRECTION: u8 = 0b0001_0000;
    pub const REFERENCE_OK: u8 = 0b0010_0000;
    pub const PRE_CALC_ACK: u8 = 0b0100_0000;
    pub const ERROR: u8 = 0b1000_0000;
}

/// Status Byte S3
/// Bits 0 - 7
pub struct StatusByteS3;
impl StatusByteS3 {
    pub const INPUT1: u8 = 0b0000_0001;
    pub const INPUT2: u8 = 0b0000_0010;
    pub const INPUT3: u8 = 0b0000_0100;
    pub const INPUT4: u8 = 0b0000_1000;
    pub const INPUT5: u8 = 0b0001_0000;
    pub const INPUT6: u8 = 0b0010_0000;
    pub const WARNING: u8 = 0b0100_0000;
    pub const RESET: u8 = 0b1000_0000;
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
