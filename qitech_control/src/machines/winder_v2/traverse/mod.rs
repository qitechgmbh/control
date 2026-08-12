use std::cell::RefCell;
use std::cmp::Ordering;
use std::rc::Rc;
use std::time::Duration;
use std::time::Instant;

use qitech_framework::machine::ActErrorKind;
use qitech_framework::machine::ActResult;
use qitech_framework::machine::ConfigProperty;
use qitech_framework::machine::Measurement;
use qitech_framework::machine::OperationCapability;
use qitech_framework::machine::StateProperty;
use qitech_lib::ethercat_hal::io::stepper_velocity_el70x1::StepperVelocityEL70x1Device;
use qitech_lib::units::AngularVelocity;
use qitech_lib::units::ConstZero;
use qitech_lib::units::Length;
use qitech_lib::units::Velocity;
use qitech_lib::units::angular_velocity::revolution_per_second;
use qitech_lib::units::length::millimeter;
use qitech_lib::units::velocity::millimeter_per_second;

use crate::converters::LinearStepConverter;

mod types;
use types::HomingState;
pub use types::Mode;
pub use types::State;
use types::TraversingState;

mod laser_pointer;
pub(super) use laser_pointer::LaserPointer;

pub struct Traverse {
    pub(super) device: Rc<RefCell<dyn StepperVelocityEL70x1Device>>,

    // --- sub devices ---
    pub(super) laser_pointer: LaserPointer,

    // --- config ---
    pub(super) limit_inner: ConfigProperty<Length>,
    pub(super) limit_outer: ConfigProperty<Length>,
    pub(super) step_size: ConfigProperty<Length>,
    pub(super) padding: ConfigProperty<Length>,

    // --- state ---
    pub(super) mode: StateProperty<Mode>,
    pub(super) state: StateProperty<State>,
    pub(super) endstop_triggered: StateProperty<bool>,

    // --- measurements ---
    pub(super) position: Measurement<Length>,

    // --- converters ---
    pub(super) fullstep_converter: LinearStepConverter,
    pub(super) microstep_converter: LinearStepConverter,
}

// --- constants ---
impl Traverse {
    const PORT: usize = 0;
}

// --- public interface ---
impl Traverse {
    pub fn apply_mode(&mut self, mode: Mode) -> Result<(), ActErrorKind> {
        if self.mode.set(mode) {
            let enabled = match mode {
                Mode::Standby => false,
                Mode::Hold => {
                    _ = self.goto_home();
                    true
                }
                Mode::Traverse => {
                    _ = self.start_traversing();
                    true
                }
            };

            let mut dev = self.device.borrow_mut();
            dev.set_enabled(Self::PORT, enabled);
        }

        Ok(())
    }

    pub fn update(&mut self, now: Instant, spool_speed: AngularVelocity) {
        self.sync();

        if self.update_state(now)
            && matches!(self.state.get(), State::Homing(HomingState::Validate(_)))
        {
            // set position to zero if we enter validate state
            self.device.borrow_mut().set_position(Self::PORT, 0);
        }

        self.update_speed(spool_speed);
    }

    // --- goto limit inner ---
    pub fn goto_limit_inner(&mut self) -> ActResult {
        assert!(self.goto_limit_capability().is_allowed());
        self.state.set(State::GoingIn);
        Ok(())
    }

    pub fn goto_limit_inner_capability(&self) -> OperationCapability {
        let base = self.goto_limit_capability();

        if base.is_forbidden() {
            base
        } else if matches!(self.state.get(), State::GoingIn) {
            OperationCapability::forbidden("already going in")
        } else {
            OperationCapability::Allowed
        }
    }

    // --- goto limit outer ---
    pub fn goto_limit_outer(&mut self) -> ActResult {
        assert!(self.goto_limit_capability().is_allowed());
        self.state.set(State::GoingOut);
        Ok(())
    }

    pub fn goto_limit_outer_capability(&self) -> OperationCapability {
        let base = self.goto_limit_capability();

        if base.is_forbidden() {
            base
        } else if matches!(self.state.get(), State::GoingOut) {
            OperationCapability::forbidden("already going out")
        } else {
            OperationCapability::Allowed
        }
    }

    // --- goto limit capability ---
    fn goto_limit_capability(&self) -> OperationCapability {
        if self.mode.get() != Mode::Hold {
            return OperationCapability::forbidden("not in hold mode");
        }

        if self.state.get() == State::NotHomed {
            return OperationCapability::forbidden("not homed");
        }

        if matches!(self.state.get(), State::GoingOut) {
            return OperationCapability::forbidden("already going out");
        }

        if matches!(self.state.get(), State::Homing(_)) {
            return OperationCapability::forbidden("currently homing");
        }

        if matches!(self.state.get(), State::Traversing(_)) {
            return OperationCapability::forbidden("currently traversing");
        }

        OperationCapability::Allowed
    }

    // --- goto home ---
    pub fn goto_home(&mut self) -> ActResult {
        self.state.set(State::Homing(HomingState::Initialize));
        Ok(())
    }

    pub fn goto_home_capability(&self) -> OperationCapability {
        if self.mode.get() != Mode::Hold {
            return OperationCapability::forbidden("not in hold mode");
        }

        if matches!(self.state.get(), State::Homing(_)) {
            return OperationCapability::forbidden("currently homing");
        }

        if matches!(self.state.get(), State::Traversing(_)) {
            return OperationCapability::forbidden("currently traversing");
        }

        OperationCapability::Allowed
    }

    // --- start traversing ---
    pub fn start_traversing(&mut self) -> ActResult {
        self.state.set(State::Traversing(TraversingState::GoingOut));
        Ok(())
    }

    // --- callbacks ---
    pub fn on_limit_inner_changed(&mut self) -> ActResult {
        let limit_inner = self.limit_inner.get();
        let offet_min = Length::new::<millimeter>(0.9);
        _ = self.limit_outer.set(limit_inner + offet_min);
        Ok(())
    }
}

// --- state update ---
impl Traverse {
    fn update_state(&mut self, now: Instant) -> bool {
        let next_state = match self.state.get() {
            State::GoingIn if self.is_near(self.limit_inner.get()) => State::Idle,
            State::GoingOut if self.is_near(self.limit_outer.get()) => State::Idle,
            State::Homing(state) => self.next_homing_state(now, state),
            State::Traversing(state) => self.next_traversing_state(state),
            current => current,
        };

        self.state.set(next_state)
    }

    fn next_homing_state(&self, now: Instant, state: HomingState) -> State {
        const VALIDATION_DURATION: Duration = Duration::from_millis(100);
        let endstop_triggered = self.endstop_triggered.get();

        use HomingState::*;
        let homing_state = match state {
            // --- re-home state pipeline ---
            Initialize if endstop_triggered => EscapeEndstop,
            EscapeEndstop if !endstop_triggered => FindEndstopFineDistancing,

            // --- find-home state pipeline ---
            Initialize => FindEndstopCoarse,
            FindEndstopCoarse if endstop_triggered => FindEndstopFineDistancing,

            // --- fine positioning ---
            FindEndstopFineDistancing if !endstop_triggered => FindEndstopFine,
            FindEndstopFine if endstop_triggered => Validate(now),

            // --- validate ---
            Validate(started) if started.elapsed() > VALIDATION_DURATION => {
                if self.is_near(Length::ZERO) {
                    return State::Idle;
                }

                // position is not 0.0, redo homing
                Initialize
            }

            other => other,
        };

        State::Homing(homing_state)
    }

    fn next_traversing_state(&self, state: TraversingState) -> State {
        let position = self.position.get();
        let padding = self.padding.get();
        let limit_outer = self.limit_outer.get();
        let limit_inner = self.limit_inner.get();

        use TraversingState::*;
        let traversing_state = match state {
            GoingOut if position >= limit_outer - padding => TraversingIn,
            TraversingIn if position <= limit_inner + padding => TraversingOut,
            TraversingOut if position >= limit_outer - padding => TraversingIn,
            other => other,
        };

        State::Traversing(traversing_state)
    }
}

// --- speed computation ---
impl Traverse {
    fn speed_far() -> Velocity {
        Velocity::new::<millimeter_per_second>(100.0)
    }

    fn speed_near() -> Velocity {
        Velocity::new::<millimeter_per_second>(10.0)
    }

    fn update_speed(&mut self, spool_speed: AngularVelocity) {
        use State::*;

        let speed = match self.state.get() {
            NotHomed | Idle => Velocity::ZERO,
            GoingIn => self.speed_towards_with_falloff(self.limit_inner.get()),
            GoingOut => self.speed_towards_with_falloff(self.limit_outer.get()),
            Homing(state) => self.get_speed_homing(state),
            Traversing(state) => self.get_speed_traversing(state, spool_speed),
        };

        let steps_per_second = self.fullstep_converter.velocity_to_steps(speed);
        _ = self
            .device
            .borrow_mut()
            .set_speed(Self::PORT, steps_per_second);
    }

    /// Moves towards `target`, slowing down to `SPEED_NEAR` once close.
    fn speed_towards_with_falloff(&self, target: Length) -> Velocity {
        let speed = if self.is_far(target) {
            Self::speed_far()
        } else {
            Self::speed_near()
        };

        self.speed_towards(target, speed)
    }

    fn get_speed_homing(&self, state: HomingState) -> Velocity {
        use HomingState::*;

        let speed = match state {
            Initialize => 0.0,
            EscapeEndstop => 10.0,
            FindEndstopCoarse => -100.0,
            FindEndstopFineDistancing => 2.0,
            FindEndstopFine => -2.0,
            Validate(_) => 0.0,
        };

        Velocity::new::<millimeter_per_second>(speed)
    }

    fn get_speed_traversing(
        &self,
        state: TraversingState,
        spool_speed: AngularVelocity,
    ) -> Velocity {
        use TraversingState::*;

        let position_offset = Length::new::<millimeter>(0.01);
        let inner_target = self.limit_inner.get() + self.padding.get() - position_offset;
        let outer_target = self.limit_outer.get() - self.padding.get() + position_offset;

        let traverse_speed = Velocity::new::<millimeter_per_second>(
            spool_speed.get::<revolution_per_second>() * self.step_size.get_as::<millimeter>(),
        );

        match state {
            GoingOut => self.speed_towards(outer_target, Self::speed_far()),
            TraversingIn => self.speed_towards(inner_target, traverse_speed),
            TraversingOut => self.speed_towards(outer_target, traverse_speed),
        }
    }
}

// --- utils ---
impl Traverse {
    const PORT_END_STOP: usize = 0;

    fn position_tolerance() -> Length {
        Length::new::<millimeter>(0.01)
    }

    fn far_threshold() -> Length {
        Length::new::<millimeter>(1.0)
    }

    fn sync(&mut self) {
        let device = self.device.borrow();

        let is_triggered = device
            .get_digital_input(Self::PORT_END_STOP)
            .unwrap_or(false);
        self.endstop_triggered.set(is_triggered);

        let steps = device.get_position(Self::PORT);
        let position = self.microstep_converter.steps_to_distance(steps as f64);
        self.position.set(position);
    }

    fn is_near(&self, target_position: Length) -> bool {
        let delta = (self.position.get() - target_position).abs();
        delta <= Self::position_tolerance()
    }

    fn is_far(&self, target_position: Length) -> bool {
        let delta = (self.position.get() - target_position).abs();
        delta > Self::far_threshold()
    }

    /// Returns a signed velocity pointing from the current position towards `target_position`.
    fn speed_towards(&self, target_position: Length, speed: Velocity) -> Velocity {
        let position = self.position.get();
        let speed = speed.abs();

        match position.partial_cmp(&target_position) {
            Some(Ordering::Greater) => -speed,
            Some(Ordering::Less) => speed,
            _ => Velocity::ZERO,
        }
    }
}
