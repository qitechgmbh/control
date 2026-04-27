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
    /// Raw WAGO velocity register value written into rxpdo.velocity.
    pub target_velocity: i16,
    /// Logical target speed in FULL steps per second.
    pub target_speed_fullsteps_per_second: i32,
    pub target_acceleration: u16,
    pub enabled: bool,
    pub freq_range_sel: u8,
    pub acc_range_sel: u8,
    /// Logical position offset in MICROSTEPS.
    pub position_offset: i128,
    pub speed_scale: f64,
    pub direction_multiplier: i8,
    pub motor_full_steps_per_rev: u16,
    pub microsteps_per_full_step: u16,
    pub restart_on_velocity_change: bool,
    pub freq_div_config: i32,
}

impl StepperVelocityWago750672 {
    const CFG_FREQ_DIV: u16 = 4;
    const CFG_ACC_FACT: u16 = 6;

    pub fn new(device: Arc<RwLock<Wago750_672>>) -> Self {
        {
            let mut dev = block_on(device.write());
            dev.desired_command = C1Command::SpeedControl;
            dev.start_requested = true;
        }

        Self {
            device,
            state: InitState::Off,
            target_velocity: 0,
            target_speed_fullsteps_per_second: 0,
            target_acceleration: 10000,
            enabled: false,
            freq_range_sel: 0,
            acc_range_sel: 0,
            position_offset: 0,
            speed_scale: 1.0,
            direction_multiplier: 1,
            motor_full_steps_per_rev: 200,
            microsteps_per_full_step: 64,
            restart_on_velocity_change: true,
            freq_div_config: 200,
        }
    }

    pub fn set_microsteps_per_full_step(&mut self, microsteps: u16) {
        if self.enabled || microsteps == 0 {
            return;
        }
        self.microsteps_per_full_step = microsteps;
    }

    pub fn set_motor_full_steps_per_rev(&mut self, steps: u16) {
        if self.enabled || steps == 0 {
            return;
        }
        self.motor_full_steps_per_rev = steps;
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
            let mut dev = block_on(self.device.write());
            dev.rxpdo.velocity = self.target_velocity.clamp(-25000, 25000);
            dev.rxpdo.acceleration = self.target_acceleration;
            dev.desired_stop2_n = true;
            dev.start_requested = self.target_velocity != 0;
            drop(dev);
            self.change_init_state(InitState::Enable);
        } else {
            self.change_init_state(InitState::Off);
            self.target_velocity = 0;
            self.target_speed_fullsteps_per_second = 0;
            self.position_offset = 0;
            self.write_control_byte(ControlByte::C1, 0b00000000);
            let mut dev = block_on(self.device.write());
            dev.initialized = false;
            dev.desired_command = C1Command::SpeedControl;
            dev.desired_stop2_n = true;
            dev.start_requested = false;
            dev.rxpdo.velocity = 0;
            dev.rxpdo.control_byte2 &= !(C2Flag::ErrorQuit as u8);
            dev.rxpdo.control_byte3 &= !(C3Flag::ResetQuit as u8);
        }
    }

    pub fn set_velocity(&mut self, velocity: i16) {
        self.set_velocity_register(velocity);
    }

    pub fn set_velocity_register(&mut self, velocity: i16) {
        let previous_velocity = self.target_velocity;
        self.target_velocity = velocity;

        let mut dev = block_on(self.device.write());

        // clamp velocity to -25000 - 25000 :: because this is
        // the min max for the controller
        dev.rxpdo.velocity = velocity.clamp(-25000, 25000);

        let velocity_changed = velocity != previous_velocity;
        if self.enabled && velocity_changed && velocity != 0 {
            let s1 = StatusByteS1::from_bits(dev.txpdo.status_byte1);
            let start_ack = s1.has_flag(S1Flag::StartAck);
            let needs_start_edge =
                self.restart_on_velocity_change || previous_velocity == 0 || !start_ack;

            if needs_start_edge {
                dev.start_requested = true;
                if dev.initialized {
                    dev.state = if start_ack {
                        InitState::StartPulseEnd
                    } else {
                        InitState::StartPulseStart
                    };
                    self.state = dev.state.clone();
                }
            }
        }
    }

    pub fn set_speed(&mut self, steps_per_second: f64) {
        self.request_speed_mode();
        self.target_speed_fullsteps_per_second = steps_per_second.round() as i32;

        let directed_fullsteps_per_second =
            steps_per_second * self.speed_scale * f64::from(self.direction_multiplier);
        let directed_microsteps_per_second =
            directed_fullsteps_per_second * f64::from(self.microsteps_per_full_step);
        let velocity_register =
            self.microsteps_per_second_to_velocity_register(directed_microsteps_per_second);

        self.set_velocity_register(velocity_register);
    }

    pub fn get_speed(&self) -> i32 {
        self.target_speed_fullsteps_per_second
    }

    pub fn set_acceleration(&mut self, acceleration: u16) {
        self.target_acceleration = acceleration;

        let mut dev = block_on(self.device.write());

        dev.rxpdo.acceleration = acceleration;
    }

    pub fn get_actual_velocity_register(&self) -> i16 {
        let dev = block_on(self.device.read());
        dev.txpdo.actual_velocity
    }

    pub fn get_target_acceleration(&self) -> u16 {
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

    pub fn set_speed_scale(&mut self, speed_scale: f64) {
        self.speed_scale = speed_scale;
    }

    pub fn set_direction_multiplier(&mut self, direction_multiplier: i8) {
        self.direction_multiplier = if direction_multiplier < 0 { -1 } else { 1 };
    }

    pub fn set_restart_on_velocity_change(&mut self, restart_on_velocity_change: bool) {
        self.restart_on_velocity_change = restart_on_velocity_change;
    }

    pub fn set_freq_div_config(&mut self, freq_div_config: i32) {
        if self.enabled || freq_div_config <= 0 {
            return;
        }
        self.freq_div_config = freq_div_config;
    }

    pub fn get_effective_freq_prescaler(&self) -> i32 {
        match self.freq_range_sel {
            0 => self.freq_div_config.max(1),
            1 => 80,
            2 => 20,
            3 => 4,
            _ => self.freq_div_config.max(1),
        }
    }

    pub fn request_speed_mode(&mut self) {
        let mut dev = block_on(self.device.write());
        let already_in_speed_mode =
            dev.desired_command as u8 == C1Command::SpeedControl as u8 && !dev.start_requested;
        if already_in_speed_mode {
            return;
        }

        let should_restart =
            !dev.initialized || dev.desired_command as u8 != C1Command::SpeedControl as u8;
        dev.desired_command = C1Command::SpeedControl;
        dev.start_requested = should_restart;
        if self.enabled && should_restart {
            dev.initialized = false;
            dev.state = InitState::SetMode;
            self.state = InitState::SetMode;
        }
    }

    /// Sets the Freq_Div configuration parameter via the mailbox-backed
    /// configuration table. This is the prescaler used when
    /// `freq_range_sel == 0`.
    pub fn request_set_freq_div_config_mailbox(&mut self, freq_div_config: u16) {
        let freq_div_config = freq_div_config.max(4);
        self.freq_div_config = i32::from(freq_div_config);

        let mut dev = block_on(self.device.write());
        dev.start_requested = false;
        dev.queue_config_write_u16(Self::CFG_FREQ_DIV, freq_div_config);
    }

    /// Sets the Acc_Fact configuration parameter via the mailbox-backed
    /// configuration table. This factor is used when `acc_range_sel == 0`.
    pub fn request_set_acc_fact_mailbox(&mut self, acc_fact: u16) {
        let acc_fact = acc_fact.max(1);

        let mut dev = block_on(self.device.write());
        dev.start_requested = false;
        dev.queue_config_write_u16(Self::CFG_ACC_FACT, acc_fact);
    }

    pub fn request_fast_stop(&mut self) {
        self.target_velocity = 0;
        self.target_speed_fullsteps_per_second = 0;
        let mut dev = block_on(self.device.write());
        dev.desired_stop2_n = false;
        dev.start_requested = false;
        dev.rxpdo.velocity = 0;
        dev.state = InitState::Running;
    }

    pub fn clear_fast_stop(&mut self) {
        let mut dev = block_on(self.device.write());
        dev.desired_stop2_n = true;
    }

    fn microsteps_per_second_to_velocity_register(&self, microsteps_per_second: f64) -> i16 {
        let prescaler = f64::from(self.get_effective_freq_prescaler());
        let register = (microsteps_per_second * prescaler / 80.0).round();
        register.clamp(-25000.0, 25000.0) as i16
    }

    fn velocity_register_to_microsteps_per_second(&self, velocity_register: i16) -> f64 {
        let prescaler = f64::from(self.get_effective_freq_prescaler());
        f64::from(velocity_register) * 80.0 / prescaler
    }

    pub fn velocity_register_to_steps_per_second(&self, velocity_register: i16) -> f64 {
        self.velocity_register_to_microsteps_per_second(velocity_register)
            / f64::from(self.microsteps_per_full_step)
    }

    pub fn get_actual_speed_steps_per_second(&self) -> f64 {
        self.velocity_register_to_steps_per_second(self.get_actual_velocity_register())
    }

    pub fn get_s3_bit0(&self) -> bool {
        let dev = block_on(self.device.read());
        StatusByteS3::from_bits(dev.txpdo.status_byte3).has_flag(S3Flag::Input1)
    }

    pub fn get_status_byte1(&self) -> u8 {
        self.read_status_byte(StatusByte::S1)
    }

    pub fn get_status_byte2(&self) -> u8 {
        self.read_status_byte(StatusByte::S2)
    }

    pub fn get_status_byte3(&self) -> u8 {
        self.read_status_byte(StatusByte::S3)
    }

    pub fn get_control_byte1(&self) -> u8 {
        let dev = block_on(self.device.read());
        dev.rxpdo.control_byte1
    }

    pub fn get_control_byte2(&self) -> u8 {
        let dev = block_on(self.device.read());
        dev.rxpdo.control_byte2
    }

    pub fn get_control_byte3(&self) -> u8 {
        let dev = block_on(self.device.read());
        dev.rxpdo.control_byte3
    }

    pub fn get_raw_position(&self) -> i128 {
        let dev = block_on(self.device.read());
        decode_i24(
            dev.txpdo.position_l,
            dev.txpdo.position_m,
            dev.txpdo.position_h,
        ) as i128
    }

    pub fn get_position(&self) -> i128 {
        self.get_raw_position() + self.position_offset
    }

    pub fn set_position(&mut self, position: i128) {
        self.position_offset = position - self.get_raw_position();
    }

    pub fn get_s1_bit3_speed_mode_ack(&self) -> bool {
        let s1 = StatusByteS1::from_bits(self.get_status_byte1());
        (s1.bits() & (C1Command::SpeedControl as u8)) == C1Command::SpeedControl as u8
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

    #[allow(dead_code)]
    fn read_status_byte(&self, status_byte: StatusByte) -> u8 {
        let dev = block_on(self.device.read());

        match status_byte {
            StatusByte::S0 => dev.txpdo.status_byte0,
            StatusByte::S1 => dev.txpdo.status_byte1,
            StatusByte::S2 => dev.txpdo.status_byte2,
            StatusByte::S3 => dev.txpdo.status_byte3,
        }
    }
}

fn decode_i24(l: u8, m: u8, h: u8) -> i32 {
    let raw = (l as u32) | ((m as u32) << 8) | ((h as u32) << 16);
    if (raw & 0x0080_0000) != 0 {
        (raw | 0xFF00_0000) as i32
    } else {
        raw as i32
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
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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
