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
    pub state: InitState,
    pub target_velocity: i16,
    pub target_acceleration: u16,
    pub enabled: bool,
    pub freq_range_sel: u8,
    pub acc_range_sel: u8,
}

impl StepperVelocityWago750672 {
    pub fn new(device: Arc<RwLock<Wago750_672>>) -> Self {
        Self {
            device,
            state: InitState::Off,
            target_velocity: 0,
            target_acceleration: 10000,
            enabled: false,
            freq_range_sel: 0,
            acc_range_sel: 0,
        }
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        if self.enabled && enabled {
            return;
        }

        // this needs to be set before the stepper controller
        // is enabled because it will generate an error otherwise
        self.set_acceleration(self.target_acceleration);

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

        // clamp velocity to -25000 - 25000 :: because this is
        // the min max for the controller
        dev.rxpdo.velocity = velocity.clamp(-25000, 25000);

        if dev.initialized {
            dev.state = InitState::StartPulseStart;
        }
    }

    pub fn set_acceleration(&mut self, acceleration: u16) {
        self.target_acceleration = acceleration;

        let mut dev = block_on(self.device.write());

        dev.rxpdo.acceleration = acceleration;
    }

    fn get_actual_velocity(&self) -> i16 {
        let dev = block_on(self.device.read());
        dev.txpdo.actual_velocity
    }

    fn get_target_acceleration(&self) -> u16 {
        let dev = block_on(self.device.read());
        dev.rxpdo.acceleration
    }

    pub fn set_freq_range_sel(&mut self, factor: u8) {
        if self.enabled || factor > 3 {
            return;
        }
        self.freq_range_sel = factor;
        let mut dev = block_on(self.device.write());
        let c2 = ControlByteC2::from_bits(dev.rxpdo.control_byte2)
            .with_freq_range(factor)
            .bits();
        dev.rxpdo.control_byte2 = c2;
    }

    pub fn set_acc_range_sel(&mut self, factor: u8) {
        if self.enabled || factor > 3 {
            return;
        }
        self.acc_range_sel = factor;
        let mut dev = block_on(self.device.write());
        let c2 = ControlByteC2::from_bits(dev.rxpdo.control_byte2)
            .with_acc_range(factor)
            .bits();
        dev.rxpdo.control_byte2 = c2;
    }

    fn change_init_state(&mut self, state: InitState) {
        self.state = state.clone();
        let mut dev = block_on(self.device.write());
        dev.state = state;
    }

    fn write_control_byte(&self, control_byte: ControlByte, value: u8) {
        let mut dev = block_on(self.device.write());

        match control_byte {
            ControlByte::C0 => dev.rxpdo.control_byte = value,
            ControlByte::C1 => dev.rxpdo.control_byte1 = value,
            ControlByte::C2 => dev.rxpdo.control_byte2 = value,
            ControlByte::C3 => dev.rxpdo.control_byte3 = value,
        }
    }

    fn read_status_byte(&self, status_byte: StatusByte) -> u8 {
        let dev = block_on(self.device.write());

        match status_byte {
            StatusByte::S0 => dev.txpdo.status_byte0,
            StatusByte::S1 => dev.txpdo.status_byte1,
            StatusByte::S2 => dev.txpdo.status_byte2,
            StatusByte::S3 => dev.txpdo.status_byte3,
        }
    }
}

// The Different Control Bytes set control the stepper controller
// by setting and resetting specific bits.
//
// There are some construction functions provided to create the
// control bytes correctly.

/// Control Byte C1
#[derive(Clone, Copy, Default)]
pub struct ControlByteC1(u8);

#[repr(u8)]
#[derive(Clone, Copy)]
pub enum C1Flag {
    Enable = 0b0000_0001,
    Stop2N = 0b0000_0010,
    Start = 0b0000_0100,
}

#[repr(u8)]
#[derive(Clone, Copy)]
pub enum C1Command {
    Idle = 0b0000_0000,
    SinglePosition = 0b0000_1000,
    RunProgram = 0b0001_0000,
    SpeedControl = 0b0001_1000,
    Reference = 0b0010_0000,
    JogMode = 0b0010_1000,
    Mailbox = 0b0011_0000,
}

impl ControlByteC1 {
    pub const fn new() -> Self {
        Self(0)
    }

    pub const fn with_flag(mut self, flag: C1Flag) -> Self {
        self.0 |= flag as u8;
        self
    }

    pub const fn with_command(mut self, cmd: C1Command) -> Self {
        self.0 = (self.0 & 0b0000_0111) | (cmd as u8);
        self
    }

    pub const fn bits(self) -> u8 {
        self.0
    }
}

/// Control Byte C2
#[derive(Clone, Copy, Default)]
pub struct ControlByteC2(u8);

#[repr(u8)]
#[derive(Clone, Copy)]
pub enum C2Flag {
    FreqRangeSelL = 0b0000_0001,
    FreqRangeSelH = 0b0000_0010,
    AccRangeSelL = 0b0000_0100,
    AccRangeSelH = 0b0000_1000,
    PreCalc = 0b0100_0000,
    ErrorQuit = 0b1000_0000,
}

impl ControlByteC2 {
    pub const fn new() -> Self {
        Self(0)
    }

    pub const fn from_bits(bits: u8) -> Self {
        Self(bits)
    }

    pub const fn with_freq_range(mut self, sel: u8) -> Self {
        self.0 = (self.0 & 0b1111_1100) | (sel & 0b0000_0011);
        self
    }

    pub const fn with_acc_range(mut self, sel: u8) -> Self {
        self.0 = (self.0 & 0b1111_0011) | ((sel & 0b0000_0011) << 2);
        self
    }

    pub const fn with_flag(mut self, flag: C2Flag) -> Self {
        self.0 |= flag as u8;
        self
    }

    pub const fn bits(self) -> u8 {
        self.0
    }
}

/// Control Byte C3
#[derive(Clone, Copy, Default)]
pub struct ControlByteC3(u8);

#[repr(u8)]
#[derive(Clone, Copy)]
pub enum C3Flag {
    SetActualPos = 0b0000_0001,
    DirectionPos = 0b0000_0010,
    DirectionNeg = 0b0000_0100,
    ResetQuit = 0b1000_0000,
}

impl ControlByteC3 {
    pub const fn new() -> Self {
        Self(0)
    }

    pub const fn with_flag(mut self, flag: C3Flag) -> Self {
        self.0 |= flag as u8;
        self
    }

    pub const fn bits(self) -> u8 {
        self.0
    }
}

// The status bytes are similar to the control bytes but are only readable
// and most of the time provide corresponding acknoledgements to the diffrent
// Control bytes.
//
// There are also helper functions to construct the diffrent Status bytes for
// compoarison.

/// Status Byte S1
#[derive(Clone, Copy, Default)]
pub struct StatusByteS1(u8);

#[repr(u8)]
#[derive(Clone, Copy)]
pub enum S1Flag {
    Ready = 0b0000_0001,
    Stop2NAck = 0b0000_0010,
    StartAck = 0b0000_0100,
}

#[repr(u8)]
#[derive(Clone, Copy)]
pub enum S1CommandAck {
    Idle = 0b0000_0000,
    SinglePosition = 0b0000_1000,
    RunProgram = 0b0001_0000,
    SpeedControl = 0b0001_1000,
    Reference = 0b0010_0000,
    JogMode = 0b0010_1000,
    Mailbox = 0b0011_0000,
}

impl StatusByteS1 {
    pub const fn from_bits(bits: u8) -> Self {
        Self(bits)
    }

    pub const fn has_flag(self, flag: S1Flag) -> bool {
        (self.0 & flag as u8) != 0
    }

    pub const fn bits(self) -> u8 {
        self.0
    }
}

/// Status Byte S2
#[derive(Clone, Copy, Default)]
pub struct StatusByteS2(u8);

#[repr(u8)]
#[derive(Clone, Copy)]
pub enum S2Flag {
    OnTarget = 0b0000_0001,
    Busy = 0b0000_0010,
    StandStill = 0b0000_0100,
    OnSpeed = 0b0000_1000,
    Direction = 0b0001_0000,
    ReferenceOk = 0b0010_0000,
    PreCalcAck = 0b0100_0000,
    Error = 0b1000_0000,
}

impl StatusByteS2 {
    pub const fn from_bits(bits: u8) -> Self {
        Self(bits)
    }

    pub const fn has_flag(self, flag: S2Flag) -> bool {
        (self.0 & flag as u8) != 0
    }
}

/// Status Byte S3
#[derive(Clone, Copy, Default)]
pub struct StatusByteS3(u8);

#[repr(u8)]
#[derive(Clone, Copy)]
pub enum S3Flag {
    Input1 = 0b0000_0001,
    Input2 = 0b0000_0010,
    Input3 = 0b0000_0100,
    Input4 = 0b0000_1000,
    Input5 = 0b0001_0000,
    Input6 = 0b0010_0000,
    Warning = 0b0100_0000,
    Reset = 0b1000_0000,
}

impl StatusByteS3 {
    pub const fn from_bits(bits: u8) -> Self {
        Self(bits)
    }

    pub const fn has_flag(self, flag: S3Flag) -> bool {
        (self.0 & flag as u8) != 0
    }
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
