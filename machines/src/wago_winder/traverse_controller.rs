use std::time::Instant;

use control_core::converters::linear_step_converter::LinearStepConverter;
use ethercat_hal::io::stepper_velocity_wago_750_672_traverse::StepperVelocityWago750672Traverse;
use units::ConstZero;
use units::angular_velocity::revolution_per_second;
use units::f64::{AngularVelocity, Length, Velocity};
use units::length::millimeter;
use units::velocity::millimeter_per_second;

#[derive(Debug)]
pub struct TraverseController {
    enabled: bool,
    position: Length,
    limit_inner: Length,
    limit_outer: Length,
    step_size: Length,
    padding: Length,
    state: State,
    fullstep_converter: LinearStepConverter,
    microstep_converter: LinearStepConverter,
    did_change_state: bool,
    stop_stable_raw_position: Option<i128>,
    stop_stable_cycles: u8,
    home_reference_microsteps: Option<i128>,
    pending_anchor_microsteps: Option<i128>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum State {
    NotHomed,
    Idle,
    GoingIn,
    GoingOut,
    Homing(HomingState),
    Traversing(TraversingState),
}

#[derive(Debug, PartialEq, Clone)]
pub enum TraversingState {
    AcquireOuter,
    AcquireOuterStop,
    TraversingIn,
    TraversingOut,
}

#[derive(Debug, PartialEq, Clone)]
pub enum HomingState {
    Initialize,
    EscapeEndstop,
    FindEndstopFineDistancing,
    FindEndstopCoarse,
    CoarseStop,
    FindEndtopFine,
    FineStop,
    Validate(Instant),
}

impl TraverseController {
    const HOMING_SEEK_DIRECTION_SIGN: f64 = 1.0;
    const MOVE_FAST_SPEED_MMPS: f64 = 80.0;
    const HOMING_ESCAPE_SPEED_MMPS: f64 = 10.0;
    const HOMING_FINE_DISTANCING_SPEED_MMPS: f64 = 2.0;

    // per your request
    const HOMING_COARSE_SPEED_MMPS: f64 = 100.0;

    const HOMING_FINE_SPEED_MMPS: f64 = 2.0;

    // only used for "am I effectively there?"
    const POSITION_TOLERANCE_MM: f64 = 0.5;

    const STANDSTILL_THRESHOLD_REGISTER: i16 = 2;
    const STOP_STABLE_CYCLES_REQUIRED: u8 = 3;
    const VALIDATE_SETTLE_MS: u128 = 100;
    const MOVE_ACCELERATION: u16 = 1000;
    const TRAVERSE_ACCELERATION: u16 = 1000;
    const HOMING_ACCELERATION: u16 = 1000;

    // Measured traverse travel is consistently larger than the nominal 32.6 mm/rev.
    // Calibrate the controller/display scale toward physical travel so commanded
    // limits and reported position better match the carriage on the rail.
    const TRAVERSE_MM_PER_REVOLUTION: f64 = 35.7;

    pub fn new(limit_inner: Length, limit_outer: Length, microsteps: u8) -> Self {
        let travel_per_revolution = Length::new::<millimeter>(Self::TRAVERSE_MM_PER_REVOLUTION);

        Self {
            enabled: false,
            position: Length::ZERO,
            limit_inner,
            limit_outer,
            step_size: Length::new::<millimeter>(1.75),
            padding: Length::new::<millimeter>(0.88),
            state: State::NotHomed,
            did_change_state: false,
            fullstep_converter: LinearStepConverter::from_circumference(200, travel_per_revolution),
            microstep_converter: LinearStepConverter::from_circumference(
                200 * microsteps as i16,
                travel_per_revolution,
            ),
            stop_stable_raw_position: None,
            stop_stable_cycles: 0,
            home_reference_microsteps: None,
            pending_anchor_microsteps: None,
        }
    }

    pub const fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    pub fn mark_homed_idle(&mut self) {
        self.position = Length::ZERO;
        self.state = State::Idle;
        self.did_change_state = true;
    }

    pub fn force_not_homed(&mut self) {
        self.position = Length::ZERO;
        self.home_reference_microsteps = None;
        self.pending_anchor_microsteps = None;
        self.state = State::NotHomed;
        self.did_change_state = true;
    }

    pub fn set_limit_inner(&mut self, limit: Length) {
        self.limit_inner = limit;
    }

    pub fn set_limit_outer(&mut self, limit: Length) {
        self.limit_outer = limit;
    }

    pub fn set_step_size(&mut self, step_size: Length) {
        self.step_size = step_size;
    }

    pub fn set_padding(&mut self, padding: Length) {
        self.padding = padding;
    }

    pub fn get_limit_inner(&self) -> Length {
        self.limit_inner
    }

    pub fn get_limit_outer(&self) -> Length {
        self.limit_outer
    }

    pub fn get_step_size(&self) -> Length {
        self.step_size
    }

    pub fn get_padding(&self) -> Length {
        self.padding
    }

    pub fn get_current_position(&self) -> Option<Length> {
        if self.is_homed() {
            Some(self.position)
        } else {
            None
        }
    }

    pub const fn did_change_state(&mut self) -> bool {
        let did_change = self.did_change_state;
        self.did_change_state = false;
        did_change
    }

    pub fn goto_limit_inner(&mut self) {
        self.state = State::GoingIn;
    }

    pub fn goto_limit_outer(&mut self) {
        self.state = State::GoingOut;
    }

    pub fn goto_home(&mut self) {
        self.state = State::Homing(HomingState::Initialize);
    }

    pub fn start_traversing(&mut self, raw_position: i128) {
        let tolerance = Length::new::<millimeter>(Self::POSITION_TOLERANCE_MM);
        let current_position_microsteps = self.logical_microsteps_from_normalized_raw(raw_position);
        let current_position = self
            .microstep_converter
            .steps_to_distance(current_position_microsteps as f64);
        self.position = current_position;
        if current_position >= self.limit_inner - tolerance
            && current_position <= self.limit_inner + tolerance
        {
            self.position = self.limit_inner;
            self.pending_anchor_microsteps =
                Some(self.target_microsteps_for_position(self.limit_inner));
        } else if current_position >= self.limit_outer - tolerance
            && current_position <= self.limit_outer + tolerance
        {
            self.position = self.limit_outer;
            self.pending_anchor_microsteps =
                Some(self.target_microsteps_for_position(self.limit_outer));
        }
        self.state = State::Traversing(TraversingState::AcquireOuter);
    }

    pub const fn is_homed(&self) -> bool {
        !matches!(self.state, State::NotHomed | State::Homing(_))
    }

    pub const fn is_going_in(&self) -> bool {
        matches!(self.state, State::GoingIn)
    }

    pub const fn is_going_out(&self) -> bool {
        matches!(self.state, State::GoingOut)
    }

    pub const fn is_going_home(&self) -> bool {
        matches!(self.state, State::Homing(_))
    }

    pub const fn is_traversing(&self) -> bool {
        matches!(self.state, State::Traversing(_))
    }

    pub fn debug_state(&self) -> String {
        format!("{:?}", self.state)
    }

    pub fn debug_homing_command_sign(&self) -> Option<i8> {
        let sign = match &self.state {
            State::Homing(HomingState::EscapeEndstop) => -Self::HOMING_SEEK_DIRECTION_SIGN,
            State::Homing(HomingState::FindEndstopFineDistancing) => {
                -Self::HOMING_SEEK_DIRECTION_SIGN
            }
            State::Homing(HomingState::FindEndstopCoarse) => Self::HOMING_SEEK_DIRECTION_SIGN,
            State::Homing(HomingState::FindEndtopFine) => Self::HOMING_SEEK_DIRECTION_SIGN,
            _ => return None,
        };
        Some(if sign < 0.0 { -1 } else { 1 })
    }

    pub fn should_auto_release_simulated_home_switch(&self) -> bool {
        matches!(
            self.state,
            State::Homing(HomingState::FindEndstopFineDistancing)
        )
    }

    fn is_at_position(&self, target_position: Length, tolerance: Length) -> bool {
        let upper_tolerance = target_position + tolerance.abs();
        let lower_tolerance = target_position - tolerance.abs();
        self.position >= lower_tolerance && self.position <= upper_tolerance
    }

    fn speed_to_position(&self, target_position: Length, absolute_speed: Velocity) -> Velocity {
        // On WAGO, positive commanded speed moves toward home (inner),
        // negative commanded speed moves away from home (outer).
        if self.position > target_position {
            absolute_speed.abs()
        } else if self.position < target_position {
            -absolute_speed.abs()
        } else {
            Velocity::ZERO
        }
    }

    fn homing_seek_speed(mm_per_second: f64) -> Velocity {
        Velocity::new::<millimeter_per_second>(Self::HOMING_SEEK_DIRECTION_SIGN * mm_per_second)
    }

    fn homing_release_speed(mm_per_second: f64) -> Velocity {
        Velocity::new::<millimeter_per_second>(-Self::HOMING_SEEK_DIRECTION_SIGN * mm_per_second)
    }

    fn target_microsteps_for_position(&self, position: Length) -> i128 {
        self.microstep_converter.distance_to_steps(position).round() as i128
    }

    fn logical_microsteps_from_normalized_raw(&self, normalized_raw_microsteps: i128) -> i128 {
        match self.home_reference_microsteps {
            Some(home_reference_microsteps) => {
                normalized_raw_microsteps - home_reference_microsteps
            }
            None => 0,
        }
    }

    fn tolerance_microsteps(&self, tolerance: Length) -> i128 {
        self.microstep_converter
            .distance_to_steps(tolerance.abs())
            .round()
            .max(1.0) as i128
    }

    fn reached_target_for_goto_steps(
        &self,
        current_position_microsteps: i128,
        target_position: Length,
        tolerance: Length,
    ) -> bool {
        let target_microsteps = self.target_microsteps_for_position(target_position);
        let tolerance_microsteps = self.tolerance_microsteps(tolerance);

        if current_position_microsteps < target_microsteps {
            current_position_microsteps >= target_microsteps - tolerance_microsteps
        } else if current_position_microsteps > target_microsteps {
            current_position_microsteps <= target_microsteps + tolerance_microsteps
        } else {
            true
        }
    }

    pub fn sync_position_velocity(&mut self, traverse: &StepperVelocityWago750672Traverse) {
        let steps =
            self.logical_microsteps_from_normalized_raw(traverse.get_normalized_raw_position());
        self.position = self.microstep_converter.steps_to_distance(steps as f64);
    }

    const fn update_did_change_state(&mut self, old_state: &State) -> bool {
        match self.state {
            State::NotHomed => !matches!(old_state, State::NotHomed),
            State::Idle => !matches!(old_state, State::Idle),
            State::GoingIn => !matches!(old_state, State::GoingIn),
            State::GoingOut => !matches!(old_state, State::GoingOut),
            State::Homing(_) => !matches!(old_state, State::Homing(_)),
            State::Traversing(_) => !matches!(old_state, State::Traversing(_)),
        }
    }

    fn reset_stop_stability(&mut self) {
        self.stop_stable_raw_position = None;
        self.stop_stable_cycles = 0;
    }

    fn raw_position_is_stably_stopped(&mut self, raw_position: i128, standstill: bool) -> bool {
        if !standstill {
            self.reset_stop_stability();
            return false;
        }

        if self.stop_stable_raw_position == Some(raw_position) {
            self.stop_stable_cycles = self.stop_stable_cycles.saturating_add(1);
        } else {
            self.stop_stable_raw_position = Some(raw_position);
            self.stop_stable_cycles = 1;
        }

        self.stop_stable_cycles >= Self::STOP_STABLE_CYCLES_REQUIRED
    }

    pub fn update_speed(
        &mut self,
        traverse: &mut StepperVelocityWago750672Traverse,
        traverse_end_stop: bool,
        spool_speed: AngularVelocity,
    ) {
        if !self.enabled {
            traverse.clear_fast_stop();
            traverse.request_speed_mode();
            traverse.set_acceleration(Self::MOVE_ACCELERATION);
            let _ = traverse.set_speed(0.0);
            return;
        }

        if traverse.is_mailbox_active() {
            return;
        }

        self.sync_position_velocity(traverse);

        let old_state = self.state.clone();
        let entered_coarse_stop = !matches!(old_state, State::Homing(HomingState::CoarseStop));
        let tolerance = Length::new::<millimeter>(Self::POSITION_TOLERANCE_MM);
        let standstill =
            traverse.get_actual_velocity_register().abs() <= Self::STANDSTILL_THRESHOLD_REGISTER;
        let raw_position = traverse.get_normalized_raw_position();

        let mut current_position_microsteps =
            self.logical_microsteps_from_normalized_raw(raw_position);

        if matches!(self.state, State::Traversing(TraversingState::AcquireOuter)) {
            if let Some(target_microsteps) = self.pending_anchor_microsteps.take() {
                self.home_reference_microsteps = Some(raw_position - target_microsteps);
                self.position = self
                    .microstep_converter
                    .steps_to_distance(target_microsteps as f64);
                current_position_microsteps = target_microsteps;
            }
        }

        match self.state.clone() {
            State::NotHomed => {}

            State::Idle => {
                if let Some(target_microsteps) = self.pending_anchor_microsteps {
                    if self.raw_position_is_stably_stopped(raw_position, standstill) {
                        self.reset_stop_stability();
                        traverse.request_set_actual_position_mailbox(target_microsteps);
                        self.home_reference_microsteps = Some(raw_position - target_microsteps);
                        self.position = self
                            .microstep_converter
                            .steps_to_distance(target_microsteps as f64);
                        self.pending_anchor_microsteps = None;
                    }
                } else {
                    self.reset_stop_stability();
                }
            }

            State::GoingIn => {
                if self.reached_target_for_goto_steps(
                    current_position_microsteps,
                    self.limit_inner,
                    tolerance,
                ) {
                    self.pending_anchor_microsteps =
                        Some(self.target_microsteps_for_position(self.limit_inner));
                    self.state = State::Idle;
                }
            }

            State::GoingOut => {
                if self.reached_target_for_goto_steps(
                    current_position_microsteps,
                    self.limit_outer,
                    tolerance,
                ) {
                    self.pending_anchor_microsteps =
                        Some(self.target_microsteps_for_position(self.limit_outer));
                    self.state = State::Idle;
                }
            }

            State::Homing(homing_state) => match homing_state {
                HomingState::Initialize => {
                    self.reset_stop_stability();
                    self.state = if traverse_end_stop {
                        State::Homing(HomingState::EscapeEndstop)
                    } else {
                        State::Homing(HomingState::FindEndstopCoarse)
                    };
                }
                HomingState::EscapeEndstop => {
                    self.reset_stop_stability();
                    if !traverse_end_stop {
                        self.state = State::Homing(HomingState::FindEndstopFineDistancing);
                    }
                }
                HomingState::FindEndstopFineDistancing => {
                    self.reset_stop_stability();
                    if !traverse_end_stop {
                        self.state = State::Homing(HomingState::FindEndtopFine);
                    }
                }
                HomingState::FindEndstopCoarse => {
                    self.reset_stop_stability();
                    if traverse_end_stop {
                        self.state = State::Homing(HomingState::CoarseStop);
                    }
                }
                HomingState::CoarseStop => {
                    if self.raw_position_is_stably_stopped(raw_position, standstill) {
                        self.reset_stop_stability();
                        self.state = State::Homing(HomingState::FindEndstopFineDistancing);
                    }
                }
                HomingState::FindEndtopFine => {
                    self.reset_stop_stability();
                    if traverse_end_stop {
                        self.state = State::Homing(HomingState::FineStop);
                    }
                }
                HomingState::FineStop => {
                    if self.raw_position_is_stably_stopped(raw_position, standstill) {
                        self.reset_stop_stability();
                        traverse.request_set_actual_position_zero_mailbox();
                        self.home_reference_microsteps = Some(raw_position);
                        self.pending_anchor_microsteps = None;
                        self.position = Length::ZERO;
                        self.state = State::Homing(HomingState::Validate(Instant::now()));
                    }
                }
                HomingState::Validate(instant) => {
                    self.reset_stop_stability();
                    if instant.elapsed().as_millis() > Self::VALIDATE_SETTLE_MS {
                        if self.is_at_position(Length::ZERO, Length::new::<millimeter>(0.01)) {
                            self.position = Length::ZERO;
                            self.state = State::Idle;
                        } else {
                            self.state = State::Homing(HomingState::Initialize);
                        }
                    }
                }
            },

            State::Traversing(traversing_state) => match traversing_state {
                TraversingState::AcquireOuter => {
                    if self.reached_target_for_goto_steps(
                        current_position_microsteps,
                        self.limit_outer,
                        tolerance,
                    ) {
                        self.pending_anchor_microsteps =
                            Some(self.target_microsteps_for_position(self.limit_outer));
                        self.reset_stop_stability();
                        self.state = State::Traversing(TraversingState::AcquireOuterStop);
                    }
                }
                TraversingState::AcquireOuterStop => {
                    if let Some(target_microsteps) = self.pending_anchor_microsteps {
                        if self.raw_position_is_stably_stopped(raw_position, standstill) {
                            self.reset_stop_stability();
                            traverse.request_set_actual_position_mailbox(target_microsteps);
                            self.home_reference_microsteps = Some(raw_position - target_microsteps);
                            self.position = self
                                .microstep_converter
                                .steps_to_distance(target_microsteps as f64);
                            self.pending_anchor_microsteps = None;
                            self.state = State::Traversing(TraversingState::TraversingIn);
                        }
                    } else {
                        self.reset_stop_stability();
                    }
                }
                TraversingState::TraversingIn => {
                    if self.position <= self.limit_inner + self.padding {
                        let target_microsteps =
                            self.target_microsteps_for_position(self.limit_inner);
                        self.home_reference_microsteps = Some(raw_position - target_microsteps);
                        self.position = self.limit_inner;
                        self.state = State::Traversing(TraversingState::TraversingOut);
                    }
                }
                TraversingState::TraversingOut => {
                    if self.position >= self.limit_outer - self.padding {
                        let target_microsteps =
                            self.target_microsteps_for_position(self.limit_outer);
                        self.home_reference_microsteps = Some(raw_position - target_microsteps);
                        self.position = self.limit_outer;
                        self.state = State::Traversing(TraversingState::TraversingIn);
                    }
                }
            },
        }

        if !self.did_change_state {
            self.did_change_state = self.update_did_change_state(&old_state);
        }

        if !traverse.is_enabled() {
            traverse.set_enabled(true);
        }

        if matches!(self.state, State::Homing(HomingState::CoarseStop)) {
            if entered_coarse_stop {
                traverse.request_stop_no_ramp_mailbox();
            }

            if traverse.is_mailbox_active() {
                return;
            }

            traverse.clear_fast_stop();
            traverse.request_speed_mode();
            traverse.set_acceleration(Self::HOMING_ACCELERATION);
            let _ = traverse.set_speed(0.0);
            return;
        }

        traverse.clear_fast_stop();
        traverse.request_speed_mode();

        let acceleration = match &self.state {
            State::Traversing(_) => Self::TRAVERSE_ACCELERATION,
            State::GoingIn | State::GoingOut | State::Idle | State::NotHomed => {
                Self::MOVE_ACCELERATION
            }
            State::Homing(_) => Self::HOMING_ACCELERATION,
        };
        traverse.set_acceleration(acceleration);

        let speed = match &self.state {
            State::NotHomed | State::Idle => Velocity::ZERO,

            State::GoingIn => self.speed_to_position(
                self.limit_inner,
                Velocity::new::<millimeter_per_second>(Self::MOVE_FAST_SPEED_MMPS),
            ),

            State::GoingOut => self.speed_to_position(
                self.limit_outer,
                Velocity::new::<millimeter_per_second>(Self::MOVE_FAST_SPEED_MMPS),
            ),

            State::Homing(homing_state) => match homing_state {
                HomingState::Initialize => Velocity::ZERO,
                HomingState::EscapeEndstop => {
                    Self::homing_release_speed(Self::HOMING_ESCAPE_SPEED_MMPS)
                }
                HomingState::FindEndstopFineDistancing => {
                    Self::homing_release_speed(Self::HOMING_FINE_DISTANCING_SPEED_MMPS)
                }
                HomingState::FindEndstopCoarse => {
                    Self::homing_seek_speed(Self::HOMING_COARSE_SPEED_MMPS)
                }
                HomingState::CoarseStop => Velocity::ZERO,
                HomingState::FindEndtopFine => {
                    Self::homing_seek_speed(Self::HOMING_FINE_SPEED_MMPS)
                }
                HomingState::FineStop => Velocity::ZERO,
                HomingState::Validate(_) => Velocity::ZERO,
            },

            State::Traversing(traversing_state) => match traversing_state {
                TraversingState::AcquireOuter => self.speed_to_position(
                    self.limit_outer,
                    Velocity::new::<millimeter_per_second>(Self::MOVE_FAST_SPEED_MMPS),
                ),
                TraversingState::AcquireOuterStop => Velocity::ZERO,
                TraversingState::TraversingIn => self.speed_to_position(
                    self.limit_inner + self.padding - Length::new::<millimeter>(0.01),
                    Self::calculate_traverse_speed(spool_speed, self.step_size).abs(),
                ),
                TraversingState::TraversingOut => self.speed_to_position(
                    self.limit_outer - self.padding + Length::new::<millimeter>(0.01),
                    Self::calculate_traverse_speed(spool_speed, self.step_size).abs(),
                ),
            },
        };

        if speed == Velocity::ZERO {
            let _ = traverse.set_speed(0.0);
            return;
        }

        let steps_per_second = self.fullstep_converter.velocity_to_steps(speed);
        let _ = traverse.set_speed(steps_per_second);
    }

    pub fn calculate_traverse_speed(spool_speed: AngularVelocity, step_size: Length) -> Velocity {
        Velocity::new::<millimeter_per_second>(
            spool_speed.get::<revolution_per_second>().abs() * step_size.get::<millimeter>(),
        )
    }
}
