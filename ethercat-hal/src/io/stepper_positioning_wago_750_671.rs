use std::sync::Arc;

use smol::{block_on, lock::RwLock};

use crate::devices::wago_modules::wago_750_671::{InitState, Wago750_671};
use crate::io::stepper_jog_wago_750_671::StepperJogWago750671;
use crate::io::stepper_velocity_wago_750_671::{
    C1Mode, C2Flag, C3Flag, S1Flag, S2Flag, S3Flag, StatusByte, StatusByteS1, StatusByteS2,
    StatusByteS3, Wago750671Mode,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Wago750671ReferenceDirection {
    Positive,
    Negative,
}

#[derive(Debug, Clone, Copy)]
pub struct Wago750671PositioningStatus {
    pub ready: bool,
    pub start_ack: bool,
    pub positioning_mode_ack: bool,
    pub reference_mode_ack: bool,
    pub jog_mode_ack: bool,
    pub busy: bool,
    pub on_target: bool,
    pub reference_ok: bool,
    pub standstill: bool,
    pub direction_positive: bool,
    pub input1: bool,
    pub input2: bool,
}

#[derive(Debug)]
pub struct StepperPositioningWago750671 {
    pub device: Arc<RwLock<Wago750_671>>,
    pub state: InitState,
    pub enabled: bool,
    pub target_velocity_register: i16,
    pub target_acceleration: u16,
    pub target_position_steps: i128,
    pub freq_range_sel: u8,
    pub acc_range_sel: u8,
    pub position_offset: i128,
    pub speed_scale: f64,
    pub direction_multiplier: i8,
    pub microsteps_per_full_step: u16,
}

impl StepperPositioningWago750671 {
    pub fn new(device: Arc<RwLock<Wago750_671>>) -> Self {
        Self {
            device,
            state: InitState::Off,
            enabled: false,
            target_velocity_register: 0,
            target_acceleration: 10000,
            target_position_steps: 0,
            freq_range_sel: 0,
            acc_range_sel: 0,
            position_offset: 0,
            speed_scale: 1.0,
            direction_multiplier: 1,
            microsteps_per_full_step: 64,
        }
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        if self.enabled == enabled {
            return;
        }

        self.set_acceleration(self.target_acceleration);
        self.enabled = enabled;

        if enabled {
            let mut dev = block_on(self.device.write());
            dev.desired_mode = C1Mode::SpeedControl;
            dev.desired_stop2_n = true;
            dev.desired_control_byte3 = 0;
            dev.start_requested = false;
            drop(dev);
            self.change_init_state(InitState::Enable);
        } else {
            self.change_init_state(InitState::Off);
            let mut dev = block_on(self.device.write());
            dev.initialized = false;
            dev.desired_mode = C1Mode::SpeedControl;
            dev.desired_stop2_n = true;
            dev.desired_control_byte3 = 0;
            dev.start_requested = false;
            dev.rxpdo.velocity = 0;
            dev.rxpdo.acceleration = self.target_acceleration;
            dev.rxpdo.target_position_l = 0;
            dev.rxpdo.target_position_m = 0;
            dev.rxpdo.target_position_h = 0;
            dev.rxpdo.control_byte1 = 0;
            dev.rxpdo.control_byte2 &= !(C2Flag::ErrorQuit as u8);
            dev.rxpdo.control_byte3 &= !(C3Flag::ResetQuit as u8);
        }
    }

    pub fn request_positioning_mode(&mut self) {
        self.request_mode(C1Mode::SpeedControl, 0, false);
    }

    pub fn jog(&mut self) -> StepperJogWago750671<'_> {
        StepperJogWago750671::new(self)
    }

    pub fn request_fast_stop(&mut self) {
        let mut dev = block_on(self.device.write());
        if !dev.desired_stop2_n {
            return;
        }
        dev.desired_stop2_n = false;
        dev.start_requested = false;
    }

    pub fn start_reference_run(&mut self, direction: Wago750671ReferenceDirection) {
        let control_byte3 = match direction {
            Wago750671ReferenceDirection::Positive => C3Flag::DirectionPos as u8,
            Wago750671ReferenceDirection::Negative => C3Flag::DirectionNeg as u8,
        };
        self.request_mode(C1Mode::Reference, control_byte3, true);
    }

    pub fn move_to_position_steps(&mut self, target_steps: i128, steps_per_second: f64) {
        self.target_position_steps = target_steps;
        let directed_steps_per_second =
            steps_per_second * self.speed_scale * f64::from(self.direction_multiplier);
        let directed_microsteps_per_second =
            directed_steps_per_second * f64::from(self.microsteps_per_full_step);
        let velocity_register =
            self.microsteps_per_second_to_velocity_register(directed_microsteps_per_second);
        self.target_velocity_register = velocity_register;

        let mut dev = block_on(self.device.write());
        dev.desired_stop2_n = true;
        dev.rxpdo.velocity = velocity_register.clamp(-25000, 25000);
        let raw_target =
            (target_steps - self.position_offset).clamp(i32::MIN as i128, i32::MAX as i128) as i32;
        let [b0, b1, b2, _] = raw_target.to_le_bytes();
        dev.rxpdo.target_position_l = b0;
        dev.rxpdo.target_position_m = b1;
        dev.rxpdo.target_position_h = b2;
        drop(dev);

        self.request_mode(C1Mode::SpeedControl, 0, true);
    }

    pub fn set_acceleration(&mut self, acceleration: u16) {
        self.target_acceleration = acceleration;
        let mut dev = block_on(self.device.write());
        dev.rxpdo.acceleration = acceleration;
    }

    pub fn get_speed(&self) -> i32 {
        i32::from(self.target_velocity_register)
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
        dev.rxpdo.control_byte2 = (dev.rxpdo.control_byte2 & 0b1111_1100) | (factor & 0b11);
    }

    pub fn set_acc_range_sel(&mut self, factor: u8) {
        if self.enabled || factor > 3 {
            return;
        }
        self.acc_range_sel = factor;
        let mut dev = block_on(self.device.write());
        dev.rxpdo.control_byte2 = (dev.rxpdo.control_byte2 & 0b1111_0011) | ((factor & 0b11) << 2);
    }

    pub fn set_speed_scale(&mut self, speed_scale: f64) {
        self.speed_scale = speed_scale;
    }

    pub fn set_direction_multiplier(&mut self, direction_multiplier: i8) {
        self.direction_multiplier = if direction_multiplier < 0 { -1 } else { 1 };
    }

    pub fn set_position(&mut self, position: i128) {
        self.position_offset = position - self.get_raw_position();
    }

    pub fn get_position(&self) -> i128 {
        self.get_raw_position() + self.position_offset
    }

    pub fn get_raw_position(&self) -> i128 {
        let dev = block_on(self.device.read());
        decode_i24(
            dev.txpdo.position_l,
            dev.txpdo.position_m,
            dev.txpdo.position_h,
        ) as i128
    }

    pub fn status(&self) -> Wago750671PositioningStatus {
        let s1 = StatusByteS1::from_bits(self.get_status_byte1());
        let s2 = StatusByteS2::from_bits(self.get_status_byte2());
        let s3 = StatusByteS3::from_bits(self.get_status_byte3());

        Wago750671PositioningStatus {
            ready: s1.has_flag(S1Flag::Ready),
            start_ack: s1.has_flag(S1Flag::StartAck),
            positioning_mode_ack: (s1.bits() & (C1Mode::SpeedControl as u8)) != 0,
            reference_mode_ack: (s1.bits() & (C1Mode::Reference as u8)) != 0,
            jog_mode_ack: (s1.bits() & (C1Mode::JogMode as u8)) != 0,
            busy: s2.has_flag(S2Flag::Busy),
            on_target: s2.has_flag(S2Flag::OnTarget),
            reference_ok: s2.has_flag(S2Flag::ReferenceOk),
            standstill: s2.has_flag(S2Flag::StandStill),
            direction_positive: s2.has_flag(S2Flag::Direction),
            input1: s3.has_flag(S3Flag::Input1),
            input2: s3.has_flag(S3Flag::Input2),
        }
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

    pub fn get_actual_velocity_register(&self) -> i16 {
        let dev = block_on(self.device.read());
        dev.txpdo.actual_velocity
    }

    pub fn get_s1_bit3_speed_mode_ack(&self) -> bool {
        let s1 = StatusByteS1::from_bits(self.get_status_byte1());
        (s1.bits() & (C1Mode::SpeedControl as u8)) != 0
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
        StatusByteS2::from_bits(self.get_status_byte2()).has_flag(S2Flag::ReferenceOk)
    }

    pub fn get_s2_busy(&self) -> bool {
        StatusByteS2::from_bits(self.get_status_byte2()).has_flag(S2Flag::Busy)
    }

    pub fn get_s3_bit0(&self) -> bool {
        StatusByteS3::from_bits(self.get_status_byte3()).has_flag(S3Flag::Input1)
    }

    pub fn get_s3_bit1(&self) -> bool {
        StatusByteS3::from_bits(self.get_status_byte3()).has_flag(S3Flag::Input2)
    }

    pub fn get_mode(&self) -> Option<Wago750671Mode> {
        let s1 = StatusByteS1::from_bits(self.get_status_byte1());
        let bits = s1.bits();

        if (bits & (C1Mode::SpeedControl as u8)) != 0 {
            Some(Wago750671Mode::Positioning)
        } else if (bits & (C1Mode::Program as u8)) != 0 {
            Some(Wago750671Mode::Program)
        } else if (bits & (C1Mode::Reference as u8)) != 0 {
            Some(Wago750671Mode::Reference)
        } else if (bits & (C1Mode::JogMode as u8)) != 0 {
            Some(Wago750671Mode::Jog)
        } else if (bits & (C1Mode::Mailbox as u8)) != 0 {
            Some(Wago750671Mode::Mailbox)
        } else {
            None
        }
    }

    fn get_effective_freq_prescaler(&self) -> i32 {
        match self.freq_range_sel {
            0 => 200,
            1 => 80,
            2 => 20,
            3 => 4,
            _ => 200,
        }
    }

    fn microsteps_per_second_to_velocity_register(&self, microsteps_per_second: f64) -> i16 {
        let prescaler = f64::from(self.get_effective_freq_prescaler());
        let register = (microsteps_per_second * prescaler / 80.0).round();
        register.clamp(-25000.0, 25000.0) as i16
    }

    fn request_mode(&mut self, mode: C1Mode, control_byte3: u8, start_requested: bool) {
        let mut dev = block_on(self.device.write());
        let sanitized_control_byte3 = control_byte3 & !(C3Flag::ResetQuit as u8);
        if dev.desired_mode as u8 == mode as u8
            && dev.desired_stop2_n
            && dev.desired_control_byte3 == sanitized_control_byte3
            && dev.start_requested == start_requested
        {
            return;
        }
        dev.desired_mode = mode;
        dev.desired_stop2_n = true;
        dev.desired_control_byte3 = sanitized_control_byte3;
        dev.start_requested = start_requested;
        if self.enabled {
            dev.initialized = false;
            dev.state = InitState::SetMode;
            self.state = InitState::SetMode;
        }
    }

    fn change_init_state(&mut self, state: InitState) {
        self.state = state.clone();
        let mut dev = block_on(self.device.write());
        dev.state = state;
    }

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
