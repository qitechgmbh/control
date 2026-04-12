use std::time::Instant;

use control_core::converters::linear_step_converter::LinearStepConverter;
use control_core::socketio::namespace::NamespaceCacheingLogic;
use ethercat_hal::io::digital_output::DigitalOutput;
use ethercat_hal::io::stepper_velocity_wago_750_671::StepperVelocityWago750671;
use smol::channel::{Receiver, Sender};
use units::f64::{Length, Velocity};
use units::length::millimeter;
use units::velocity::millimeter_per_second;

use self::api::{StateEvent, WagoTraverseTestMachineEvents, WagoTraverseTestMachineNamespace};
use crate::{
    AsyncThreadMessage, Machine, MachineMessage, VENDOR_QITECH, WAGO_TRAVERSE_TEST_MACHINE,
    machine_identification::{MachineIdentification, MachineIdentificationUnique},
};

pub mod act;
pub mod api;
pub mod new;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TestControlMode {
    Idle,
    ManualMmPerSecond,
    ManualVelocityRegister,
    Controller,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TestMachineMode {
    Standby,
    Hold,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BenchHomingState {
    NotHomed,
    Idle,
    CoarseSeek,
    CoarseStop,
    ReenableForBackoff,
    BackoffRelease,
    Validate,
}

#[derive(Debug)]
pub struct WagoTraverseTestMachine {
    pub api_receiver: Receiver<MachineMessage>,
    pub api_sender: Sender<MachineMessage>,
    pub machine_identification_unique: MachineIdentificationUnique,
    pub main_sender: Option<Sender<AsyncThreadMessage>>,
    pub namespace: WagoTraverseTestMachineNamespace,
    pub last_state_emit: Instant,
    pub switch_output: DigitalOutput,
    pub switch_output_on: bool,
    pub traverse: StepperVelocityWago750671,
    pub fullstep_converter: LinearStepConverter,
    pub microstep_converter: LinearStepConverter,
    pub control_mode: TestControlMode,
    pub mode: TestMachineMode,
    pub homing_state: BenchHomingState,
    pub homing_backoff_target_steps: Option<i128>,
    pub homing_validate_started_at: Option<Instant>,
    pub limit_inner: Length,
    pub limit_outer: Length,
    pub manual_speed_mm_per_second: f64,
    pub manual_velocity_register: i16,
}

impl Machine for WagoTraverseTestMachine {
    fn get_machine_identification_unique(&self) -> MachineIdentificationUnique {
        self.machine_identification_unique.clone()
    }

    fn get_main_sender(&self) -> Option<Sender<AsyncThreadMessage>> {
        self.main_sender.clone()
    }
}

impl WagoTraverseTestMachine {
    pub const MACHINE_IDENTIFICATION: MachineIdentification = MachineIdentification {
        vendor: VENDOR_QITECH,
        machine: WAGO_TRAVERSE_TEST_MACHINE,
    };

    pub fn get_state(&self) -> StateEvent {
        let raw_position_steps = self.traverse.get_raw_position();
        let wrapper_position_steps = self.traverse.get_position();
        let raw_position_mm = self
            .microstep_converter
            .steps_to_distance(raw_position_steps as f64)
            .get::<millimeter>();
        let wrapper_position_mm = self
            .microstep_converter
            .steps_to_distance(wrapper_position_steps as f64)
            .get::<millimeter>();
        let controller_position_mm = self.is_homed().then_some(wrapper_position_mm);
        let actual_velocity_register = self.traverse.get_actual_velocity_register();
        let actual_speed_steps_per_second = self
            .traverse
            .velocity_register_to_steps_per_second(actual_velocity_register);
        let actual_speed_mm_per_second = self
            .fullstep_converter
            .steps_to_velocity(actual_speed_steps_per_second)
            .get::<millimeter_per_second>();

        StateEvent {
            enabled: self.traverse.enabled,
            mode: format!("{:?}", self.mode),
            control_mode: format!("{:?}", self.control_mode),
            controller_state: format!("{:?}", self.homing_state),
            is_homed: self.is_homed(),
            speed_mode_ack: self.traverse.get_s1_bit3_speed_mode_ack(),
            di1: self.traverse.get_s3_bit0(),
            di2: self.traverse.get_s3_bit1(),
            switch_output_on: self.switch_output_on,
            target_velocity_register: self.traverse.target_velocity_register,
            target_speed_steps_per_second: self.traverse.target_speed_fullsteps_per_second,
            actual_velocity_register,
            actual_speed_steps_per_second,
            actual_speed_mm_per_second,
            reference_mode_ack: self.traverse.get_s1_bit5_reference_mode_ack(),
            reference_ok: self.traverse.get_s2_reference_ok(),
            busy: self.traverse.get_s2_busy(),
            target_acceleration: self.traverse.target_acceleration,
            speed_scale: self.traverse.speed_scale,
            direction_multiplier: self.traverse.direction_multiplier,
            freq_range_sel: self.traverse.freq_range_sel,
            acc_range_sel: self.traverse.acc_range_sel,
            raw_position_steps: raw_position_steps as i64,
            wrapper_position_steps: wrapper_position_steps as i64,
            raw_position_mm,
            wrapper_position_mm,
            controller_position_mm,
            limit_inner_mm: self.limit_inner.get::<millimeter>(),
            limit_outer_mm: self.limit_outer.get::<millimeter>(),
            manual_speed_mm_per_second: self.manual_speed_mm_per_second,
            manual_velocity_register: self.manual_velocity_register,
            control_byte1: self.traverse.get_control_byte1(),
            control_byte2: self.traverse.get_control_byte2(),
            control_byte3: self.traverse.get_control_byte3(),
            status_byte1: self.traverse.get_status_byte1(),
            status_byte2: self.traverse.get_status_byte2(),
            status_byte3: self.traverse.get_status_byte3(),
        }
    }

    pub fn emit_state(&mut self) {
        let event = self.get_state().build();
        self.namespace
            .emit(WagoTraverseTestMachineEvents::State(event));
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.traverse.set_enabled(enabled);
        self.manual_speed_mm_per_second = 0.0;
        self.manual_velocity_register = 0;
        self.control_mode = TestControlMode::Idle;
        self.switch_output_on = false;
        self.switch_output.set(false);
        self.traverse.clear_fast_stop();
        self.traverse.request_speed_mode();
        self.traverse.set_speed(0.0);
        self.traverse.set_position(0);
        self.homing_state = BenchHomingState::NotHomed;
        self.homing_backoff_target_steps = None;
        self.homing_validate_started_at = None;
        self.mode = if enabled {
            TestMachineMode::Hold
        } else {
            TestMachineMode::Standby
        };
        self.emit_state();
    }

    pub fn set_mode(&mut self, mode: TestMachineMode) {
        self.mode = mode;
        self.set_enabled(matches!(mode, TestMachineMode::Hold));
    }

    pub fn set_switch_output(&mut self, on: bool) {
        self.switch_output_on = on;
        self.switch_output.set(on);
        self.emit_state();
    }

    pub fn set_manual_speed_mm_per_second(&mut self, speed_mm_per_second: f64) {
        self.manual_speed_mm_per_second = speed_mm_per_second;
        self.control_mode = TestControlMode::ManualMmPerSecond;
        self.traverse.request_speed_mode();
        self.emit_state();
    }

    pub fn set_manual_velocity_register(&mut self, velocity_register: i16) {
        self.manual_velocity_register = velocity_register;
        self.control_mode = TestControlMode::ManualVelocityRegister;
        self.traverse.request_speed_mode();
        self.emit_state();
    }

    pub fn stop(&mut self) {
        self.manual_speed_mm_per_second = 0.0;
        self.manual_velocity_register = 0;
        self.control_mode = TestControlMode::Idle;
        self.traverse.request_speed_mode();
        self.traverse.set_speed(0.0);
        self.emit_state();
    }

    pub fn goto_home(&mut self) {
        self.switch_output_on = false;
        self.switch_output.set(false);
        self.traverse.set_enabled(false);
        self.traverse.set_enabled(true);
        self.traverse.clear_fast_stop();
        self.homing_backoff_target_steps = None;
        self.homing_validate_started_at = None;
        self.control_mode = TestControlMode::Controller;
        self.traverse.request_speed_mode();
        self.homing_state = BenchHomingState::CoarseSeek;
        self.emit_state();
    }

    pub fn goto_limit_inner(&mut self) {
        self.control_mode = TestControlMode::Idle;
        self.emit_state();
    }

    pub fn goto_limit_outer(&mut self) {
        self.control_mode = TestControlMode::Idle;
        self.emit_state();
    }

    pub fn force_not_homed(&mut self) {
        self.manual_speed_mm_per_second = 0.0;
        self.manual_velocity_register = 0;
        self.control_mode = TestControlMode::Idle;
        self.traverse.clear_fast_stop();
        self.traverse.request_speed_mode();
        self.traverse.set_speed(0.0);
        self.homing_state = BenchHomingState::NotHomed;
        self.homing_backoff_target_steps = None;
        self.homing_validate_started_at = None;
        self.emit_state();
    }

    pub fn set_position_steps(&mut self, position_steps: i64) {
        self.traverse.set_position(position_steps as i128);
        self.emit_state();
    }

    pub fn set_position_mm(&mut self, position_mm: f64) {
        let steps = self
            .microstep_converter
            .distance_to_steps(Length::new::<millimeter>(position_mm))
            .round() as i128;
        self.traverse.set_position(steps);
        self.emit_state();
    }

    pub fn set_limit_inner_mm(&mut self, limit_inner_mm: f64) {
        self.limit_inner = Length::new::<millimeter>(limit_inner_mm);
        self.emit_state();
    }

    pub fn set_limit_outer_mm(&mut self, limit_outer_mm: f64) {
        self.limit_outer = Length::new::<millimeter>(limit_outer_mm);
        self.emit_state();
    }

    pub fn set_speed_scale(&mut self, speed_scale: f64) {
        self.traverse.set_speed_scale(speed_scale);
        self.emit_state();
    }

    pub fn set_direction_multiplier(&mut self, direction_multiplier: i8) {
        self.traverse.set_direction_multiplier(direction_multiplier);
        self.emit_state();
    }

    pub fn set_freq_range_sel(&mut self, factor: u8) {
        self.traverse.set_freq_range_sel(factor);
        self.emit_state();
    }

    pub fn set_acc_range_sel(&mut self, factor: u8) {
        self.traverse.set_acc_range_sel(factor);
        self.emit_state();
    }

    pub fn set_acceleration(&mut self, acceleration: u16) {
        self.traverse.set_acceleration(acceleration);
        self.emit_state();
    }

    fn is_homed(&self) -> bool {
        matches!(self.homing_state, BenchHomingState::Idle)
    }

    pub fn act_drive_mode(&mut self) {
        let di2 = self.traverse.get_s3_bit1();
        match self.control_mode {
            TestControlMode::Idle => {
                self.traverse.clear_fast_stop();
                self.traverse.request_speed_mode();
                self.traverse.set_speed(0.0);
            }
            TestControlMode::ManualMmPerSecond => {
                if di2 {
                    self.traverse.request_fast_stop();
                    self.traverse.set_speed(0.0);
                    return;
                }
                self.traverse.clear_fast_stop();
                self.traverse.request_speed_mode();
                let speed = Velocity::new::<units::velocity::millimeter_per_second>(
                    self.manual_speed_mm_per_second,
                );
                let steps_per_second = self.fullstep_converter.velocity_to_steps(speed);
                self.traverse.set_speed(steps_per_second);
            }
            TestControlMode::ManualVelocityRegister => {
                if di2 {
                    self.traverse.request_fast_stop();
                    self.traverse.set_velocity_register(0);
                    return;
                }
                self.traverse.clear_fast_stop();
                self.traverse.request_speed_mode();
                self.traverse
                    .set_velocity_register(self.manual_velocity_register);
            }
            TestControlMode::Controller => {
                const COARSE_REGISTER: i16 = 5000;
                const BACKOFF_REGISTER: i16 = -1000;
                const BACKOFF_DISTANCE_MM: f64 = 2.0;
                let current_steps = self.traverse.get_raw_position();
                match self.homing_state {
                    BenchHomingState::NotHomed => {
                        self.control_mode = TestControlMode::Idle;
                        self.traverse.clear_fast_stop();
                        self.traverse.request_speed_mode();
                        self.traverse.set_velocity_register(0);
                    }
                    BenchHomingState::Idle => {
                        self.control_mode = TestControlMode::Idle;
                        self.traverse.clear_fast_stop();
                        self.traverse.request_speed_mode();
                        self.traverse.set_velocity_register(0);
                    }
                    BenchHomingState::CoarseSeek => {
                        self.traverse.clear_fast_stop();
                        self.traverse.request_speed_mode();
                        self.traverse.set_acceleration(100);
                        self.traverse.set_velocity_register(COARSE_REGISTER);
                        if di2 {
                            self.traverse.set_enabled(false);
                            self.homing_state = BenchHomingState::CoarseStop;
                        }
                    }
                    BenchHomingState::CoarseStop => {
                        if self.switch_output_on {
                            self.switch_output_on = false;
                            self.switch_output.set(false);
                        }
                        if self.traverse.get_actual_velocity_register().abs() <= 2 {
                            self.traverse.set_enabled(true);
                            self.traverse.clear_fast_stop();
                            self.traverse.request_speed_mode();
                            self.homing_backoff_target_steps = Some(current_steps);
                            self.homing_state = BenchHomingState::ReenableForBackoff;
                        }
                    }
                    BenchHomingState::ReenableForBackoff => {
                        self.traverse.clear_fast_stop();
                        self.traverse.request_speed_mode();
                        self.traverse.set_acceleration(100);
                        self.traverse.set_velocity_register(BACKOFF_REGISTER);
                        self.homing_state = BenchHomingState::BackoffRelease;
                    }
                    BenchHomingState::BackoffRelease => {
                        self.traverse.clear_fast_stop();
                        self.traverse.request_speed_mode();
                        self.traverse.set_velocity_register(BACKOFF_REGISTER);
                        if let Some(start_steps) = self.homing_backoff_target_steps {
                            let backoff_steps = self
                                .microstep_converter
                                .distance_to_steps(Length::new::<millimeter>(BACKOFF_DISTANCE_MM))
                                .round() as i128;
                            if (current_steps - start_steps).abs() >= backoff_steps {
                                self.traverse.set_velocity_register(0);
                                self.traverse.set_position(0);
                                self.homing_validate_started_at = Some(Instant::now());
                                self.homing_state = BenchHomingState::Validate;
                            }
                        }
                    }
                    BenchHomingState::Validate => {
                        self.traverse.clear_fast_stop();
                        self.traverse.request_speed_mode();
                        self.traverse.set_velocity_register(0);
                        if self
                            .homing_validate_started_at
                            .is_some_and(|started| started.elapsed().as_millis() > 100)
                        {
                            self.homing_state = BenchHomingState::Idle;
                            self.control_mode = TestControlMode::Idle;
                            self.homing_backoff_target_steps = None;
                            self.homing_validate_started_at = None;
                        }
                    }
                }
            }
        }
    }
}
