use std::sync::Arc;

use smol::{block_on, lock::RwLock};

use crate::devices::wago_modules::wago_750_671::{InitState, Wago750_671};
use crate::io::stepper_reference_wago_750_671::StepperReferenceWago750671;
use crate::io::traverse_axis::{TraverseEndstop, TraverseStepperAxis};
use anyhow::Error;

#[derive(Debug)]
pub struct StepperVelocityWago750671 {
    pub device: Arc<RwLock<Wago750_671>>,
    pub state: InitState,

    /// Raw WAGO velocity register value written into rxpdo.velocity
    pub target_velocity_register: i16,
    /// Backwards-compatible raw velocity alias used by older minimal machines.
    pub target_velocity: i16,

    /// Logical target speed in FULL steps per second.
    /// This is the semantic contract exposed to higher layers.
    pub target_speed_fullsteps_per_second: i32,

    pub target_acceleration: u16,
    pub enabled: bool,

    /// WAGO frequency range selector (0..=3)
    pub freq_range_sel: u8,

    /// WAGO acceleration range selector (0..=3)
    pub acc_range_sel: u8,

    /// Logical position offset in MICROSTEPS
    pub position_offset: i128,

    /// Optional user scaling applied before writing speed
    pub speed_scale: f64,

    /// +1 or -1
    pub direction_multiplier: i8,

    /// Motor full steps per revolution, usually 200
    pub motor_full_steps_per_rev: u16,

    /// Microsteps per full step.
    /// Set this to 64 if you want EL7031-like microstep resolution.
    pub microsteps_per_full_step: u16,

    /// Whether changing target velocity while already running in speed mode
    /// should retrigger the WAGO start pulse.
    pub restart_on_velocity_change: bool,

    /// Config fallback used by the 671 when `freq_range_sel == 0`.
    pub freq_div_config: i32,
}

#[derive(Debug, Clone, Copy)]
pub enum Wago750671Mode {
    PrimaryApplication,
    Positioning,
    Program,
    Reference,
    Jog,
    Mailbox,
}

impl StepperVelocityWago750671 {
    const MBX_DRIVE_COMMAND: u8 = 0x40;
    const MBX_STOP_NO_RAMP: u8 = 0x19;
    const MBX_SET_CURRENT: u8 = 0x39;
    const MBX_SET_ACTUAL_POSITION: u8 = 0x2E;
    const MBX_SET_ACTUAL_POSITION_ZERO: u8 = 0x2F;

    pub fn new(device: Arc<RwLock<Wago750_671>>) -> Self {
        Self {
            device,
            state: InitState::Off,
            target_velocity_register: 0,
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

    /// Recommended when you want this axis to behave like EL7031 scaling.
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
        if self.enabled == enabled {
            return;
        }

        // Must be set before enabling to avoid controller errors
        self.set_acceleration(self.target_acceleration);

        self.enabled = enabled;
        if enabled {
            let mut dev = block_on(self.device.write());
            dev.desired_mode = C1Mode::SpeedControl;
            dev.desired_control_byte3 = 0;
            dev.start_requested = true;
            drop(dev);
            self.change_init_state(InitState::Enable);
        } else {
            self.change_init_state(InitState::Off);
            self.target_velocity_register = 0;
            self.target_velocity = 0;
            self.target_speed_fullsteps_per_second = 0;
            // Logical zeroing in speed mode is software-only on the 671.
            // Drop any stale offset when the axis is torn down so the next
            // enabled session starts from the raw controller position again.
            self.position_offset = 0;

            let mut dev = block_on(self.device.write());
            dev.initialized = false;
            dev.desired_mode = C1Mode::SpeedControl;
            dev.desired_control_byte3 = 0;
            dev.start_requested = false;
            dev.rxpdo.velocity = 0;
            dev.rxpdo.control_byte1 = 0;
            dev.rxpdo.control_byte2 &= !(C2Flag::ErrorQuit as u8);
            dev.rxpdo.control_byte3 &= !(C3Flag::ResetQuit as u8);
        }
    }

    /// Direct raw-register setter.
    /// Prefer `set_speed()` for application code.
    pub fn set_velocity_register(&mut self, velocity_register: i16) {
        let previous_velocity_register = self.target_velocity_register;
        self.target_velocity_register = velocity_register;
        self.target_velocity = velocity_register;

        let mut dev = block_on(self.device.write());
        let reset_active = StatusByteS3::from_bits(dev.txpdo.status_byte3).has_flag(S3Flag::Reset);

        let direction_bits = if velocity_register > 0 {
            C3Flag::DirectionPos as u8
        } else if velocity_register < 0 {
            C3Flag::DirectionNeg as u8
        } else {
            0
        };
        dev.desired_control_byte3 = direction_bits & !(C3Flag::ResetQuit as u8);

        dev.rxpdo.velocity = velocity_register.clamp(-25000, 25000);
        dev.rxpdo.control_byte3 = if reset_active {
            dev.desired_control_byte3 | C3Flag::ResetQuit as u8
        } else {
            dev.desired_control_byte3
        };

        let s1 = StatusByteS1::from_bits(dev.txpdo.status_byte1);
        let speed_mode_selected = dev.desired_mode as u8 == C1Mode::SpeedControl as u8;
        let start_ack = s1.has_flag(S1Flag::StartAck);
        let velocity_changed = velocity_register != previous_velocity_register;
        let needs_start_edge = self.restart_on_velocity_change
            || (previous_velocity_register == 0 && velocity_register != 0)
            || (!start_ack && velocity_register != 0);
        if self.enabled && speed_mode_selected && velocity_changed && needs_start_edge {
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

    /// Set logical speed in FULL STEPS PER SECOND.
    ///
    /// This keeps the public semantics aligned with EL70x1-style code.
    pub fn set_speed(&mut self, steps_per_second: f64) {
        // The 671 only accepts live speed changes cleanly while its speed
        // generator session is active. Make that contract part of the raw
        // speed wrapper so spool, traverse, and puller all share it.
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

    pub fn set_velocity(&mut self, velocity: i16) {
        self.set_velocity_register(velocity);
    }

    /// Get logical speed in FULL STEPS PER SECOND.
    pub fn get_speed(&self) -> i32 {
        self.target_speed_fullsteps_per_second
    }

    pub fn set_acceleration(&mut self, acceleration: u16) {
        self.target_acceleration = acceleration;

        let mut dev = block_on(self.device.write());
        dev.rxpdo.acceleration = acceleration;
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

    /// Effective WAGO frequency prescaler.
    pub fn get_effective_freq_prescaler(&self) -> i32 {
        match self.freq_range_sel {
            0 => self.freq_div_config.max(1),
            1 => 80,
            2 => 20,
            3 => 4,
            _ => self.freq_div_config.max(1),
        }
    }

    pub fn tick(&mut self) {}

    fn change_init_state(&mut self, state: InitState) {
        self.state = state.clone();
        let mut dev = block_on(self.device.write());
        dev.state = state;
    }

    pub(crate) fn request_mode_internal(
        &mut self,
        mode: C1Mode,
        control_byte3: u8,
        start_requested: bool,
    ) {
        let mut dev = block_on(self.device.write());
        let sanitized_control_byte3 = control_byte3 & !(C3Flag::ResetQuit as u8);
        if dev.desired_mode as u8 == mode as u8
            && dev.desired_control_byte3 == sanitized_control_byte3
            && dev.start_requested == start_requested
        {
            return;
        }
        dev.desired_mode = mode;
        dev.desired_control_byte3 = sanitized_control_byte3;
        dev.start_requested = start_requested;
        if self.enabled {
            dev.initialized = false;
            dev.state = InitState::SetMode;
            self.state = InitState::SetMode;
        }
    }

    pub fn request_speed_mode(&mut self) {
        let mut dev = block_on(self.device.write());
        let reset_active = StatusByteS3::from_bits(dev.txpdo.status_byte3).has_flag(S3Flag::Reset);
        let already_in_speed_mode = dev.desired_mode as u8 == C1Mode::SpeedControl as u8
            && dev.desired_control_byte3 == 0
            && !dev.start_requested;
        if already_in_speed_mode {
            return;
        }

        let should_restart =
            !dev.initialized || dev.desired_mode as u8 != C1Mode::SpeedControl as u8;
        dev.desired_mode = C1Mode::SpeedControl;
        dev.desired_control_byte3 = 0;
        if reset_active {
            dev.rxpdo.control_byte3 |= C3Flag::ResetQuit as u8;
        }
        dev.start_requested = should_restart;
        if self.enabled && should_restart {
            dev.initialized = false;
            dev.state = InitState::SetMode;
            self.state = InitState::SetMode;
        }
    }

    pub fn activate_jog(&mut self, timeout_ms: u16) {
        {
            let mut dev = block_on(self.device.write());
            dev.rxpdo.acceleration = timeout_ms;
        }
        let jog_ack = self.get_s1_bit6_jog_mode_ack();
        let start_ack = self.get_s1_bit2_start_ack();
        if !jog_ack {
            self.request_mode_internal(C1Mode::JogMode, 0, false);
        } else if !start_ack {
            self.request_mode_internal(C1Mode::JogMode, 0, true);
        } else {
            self.request_mode_internal(C1Mode::JogMode, 0, false);
        }
    }

    pub fn jog(&mut self, direction: C3Flag, timeout_ms: u16) {
        {
            let mut dev = block_on(self.device.write());
            dev.rxpdo.acceleration = timeout_ms;
        }
        let control_byte3 = match direction {
            C3Flag::DirectionPos | C3Flag::DirectionNeg => direction as u8,
            _ => 0,
        };
        let jog_ack = self.get_s1_bit6_jog_mode_ack();
        let start_ack = self.get_s1_bit2_start_ack();
        if !jog_ack {
            self.request_mode_internal(C1Mode::JogMode, 0, false);
        } else if !start_ack {
            self.request_mode_internal(C1Mode::JogMode, 0, true);
        } else {
            self.request_mode_internal(C1Mode::JogMode, control_byte3, false);
        }
    }

    pub fn stop_jog(&mut self) {
        self.request_mode_internal(C1Mode::JogMode, 0, false);
    }

    pub fn request_fast_stop(&mut self) {
        self.target_velocity_register = 0;
        self.target_velocity = 0;
        self.target_speed_fullsteps_per_second = 0;
        let mut dev = block_on(self.device.write());
        dev.desired_stop2_n = false;
        dev.start_requested = false;
        dev.rxpdo.velocity = 0;
    }

    pub fn clear_fast_stop(&mut self) {
        let mut dev = block_on(self.device.write());
        dev.desired_stop2_n = true;
    }

    pub fn request_stop_no_ramp_mailbox(&mut self) {
        self.target_velocity_register = 0;
        self.target_velocity = 0;
        self.target_speed_fullsteps_per_second = 0;

        let mut dev = block_on(self.device.write());
        dev.desired_stop2_n = true;
        dev.desired_control_byte3 = 0;
        dev.start_requested = false;
        dev.rxpdo.velocity = 0;
        dev.queue_mailbox_command(
            Self::MBX_DRIVE_COMMAND,
            Self::MBX_STOP_NO_RAMP,
            0,
            0,
            0,
            true,
        );

        if self.enabled {
            dev.desired_mode = C1Mode::Mailbox;
            dev.initialized = false;
            dev.state = InitState::SetMode;
            self.state = InitState::SetMode;
        }
    }

    pub fn request_set_actual_position_mailbox(&mut self, position_microsteps: i128) {
        let raw_position = position_microsteps.clamp(i32::MIN as i128, i32::MAX as i128) as i32;
        let [lsb, mid, msb, _] = raw_position.to_le_bytes();

        let mut dev = block_on(self.device.write());
        dev.desired_stop2_n = true;
        dev.desired_control_byte3 = 0;
        dev.start_requested = false;
        dev.rxpdo.velocity = 0;
        dev.queue_mailbox_command(
            Self::MBX_DRIVE_COMMAND,
            Self::MBX_SET_ACTUAL_POSITION,
            lsb,
            mid,
            msb,
            true,
        );

        if self.enabled {
            dev.desired_mode = C1Mode::Mailbox;
            dev.initialized = false;
            dev.state = InitState::SetMode;
            self.state = InitState::SetMode;
        }
    }

    pub fn request_set_actual_position_zero_mailbox(&mut self) {
        let mut dev = block_on(self.device.write());
        dev.desired_stop2_n = true;
        dev.desired_control_byte3 = 0;
        dev.start_requested = false;
        dev.rxpdo.velocity = 0;
        dev.queue_mailbox_command(
            Self::MBX_DRIVE_COMMAND,
            Self::MBX_SET_ACTUAL_POSITION_ZERO,
            0,
            0,
            0,
            true,
        );

        if self.enabled {
            dev.desired_mode = C1Mode::Mailbox;
            dev.initialized = false;
            dev.state = InitState::SetMode;
            self.state = InitState::SetMode;
        }
    }

    pub fn request_set_current_mailbox(&mut self, current_percent: u8, valid_range_bits: u8) {
        let mut dev = block_on(self.device.write());
        dev.desired_stop2_n = true;
        dev.desired_control_byte3 = 0;
        dev.start_requested = false;
        dev.rxpdo.velocity = 0;
        dev.queue_mailbox_command(
            Self::MBX_DRIVE_COMMAND,
            Self::MBX_SET_CURRENT,
            current_percent.min(150),
            0,
            valid_range_bits & 0x0F,
            true,
        );

        if self.enabled {
            dev.desired_mode = C1Mode::Mailbox;
            dev.initialized = false;
            dev.state = InitState::SetMode;
            self.state = InitState::SetMode;
        }
    }

    pub fn is_mailbox_active(&self) -> bool {
        let dev = block_on(self.device.read());
        dev.mailbox_active || dev.mailbox_in_flight || matches!(dev.desired_mode, C1Mode::Mailbox)
    }

    pub fn reference(&mut self) -> StepperReferenceWago750671<'_> {
        StepperReferenceWago750671::new(self)
    }

    /// Convert microsteps/s to the WAGO velocity register.
    ///
    /// WAGO speed-mode relation:
    /// output_frequency_hz = velocity_register * 80 / freq_prescaler
    ///
    /// We treat "output_frequency_hz" as microsteps/s.
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

    pub fn get_actual_velocity_register(&self) -> i16 {
        let dev = block_on(self.device.read());
        dev.txpdo.actual_velocity
    }

    pub fn get_actual_speed_steps_per_second(&self) -> f64 {
        self.velocity_register_to_steps_per_second(self.get_actual_velocity_register())
    }

    pub fn get_s3_bit0(&self) -> bool {
        let dev = block_on(self.device.read());
        StatusByteS3::from_bits(dev.txpdo.status_byte3).has_flag(S3Flag::Input1)
    }

    pub fn get_s3_bit1(&self) -> bool {
        let dev = block_on(self.device.read());
        StatusByteS3::from_bits(dev.txpdo.status_byte3).has_flag(S3Flag::Input2)
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

    pub fn get_target_acceleration(&self) -> u16 {
        let dev = block_on(self.device.read());
        dev.rxpdo.acceleration
    }

    /// Raw controller position in MICROSTEPS
    pub fn get_raw_position(&self) -> i128 {
        let dev = block_on(self.device.read());
        decode_i24(
            dev.txpdo.position_l,
            dev.txpdo.position_m,
            dev.txpdo.position_h,
        ) as i128
    }

    /// Logical application position in MICROSTEPS
    pub fn get_position(&self) -> i128 {
        self.get_raw_position() + self.position_offset
    }

    /// Logical zeroing in MICROSTEPS.
    /// This preserves your current traverse-controller behavior.
    pub fn set_position(&mut self, position: i128) {
        self.position_offset = position - self.get_raw_position();
    }

    pub fn get_mode(&self) -> Option<Wago750671Mode> {
        let s1 = StatusByteS1::from_bits(self.get_status_byte1());
        let bits = s1.bits();

        if (bits & 0b0000_1000) != 0 {
            Some(Wago750671Mode::PrimaryApplication)
        } else if (bits & 0b0001_0000) != 0 {
            Some(Wago750671Mode::Program)
        } else if (bits & 0b0010_0000) != 0 {
            Some(Wago750671Mode::Reference)
        } else if (bits & 0b0100_0000) != 0 {
            Some(Wago750671Mode::Jog)
        } else if (bits & 0b1000_0000) != 0 {
            Some(Wago750671Mode::Mailbox)
        } else {
            None
        }
    }

    pub fn get_s1_bit3_speed_mode_ack(&self) -> bool {
        let s1 = StatusByteS1::from_bits(self.get_status_byte1());
        (s1.bits() & 0b0000_1000) != 0
    }

    pub fn get_s1_bit2_start_ack(&self) -> bool {
        let s1 = StatusByteS1::from_bits(self.get_status_byte1());
        s1.has_flag(S1Flag::StartAck)
    }

    pub fn get_s1_bit5_reference_mode_ack(&self) -> bool {
        let s1 = StatusByteS1::from_bits(self.get_status_byte1());
        (s1.bits() & (C1Mode::Reference as u8)) != 0
    }

    pub fn get_s1_bit6_jog_mode_ack(&self) -> bool {
        let s1 = StatusByteS1::from_bits(self.get_status_byte1());
        (s1.bits() & (C1Mode::JogMode as u8)) != 0
    }

    pub fn get_s2_reference_ok(&self) -> bool {
        let s2 = StatusByteS2::from_bits(self.get_status_byte2());
        s2.has_flag(S2Flag::ReferenceOk)
    }

    pub fn get_s2_busy(&self) -> bool {
        let s2 = StatusByteS2::from_bits(self.get_status_byte2());
        s2.has_flag(S2Flag::Busy)
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

impl TraverseStepperAxis for StepperVelocityWago750671 {
    fn set_speed(&mut self, steps_per_second: f64) -> Result<(), Error> {
        Self::set_speed(self, steps_per_second);
        Ok(())
    }

    fn set_enabled(&mut self, enabled: bool) {
        Self::set_enabled(self, enabled);
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }

    fn get_position(&self) -> i128 {
        Self::get_position(self)
    }

    fn set_position(&mut self, position: i128) {
        Self::set_position(self, position);
    }
}

impl TraverseEndstop for bool {
    fn is_active(&self) -> bool {
        *self
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
pub enum C1Mode {
    SpeedControl = 0b0000_1000,
    Program = 0b0001_0000,
    Reference = 0b0010_0000,
    JogMode = 0b0100_0000,
    Mailbox = 0b1000_0000,
}

impl ControlByteC1 {
    pub const fn new() -> Self {
        Self(0)
    }

    pub const fn with_flag(mut self, flag: C1Flag) -> Self {
        self.0 |= flag as u8;
        self
    }

    pub const fn with_mode(mut self, mode: C1Mode) -> Self {
        self.0 = (self.0 & 0b0000_0111) | (mode as u8);
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
pub enum StatusByte {
    S0,
    S1,
    S2,
    S3,
}
