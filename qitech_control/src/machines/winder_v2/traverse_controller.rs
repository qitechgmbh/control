use super::{TRAVERSE_END_STOP_PORT, TRAVERSE_PORT};
use crate::converters::linear_step_converter::LinearStepConverter;
use qitech_framework::machine::{ActResult, ConfigProperty, Measurement, StateProperty};
use qitech_lib::ethercat_hal::io::stepper_velocity_el70x1::StepperVelocityEL70x1Device;
use qitech_lib::units::ConstZero;
use qitech_lib::units::angular_velocity::revolution_per_second;
use qitech_lib::units::f64::{AngularVelocity, Length, Velocity};
use qitech_lib::units::length::millimeter;
use qitech_lib::units::velocity::millimeter_per_second;
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Instant;

pub struct Traverse {
    enabled: bool,
    device: Rc<RefCell<dyn StepperVelocityEL70x1Device>>,

    position: Measurement<Option<Length>>,
    pub limit_inner: ConfigProperty<Length>,
    pub limit_outer: ConfigProperty<Length>,
    step_size: ConfigProperty<Length>,
    padding: ConfigProperty<Length>,
    state: StateProperty<State>,
    fullstep_converter: LinearStepConverter,
    microstep_converter: LinearStepConverter,
}

impl qitech_framework::__private::PropertyType for State {
    type Constraints = qitech_framework::__private::EnumConstraints<Self>;
}

impl qitech_framework::__private::PropertyAdapter for State {
    type Type = State;
    type Input = State;

    fn convert_input(input: Self::Input) -> Self::Type {
        input
    }

    fn into_scalar(value: Self::Type) -> qitech_framework::__private::ScalarValue {
        let s = match value {
            State::NotHomed => "not_homed",
            State::Idle => "idle",
            State::GoingIn => "going_in",
            State::GoingOut => "going_out",
            State::Homing(state) => match state {
                HomingState::Initialize => "initialize",
                HomingState::EscapeEndstop => "escape_endstop",
                HomingState::FindEndstopFineDistancing => "find_endstop_fine_distancing",
                HomingState::FindEndstopCoarse => "find_endstop_coarse",
                HomingState::FindEndtopFine => "find_endstop_fine",
                HomingState::Validate(_) => "validate",
            },
            State::Traversing(state) => match state {
                TraversingState::GoingOut => "going_out (traverse)",
                TraversingState::TraversingIn => "traversing_in",
                TraversingState::TraversingOut => "traversing_out",
            },
        }
        .to_string();

        qitech_framework::__private::ScalarValue::Enum(s)
    }

    fn from_scalar(
        value: qitech_framework::__private::ScalarValue,
    ) -> Result<Self::Type, qitech_framework::__private::ScalarValueTypeMismatchError> {
        let qitech_framework::__private::ScalarValue::Enum(s) = value else {
            return Err(qitech_framework::__private::ScalarValueTypeMismatchError);
        };

        Ok(match s.as_str() {
            "not_homed" => State::NotHomed,
            "idle" => State::Idle,
            "going_in" => State::GoingIn,
            "going_out" => State::GoingOut,

            "initialize" => State::Homing(HomingState::Initialize),
            "escape_endstop" => State::Homing(HomingState::EscapeEndstop),
            "find_endstop_fine_distancing" => State::Homing(HomingState::FindEndstopFineDistancing),
            "find_endstop_coarse" => State::Homing(HomingState::FindEndstopCoarse),
            "find_endstop_fine" => State::Homing(HomingState::FindEndtopFine),

            "going_out (traverse)" => State::Traversing(TraversingState::GoingOut),
            "traversing_in" => State::Traversing(TraversingState::TraversingIn),
            "traversing_out" => State::Traversing(TraversingState::TraversingOut),

            _ => return Err(qitech_framework::__private::ScalarValueTypeMismatchError),
        })
    }

    fn validate_scalar_property_definition(
        definition: &qitech_framework::__private::ScalarPropertyDefinition,
        ignore_nullable: bool,
    ) -> bool {
        _ = definition;
        _ = ignore_nullable;
        true
    }

    fn validate_measurement_definition(
        definition: &qitech_framework::__private::MeasurementDefinition,
        ignore_nullable: bool,
    ) -> bool {
        _ = definition;
        _ = ignore_nullable;
        true
    }

    fn apply_constraints(
        constraints: &<Self::Type as qitech_framework::__private::PropertyType>::Constraints,
        value: &Self::Type,
    ) -> Result<(), qitech_framework::__private::ConstraintViolationError> {
        _ = constraints;
        _ = value;
        Ok(())
    }

    fn as_constraints(
        constraints: &<Self::Type as qitech_framework::__private::PropertyType>::Constraints,
    ) -> qitech_framework::__private::Constraints {
        _ = constraints;
        qitech_framework::__private::Constraints::Enum { allowed: vec![] }
    }
}

#[derive(Debug, Default, PartialEq, Eq, Clone, Copy)]
pub enum State {
    #[default]
    /// Initial state
    NotHomed,

    /// Doing nothing
    /// Already homed
    Idle,

    /// Going to inner limit
    ///
    /// After reaching the inner limit, the state will change to [`State::Idle`]
    GoingIn,

    /// Going to outer limit
    ///
    /// After reaching the outer limit, the state will change to [`State::Idle`]
    GoingOut,

    /// Homing is in progress
    ///
    /// After homing is done, the state will change to [`State::Idle`]
    Homing(HomingState),

    /// Move between inner and outer limits
    Traversing(TraversingState),
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum TraversingState {
    /// Like [`State::GoingOut`] but
    /// - will go into [`State::GoingIn`] after reaching the outer limit
    GoingOut,

    /// Like [`State::GoingIn`] but
    /// - will go into [`State::GoingOut`] after reaching the inner limit
    /// - speed is synced to spool speed
    TraversingIn,

    /// Like [`State::GoingOut`] but
    /// - will go into [`State::GoingIn`] after reaching the outer limit
    /// - speed is synced to spool speed
    TraversingOut,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum HomingState {
    /// In this state the traverse is not moving but checks if the endstop si triggered
    /// If the endstop is triggered we go into [`HomingState::EscapeEndstop`]
    /// If the endstop is not triggered we go into [`HomingState::FindEndstop`]
    Initialize,

    /// In this state the traverse is moving out away from the endstop until it's not triggered anymore
    /// The it goes into [`HomingState::FindEnstopFineDistancing`]
    EscapeEndstop,

    /// Moving out away from the endstop
    /// Then Transition into [`HomingState::FindEndtopFine`]
    FindEndstopFineDistancing,

    /// In this state the traverse is fast until it reaches the endstop
    FindEndstopCoarse,

    /// In this state the traverse is moving slowly until it reaches the endstop
    FindEndtopFine,

    /// In this state we check if th current position is actually 0.0, if not we redo the homing routine
    Validate(Instant),
}

impl Traverse {
    pub fn new(
        device: Rc<RefCell<dyn StepperVelocityEL70x1Device>>,
        limit_inner: ConfigProperty<Length>,
        limit_outer: ConfigProperty<Length>,
        step_size: ConfigProperty<Length>,
        padding: ConfigProperty<Length>,
        state: StateProperty<State>,
        position: Measurement<Option<Length>>,
        microsteps: u8,
    ) -> Self {
        Self {
            enabled: false,
            device,
            position,
            limit_inner,
            limit_outer,
            step_size,
            padding,
            state,
            fullstep_converter: LinearStepConverter::from_circumference(
                200,
                Length::new::<millimeter>(32.0),
            ),
            microstep_converter: LinearStepConverter::from_circumference(
                200 * microsteps as i16,
                Length::new::<millimeter>(32.0),
            ),
        }
    }
}

// Getter & Setter
impl Traverse {
    pub const fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
}

// State management
impl Traverse {
    pub fn goto_limit_inner(&mut self) {
        self.state.set(State::GoingIn);
    }

    pub fn goto_limit_outer(&mut self) -> ActResult {
        self.state.set(State::GoingOut);
        Ok(())
    }

    pub fn goto_home(&mut self) -> ActResult {
        self.state.set(State::Homing(HomingState::Initialize));
        Ok(())
    }

    pub fn start_traversing(&mut self) -> ActResult {
        self.state.set(State::Traversing(TraversingState::GoingOut));
        Ok(())
    }

    pub fn is_homed(&self) -> bool {
        // if not [`State::NotHomed`], then it is homed
        !matches!(self.state.get(), State::NotHomed)
    }

    pub fn is_going_in(&self) -> bool {
        // [`State::GoingIn`]
        matches!(self.state.get(), State::GoingIn)
    }

    pub fn is_going_out(&self) -> bool {
        // [`State::GoingOut`]
        matches!(self.state.get(), State::GoingOut)
    }

    pub fn is_going_home(&self) -> bool {
        // [`State::Homing`]
        matches!(self.state.get(), State::Homing(_))
    }

    pub fn is_traversing(&self) -> bool {
        // [`State::Traversing`]
        matches!(self.state.get(), State::Traversing(_))
    }
}

impl Traverse {
    // If at inner limit within a tolerance
    fn is_at_position(&self, target_position: Length, tolerance: Length) -> bool {
        let upper_tolerance = target_position + tolerance.abs();
        let lower_tolerance = target_position - tolerance.abs();
        self.position >= lower_tolerance && self.position <= upper_tolerance
    }

    /// Calculate distance to position
    fn distance_to_position(&self, target_position: Length) -> Length {
        if self.position > target_position {
            self.position - target_position
        } else if self.position < target_position {
            target_position - self.position
        } else {
            Length::ZERO
        }
    }

    // Changes the direction of the speed based on the current position and target position
    fn speed_to_position(&self, target_position: Length, absolute_speed: Velocity) -> Velocity {
        // If we are over the target position we need to move negative
        if self.position > target_position {
            -absolute_speed.abs()
        } else if self.position < target_position {
            absolute_speed.abs()
        } else {
            Velocity::ZERO
        }
    }

    /// Gets the current traverse position as a [`Length`].
    pub fn sync_position(&mut self, traverse: &dyn StepperVelocityEL70x1Device) {
        let steps = traverse.get_position(TRAVERSE_PORT);
        let pos = self.microstep_converter.steps_to_distance(steps as f64);
        self.position.set(Some(pos));
    }

    /// Calculates a desired speed based on the current state and the end stop status.
    ///
    /// Positive speed moved out, negative speed moves in.
    fn get_speed(
        &mut self,
        traverse: &mut dyn StepperVelocityEL70x1Device,
        spool_speed: AngularVelocity,
    ) -> Velocity {
        // Don't move if not enabled or in a state that doesn't result in movement
        if !self.enabled {
            return Velocity::ZERO;
        }

        self.sync_position(traverse);

        // Automatic Transitions
        match self.state.get() {
            State::NotHomed => {}
            State::Idle => {}
            State::GoingIn => {
                // If inner limit is reached
                if self.is_at_position(self.limit_inner.get(), Length::new::<millimeter>(0.01)) {
                    // Put Into Idle
                    self.state.set(State::Idle);
                }
            }
            State::GoingOut => {
                // If outer limit is reached
                if self.is_at_position(self.limit_outer.get(), Length::new::<millimeter>(0.01)) {
                    // Put Into Idle
                    self.state.set(State::Idle);
                }
            }
            State::Homing(homing_state) => match homing_state {
                HomingState::Initialize => {
                    // If endstop is triggered, escape the endstop
                    if traverse
                        .get_digital_input(TRAVERSE_END_STOP_PORT)
                        .unwrap_or(false)
                    {
                        self.state.set(State::Homing(HomingState::EscapeEndstop));
                    } else {
                        // If endstop is not triggered, move to the endstop
                        self.state
                            .set(State::Homing(HomingState::FindEndstopCoarse));
                    }
                }
                HomingState::EscapeEndstop => {
                    // Move out until endstop is not triggered anymore
                    if !traverse
                        .get_digital_input(TRAVERSE_END_STOP_PORT)
                        .unwrap_or(false)
                    {
                        self.state
                            .set(State::Homing(HomingState::FindEndstopFineDistancing));
                    }
                }
                HomingState::FindEndstopFineDistancing => {
                    // Move out until endstop is not triggered anymore
                    if !traverse
                        .get_digital_input(TRAVERSE_END_STOP_PORT)
                        .unwrap_or(false)
                    {
                        // Find endstop fine
                        self.state.set(State::Homing(HomingState::FindEndtopFine));
                    }
                }
                HomingState::FindEndtopFine => {
                    // If endstop is reached change to idle
                    if traverse
                        .get_digital_input(TRAVERSE_END_STOP_PORT)
                        .unwrap_or(false)
                    {
                        // Set poition of traverse to 0
                        traverse.set_position(TRAVERSE_PORT, 0);
                        // Put Into Idle
                        self.state
                            .set(State::Homing(HomingState::Validate(Instant::now())));
                    }
                }
                HomingState::FindEndstopCoarse => {
                    // Move to endstop
                    if traverse
                        .get_digital_input(TRAVERSE_END_STOP_PORT)
                        .unwrap_or(false)
                    {
                        // Move awaiy from endstop
                        self.state
                            .set(State::Homing(HomingState::FindEndstopFineDistancing));
                    }
                }
                HomingState::Validate(instant) => {
                    // If 100ms have passed check if position is actually 0.0
                    if instant.elapsed().as_millis() > 100 {
                        if self.is_at_position(Length::ZERO, Length::new::<millimeter>(0.01)) {
                            // If position is 0.0, put into idle
                            self.state.set(State::Idle);
                        } else {
                            // If position is not 0.0, redo homing
                            self.state.set(State::Homing(HomingState::Initialize));
                        }
                    }
                }
            },

            // If state changed we
            State::Traversing(traversing_state) => match traversing_state {
                TraversingState::GoingOut => {
                    // If outer limit is reached
                    if self.position >= self.limit_outer.get() - self.padding.get() {
                        // Turn around
                        self.state
                            .set(State::Traversing(TraversingState::TraversingIn));
                    }
                }
                TraversingState::TraversingIn => {
                    // If inner limit is reached
                    if self.position <= self.limit_inner.get() + self.padding.get() {
                        // Turn around
                        self.state
                            .set(State::Traversing(TraversingState::TraversingOut));
                    }
                }
                TraversingState::TraversingOut => {
                    // If outer limit is reached
                    if self.position >= self.limit_outer.get() - self.padding.get() {
                        // Turn around
                        self.state
                            .set(State::Traversing(TraversingState::TraversingIn));
                    }
                }
            },
        }

        // Speed
        match self.state.get() {
            State::NotHomed => Velocity::ZERO, // Not homed, no movement
            State::Idle => Velocity::ZERO,     // No movement in idle state
            State::GoingIn => {
                // Move in at a speed of 10-100 mm/s
                self.speed_to_position(
                    self.limit_inner.get(),
                    match self.distance_to_position(self.limit_inner.get()).abs()
                        > Length::new::<millimeter>(1.0)
                    {
                        true => Velocity::new::<millimeter_per_second>(100.0),
                        false => Velocity::new::<millimeter_per_second>(10.0),
                    },
                )
            }
            State::GoingOut => {
                // Move out at a speed of 10-100 mm/s
                self.speed_to_position(
                    self.limit_outer.get(),
                    match self.distance_to_position(self.limit_outer.get()).abs()
                        > Length::new::<millimeter>(1.0)
                    {
                        true => Velocity::new::<millimeter_per_second>(100.0),
                        false => Velocity::new::<millimeter_per_second>(10.0),
                    },
                )
            }
            State::Homing(homing_state) => match homing_state {
                HomingState::Initialize => Velocity::ZERO,
                HomingState::EscapeEndstop => {
                    // Move out at a speed of 10 mm/s
                    Velocity::new::<millimeter_per_second>(10.0)
                }
                HomingState::FindEndstopFineDistancing => {
                    // Move out at a speed of 2 mm/s
                    Velocity::new::<millimeter_per_second>(2.0)
                }
                HomingState::FindEndstopCoarse => {
                    // Move in at a speed of -100 mm/s
                    Velocity::new::<millimeter_per_second>(-100.0)
                }
                HomingState::FindEndtopFine => {
                    // move into the endstop at 2 mm/s
                    Velocity::new::<millimeter_per_second>(-2.0)
                }
                HomingState::Validate(_) => {
                    // We stand still until the validation cooldown has passed
                    Velocity::ZERO
                }
            }, // Homing speed
            State::Traversing(traversing_state) => match traversing_state {
                TraversingState::GoingOut => {
                    // Move out at a speed of 100 mm/s
                    self.speed_to_position(
                        self.limit_outer.get() - self.padding.get()
                            + Length::new::<millimeter>(0.01),
                        Velocity::new::<millimeter_per_second>(100.0),
                    )
                }
                TraversingState::TraversingIn => self.speed_to_position(
                    self.limit_inner.get() + self.padding.get() - Length::new::<millimeter>(0.01),
                    Self::calculate_traverse_speed(spool_speed, self.step_size.get()),
                ),
                TraversingState::TraversingOut => self.speed_to_position(
                    self.limit_outer.get() - self.padding.get() + Length::new::<millimeter>(0.01),
                    Self::calculate_traverse_speed(spool_speed, self.step_size.get()),
                ),
            },
        }
    }

    /// Calculate the traverse speed
    ///
    /// The traverse speed is the linear speed at which the winding mechanism moves along the spool.
    /// It's directly proportional to how fast the spool rotates and how far the traverse moves per rotation.
    ///
    /// - Traverse Distance per Revolution [mm] = Step Size [mm]
    /// - Traverse Speed [mm/s] = Spool Speed [rev/s or rad/s] * Step Size [mm]
    ///
    /// Note: While the traverse range (from outer limit minus padding to inner limit plus padding)
    /// determines the total area to be covered, the traverse speed itself depends only on
    /// the step size and spool rotation speed.
    pub fn calculate_traverse_speed(spool_speed: AngularVelocity, step_size: Length) -> Velocity {
        // Calculate the traverse speed directly from spool speed and step size
        let traverse_speed: Velocity = Velocity::new::<millimeter_per_second>(
            spool_speed.get::<revolution_per_second>() * step_size.get::<millimeter>(),
        );

        traverse_speed
    }

    pub fn update_speed(
        &mut self,
        traverse: &mut dyn StepperVelocityEL70x1Device,
        spool_speed: AngularVelocity,
    ) {
        let speed = self.get_speed(traverse, spool_speed);
        let steps_per_second = self.fullstep_converter.velocity_to_steps(speed);
        // ignore if we can't set speed
        let _ = traverse.set_speed(TRAVERSE_PORT, steps_per_second);
    }
}
