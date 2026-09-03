use std::cell::RefCell;
use std::cmp::Ordering;
use std::rc::Rc;
use std::time::Duration;

use qitech_control_core::converters::linear_step_converter::LinearStepConverter;
use qitech_framework::machine::ActErrorKind;
use qitech_framework::machine::ActResult;
use qitech_framework::machine::BuildContext;
use qitech_framework::machine::BuildResult;
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

use crate::machines::winder_v2::WinderV1;

mod types;
use types::HomingState;
pub use types::Mode;
pub use types::State;
use types::TraversingState;

pub struct Traverse {
    // --- hardware ---
    device: Rc<RefCell<dyn StepperVelocityEL70x1Device>>,

    // --- config ---
    limit_inner: ConfigProperty<Length>,
    limit_outer: ConfigProperty<Length>,
    step_size: ConfigProperty<Length>,
    padding: ConfigProperty<Length>,

    // --- state ---
    mode: StateProperty<Mode>,
    state: StateProperty<State>,
    is_homed: StateProperty<bool>,
    endstop_triggered: StateProperty<bool>,

    // --- measurements ---
    position: Measurement<Length>,

    // --- converters ---
    fullstep_converter: LinearStepConverter,
    microstep_converter: LinearStepConverter,
}

// --- constants ---
impl Traverse {
    const PORT: usize = 0;
    const PORT_END_STOP: usize = 0;

    fn position_tolerance() -> Length {
        Length::new::<millimeter>(0.01)
    }

    fn far_threshold() -> Length {
        Length::new::<millimeter>(1.0)
    }

    fn speed_far() -> Velocity {
        Velocity::new::<millimeter_per_second>(100.0)
    }

    fn speed_near() -> Velocity {
        Velocity::new::<millimeter_per_second>(10.0)
    }

    fn limit_gap_min() -> Length {
        Length::new::<millimeter>(0.9)
    }
}

// --- init ---
impl Traverse {
    pub fn init<const VARIANT: usize>(
        ctx: &mut BuildContext,
        device: Rc<RefCell<dyn StepperVelocityEL70x1Device>>,
    ) -> BuildResult<Self> {
        const MICROSTEPS: i16 = 64;

        ctx.command("traverse.goto_home")
            .can_execute(|m: &WinderV1<VARIANT>| m.traverse.goto_home_capability())
            .execute(|m: &mut WinderV1<VARIANT>| m.traverse.goto_home())
            .build()?;

        ctx.command("traverse.goto_limit_inner")
            .can_execute(|m: &WinderV1<VARIANT>| m.traverse.goto_limit_inner_capability())
            .execute(|m: &mut WinderV1<VARIANT>| m.traverse.goto_limit_inner())
            .build()?;

        ctx.command("traverse.goto_limit_outer")
            .can_execute(|m: &WinderV1<VARIANT>| m.traverse.goto_limit_outer_capability())
            .execute(|m: &mut WinderV1<VARIANT>| m.traverse.goto_limit_outer())
            .build()?;

        Ok(Self {
            device,
            limit_inner: ctx
                .config::<millimeter>("traverse.limit_inner")
                .on_external_changed(|m: &mut WinderV1<VARIANT>| {
                    m.traverse.on_limit_inner_changed()
                })
                .default(22.0)
                .minimum(0.0)
                .maximum(92.0 - Self::limit_gap_min().get::<millimeter>())
                .build()?,

            limit_outer: ctx
                .config::<millimeter>("traverse.limit_outer")
                .on_external_changed(|m: &mut WinderV1<VARIANT>| {
                    m.traverse.on_limit_outer_changed()
                })
                .default(92.0)
                .minimum(22.0 + Self::limit_gap_min().get::<millimeter>())
                .maximum(92.0)
                .build()?,
            step_size: ctx
                .config::<millimeter>("traverse.step_size")
                .default(1.75)
                .build()?,
            padding: ctx
                .config::<millimeter>("traverse.padding")
                .default(0.88)
                .build()?,
            mode: ctx.state::<Mode>("traverse.mode").build()?,
            state: ctx.state::<State>("traverse.state").build()?,
            is_homed: ctx.state::<bool>("traverse.homed").build()?,
            endstop_triggered: ctx.state::<bool>("traverse.endstop_triggered").build()?,
            position: ctx.measurement::<millimeter>("traverse.position").build()?,

            // --- converters ---
            fullstep_converter: LinearStepConverter::from_circumference(
                200,
                Length::new::<millimeter>(32.0),
            ),
            microstep_converter: LinearStepConverter::from_circumference(
                200 * MICROSTEPS,
                Length::new::<millimeter>(32.0),
            ),
        })
    }
}

// --- public interface ---
impl Traverse {
    pub fn is_homed(&self) -> bool {
        self.is_homed.get()
    }

    pub fn is_homing(&self) -> bool {
        matches!(self.state.get(), State::Homing(_))
    }

    pub fn set_mode(&mut self, mode: Mode) -> Result<(), ActErrorKind> {
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

    pub fn update(&mut self, dt: Duration, spool_speed: AngularVelocity) {
        self.sync();

        if self.update_state(dt)
            && matches!(self.state.get(), State::Homing(HomingState::Validate(_)))
        {
            // set position to zero if we enter validate state
            self.device.borrow_mut().set_position(Self::PORT, 0);
        }

        self.update_speed(spool_speed);
    }

    // --- goto home ---
    fn goto_home(&mut self) -> ActResult {
        self.state.set(State::Homing(HomingState::Initialize));
        Ok(())
    }

    fn goto_home_capability(&self) -> OperationCapability {
        if self.mode.get() != Mode::Hold {
            return OperationCapability::forbidden("requires hold mode");
        }

        if matches!(self.state.get(), State::Homing(_)) {
            return OperationCapability::forbidden("already homing");
        }

        if matches!(self.state.get(), State::Traversing(_)) {
            return OperationCapability::forbidden("currently traversing");
        }

        OperationCapability::Allowed
    }

    // --- goto limit inner ---
    fn goto_limit_inner(&mut self) -> ActResult {
        self.state.set(State::GoingIn);
        Ok(())
    }

    fn goto_limit_inner_capability(&self) -> OperationCapability {
        let base = self.goto_limit_capability();

        if base.is_forbidden() {
            base
        } else if matches!(self.state.get(), State::GoingIn) {
            OperationCapability::forbidden("already moving inward")
        } else {
            OperationCapability::Allowed
        }
    }

    // --- goto limit outer ---
    fn goto_limit_outer(&mut self) -> ActResult {
        self.state.set(State::GoingOut);
        Ok(())
    }

    pub fn goto_limit_outer_capability(&self) -> OperationCapability {
        let base = self.goto_limit_capability();

        if base.is_forbidden() {
            base
        } else if matches!(self.state.get(), State::GoingOut) {
            OperationCapability::forbidden("already moving outward")
        } else {
            OperationCapability::Allowed
        }
    }

    // --- goto limit capability ---
    fn goto_limit_capability(&self) -> OperationCapability {
        if self.mode.get() != Mode::Hold {
            return OperationCapability::forbidden("requires hold mode");
        }

        if !self.is_homed.get() {
            return OperationCapability::forbidden("requires homing");
        }

        if matches!(self.state.get(), State::Homing(_)) {
            return OperationCapability::forbidden("already homing");
        }

        if matches!(self.state.get(), State::Traversing(_)) {
            return OperationCapability::forbidden("currently traversing");
        }

        OperationCapability::Allowed
    }

    // --- start traversing ---
    fn start_traversing(&mut self) -> ActResult {
        self.state.set(State::Traversing(TraversingState::GoingOut));
        Ok(())
    }

    // --- callbacks ---
    fn on_limit_inner_changed(&mut self) -> ActResult {
        let min_outer = self.limit_inner.get() + Self::limit_gap_min();

        self.limit_outer
            .set_min_clamped(min_outer)
            .expect("Failed to update limits");

        Ok(())
    }

    fn on_limit_outer_changed(&mut self) -> ActResult {
        let max_inner = self.limit_outer.get() - Self::limit_gap_min();

        self.limit_inner
            .set_max_clamped(max_inner)
            .expect("Failed to update limits");

        Ok(())
    }
}

// --- state update ---
impl Traverse {
    fn update_state(&mut self, dt: Duration) -> bool {
        let next_state = match self.state.get() {
            State::GoingIn if self.is_near(self.limit_inner.get()) => State::Idle,
            State::GoingOut if self.is_near(self.limit_outer.get()) => State::Idle,
            State::Homing(state) => self.update_state_homing(dt, state),
            State::Traversing(state) => self.update_state_traversing(state),
            current => current,
        };

        self.state.set(next_state)
    }

    fn update_state_homing(&mut self, dt: Duration, state: HomingState) -> State {
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
            FindEndstopFine if endstop_triggered => Validate(VALIDATION_DURATION),

            // --- validate ---
            Validate(remaining) => {
                let remaining = remaining.saturating_sub(dt);

                if !remaining.is_zero() {
                    return State::Homing(Validate(remaining));
                }

                if self.is_near(Length::ZERO) {
                    self.is_homed.set(true);
                    return State::Idle;
                }

                // position is not 0.0, redo homing
                Initialize
            }

            other => other,
        };

        State::Homing(homing_state)
    }

    fn update_state_traversing(&self, state: TraversingState) -> State {
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
    fn update_speed(&mut self, spool_speed: AngularVelocity) {
        use State::*;

        let speed = match self.state.get() {
            Idle => Velocity::ZERO,
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

        let speed_f64 = match state {
            Initialize => 0.0,
            EscapeEndstop => 10.0,
            FindEndstopCoarse => -100.0,
            FindEndstopFineDistancing => 2.0,
            FindEndstopFine => -2.0,
            Validate(_) => 0.0,
        };

        Velocity::new::<millimeter_per_second>(speed_f64)
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
