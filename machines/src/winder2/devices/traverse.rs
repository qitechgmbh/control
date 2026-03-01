use std::time::Instant;

use control_core::converters::linear_step_converter::LinearStepConverter;
use ethercat_hal::io::digital_input::DigitalInput;
use ethercat_hal::io::digital_output::DigitalOutput;
use ethercat_hal::io::stepper_velocity_el70x1::StepperVelocityEL70x1;
use units::AngularVelocity;
use units::ConstZero;
use units::Length;
use units::Velocity;
use units::angular_velocity::revolution_per_second;
use units::length::millimeter;
use units::velocity::millimeter_per_second;

use crate::winder2::devices::Spool;

use super::OperationState;

/// Represents the puller motor
#[derive(Debug)]
pub struct Traverse
{
    // hardware
    motor: StepperVelocityEL70x1,
    laser_pointer: DigitalOutput,
    limit_switch: DigitalInput,

    // config
    operation_state: OperationState,
    position:     Length,
    limit_inner:  Length,
    limit_outer:  Length,
    step_size:    Length,
    padding:      Length,
    state:        State,

    // converters
    fullstep_converter:  LinearStepConverter,
    microstep_converter: LinearStepConverter,
}

// constants
impl Traverse
{
    const DEFAULT_LIMIT_INNER:  f64 = 22.0; // in mm
    const DEFAULT_LIMIT_OUTER:  f64 = 92.0; // in mm
    const DEFAULT_PADDING:      f64 = 0.88; // in mm
    const DEFAULT_STEP_SIZE:    f64 = 1.75; // in mm
    const CIRCUMFERENCE:        f64 = 35.0; // in mm
    const STEPS_PER_REVOLUTION: i16 = 200;
    const MICRO_STEPS_COUNT:    i16 = 64;
}

// public interface
impl Traverse
{
    pub fn new(
        motor: StepperVelocityEL70x1, 
        limit_switch: DigitalInput,
        laser_pointer: DigitalOutput,
    ) -> Self
    {
        let limit_inner   = Length::new::<millimeter>(Self::DEFAULT_LIMIT_INNER);
        let limit_outer   = Length::new::<millimeter>(Self::DEFAULT_LIMIT_OUTER);
        let circumference = Length::new::<millimeter>(Self::CIRCUMFERENCE);

        let fullstep_converter = LinearStepConverter::from_circumference(
            Self::STEPS_PER_REVOLUTION,
            circumference,
        );

        let microstep_converter = LinearStepConverter::from_circumference(
            Self::STEPS_PER_REVOLUTION * Self::MICRO_STEPS_COUNT,
            circumference,
        );

        let step_size = Length::new::<millimeter>(Self::DEFAULT_STEP_SIZE);
        let padding   = Length::new::<millimeter>(Self::DEFAULT_PADDING);

        Self {
            operation_state: OperationState::Disabled,
            motor,
            laser_pointer,
            limit_switch,
            fullstep_converter,
            microstep_converter,
            limit_inner,
            limit_outer,
            step_size, 
            padding,
            position: Length::ZERO,
            state: State::NotHomed,
        }
    }

    /// update the traverse. Returns true if the state
    /// changed from this update
    pub fn update(&mut self, spool: &Spool)
    {
        if self.operation_state == OperationState::Disabled { return; }

        self.update_position();
        self.update_state();

        let steps_per_second = self.compute_output_steps(spool.speed());
        _ = self.motor.set_speed(steps_per_second);
    }
}

// getters + setters
impl Traverse
{
    pub fn set_operation_state(&mut self, device_state: OperationState)
    {
        use OperationState::*;

        // No change, nothing to do
        if self.operation_state == device_state { return; }

        // Leaving standby, enable motor
        if self.operation_state == Disabled
        {
            self.motor.set_enabled(true);
        }

        match device_state // guranteed state change
        {
            Disabled => self.motor.set_enabled(false),
            Holding  => self.goto_home(),
            Running  => self.start_traversing(),
        }

        self.operation_state = device_state;
    }

    pub fn state(&self) -> State
    {
        self.state
    }

    pub fn limit_inner(&self) -> Length { self.limit_inner }

    pub fn try_set_limit_inner(&mut self, limit_inner: Length) -> Result<(), ()>
    { 
        // Validate the new inner limit against current outer limit
        if !Self::validate_traverse_limits(limit_inner, self.limit_outer)
        {
            // Don't update if validation fails - keep the current value
            return Err(());
        }

        self.limit_inner = limit_inner;
        Ok(())
    }

    pub fn limit_outer(&self) -> Length { self.limit_outer }
    pub fn try_set_limit_outer(&mut self, limit_outer: Length) -> Result<(), ()>
    { 
        // Validate the new outer limit against current inner limit
        if !Self::validate_traverse_limits(self.limit_inner, limit_outer)
        {
            // Don't update if validation fails - keep the current value
            return Err(());
        }

        self.limit_outer = limit_outer; 
        Ok(())
    }

    pub fn step_size(&self) -> Length { self.step_size }
    pub fn set_step_size(&mut self, step_size: Length) { self.step_size = step_size; }

    pub fn padding(&self) -> Length { self.padding }
    pub fn set_padding(&mut self, padding: Length) { self.padding = padding; }

    pub fn current_position(&self) -> Option<Length> 
    {
        match self.is_homed() 
        {
            true  => Some(self.position),
            false => None,
        }
    }

    pub fn laser_pointer_enabled(&self) -> bool
    {
        self.laser_pointer.get()
    }

    pub fn set_laser_pointer_enabled(&mut self, value: bool)
    {
        self.laser_pointer.set(value);
    }
}

// State management
impl Traverse 
{
    pub fn can_goto_limit_inner(&self) -> bool
    {
        self.operation_state == OperationState::Disabled
            || !self.is_homed() 
            || self.is_going_in() 
            || self.is_going_home()
            || self.is_traversing()
    }

    pub fn try_goto_limit_inner(&mut self) -> Result<(), ()>
    {
        if !self.can_goto_limit_inner()
        {
            return Err(());
        }

        self.state = State::GoingIn;
        Ok(())
    }

    pub fn can_goto_limit_outer(&self)-> bool
    {
        self.operation_state == OperationState::Disabled
            || !self.is_homed() 
            || self.is_going_out() 
            || self.is_going_home()
            || self.is_traversing()
    }

    pub fn try_goto_limit_outer(&mut self) -> Result<(), ()>
    {
        if !self.can_goto_limit_outer()
        {
            return Err(());
        }

        self.state = State::GoingOut;
        Ok(())
    }

    pub fn can_go_home(&self)-> bool
    {
        self.operation_state == OperationState::Disabled
            || !self.is_homed() 
            || self.is_going_out() 
            || self.is_going_home()
            || self.is_traversing()
    }

    pub fn try_goto_home(&mut self) -> Result<(), ()>
    {
        if !self.can_go_home()
        {
            return Err(());
        }

        self.goto_home();
        Ok(())
    }

    fn goto_home(&mut self)
    {
        self.state = State::Homing(HomingState::Initialize);
    }

    fn start_traversing(&mut self) 
    {
        self.state = State::Traversing(TraversingState::GoingOut);
    }

    #[allow(dead_code)]
    pub fn is_idle(&self) -> bool 
    {
        self.state == State::Idle
    }

    pub fn is_homed(&self) -> bool {
        // if not [`State::NotHomed`], then it is homed
        !matches!(self.state, State::NotHomed)
    }

    pub fn is_going_in(&self) -> bool {
        self.state == State::GoingIn
    }

    pub fn is_going_out(&self) -> bool {
        self.state == State::GoingOut
    }

    pub fn is_going_home(&self) -> bool {
        matches!(self.state, State::Homing(_))
    }

    pub fn is_traversing(&self) -> bool {
        matches!(self.state, State::Traversing(_))
    }
}

// helpers
impl Traverse 
{
    fn compute_output_steps(&self, spool_speed: AngularVelocity) -> f64
    {
        let speed = self.veloctity_from_state(spool_speed);
        self.fullstep_converter.velocity_to_steps(speed)
    }

    pub fn update_position(&mut self) 
    {
        let steps = self.motor.get_position() as f64;
        self.position = self.microstep_converter.steps_to_distance(steps);
    }

    fn endstop_triggered(&self) -> bool
    {
        self.limit_switch.get_value().unwrap_or(false)
    }

    fn calculate_traverse_speed(spool_speed: AngularVelocity, step_size: Length) -> Velocity 
    {
        let spool_speed = spool_speed.get::<revolution_per_second>();
        let step_size   = step_size.get::<millimeter>();

        // Calculate the traverse speed directly from spool speed and step size
        Velocity::new::<millimeter_per_second>(spool_speed * step_size)
    }

    // Changes the direction of the speed based on the current position and target position
    fn speed_to_position(&self, target_position: Length, absolute_speed: Velocity) -> Velocity 
    {
        // If we are over the target position we need to move negative
        if self.position > target_position {
            -absolute_speed.abs()
        } else if self.position < target_position {
            absolute_speed.abs()
        } else {
            Velocity::ZERO
        }
    }

    /// Calculate distance to position
    fn distance_to_position(&self, target_position: Length) -> Length 
    {
        (self.position - target_position).abs()
    }

    // If at inner limit within a tolerance
    fn is_at_position(&self, target_position: Length, tolerance: Length) -> bool 
    {
        let upper_tolerance = target_position + tolerance.abs();
        let lower_tolerance = target_position - tolerance.abs();
        lower_tolerance <= self.position && self.position <= upper_tolerance
    }

    /// Validates that traverse limits maintain proper constraints:
    /// - Inner limit must be smaller than outer limit
    /// - At least 0.9mm difference between inner and outer limits
    fn validate_traverse_limits(inner: Length, outer: Length) -> bool 
    {
        outer > inner + Length::new::<millimeter>(0.9)
    }
}

// state updates
impl Traverse
{
    fn update_state(&mut self)
    {
        use State::*;

        match self.state
        {
            NotHomed | Idle => {}
            GoingIn => 
            {
                // If inner limit is reached
                if self.is_at_position(self.limit_inner, Length::new::<millimeter>(0.01)) 
                {
                    // Put Into Idle
                    self.state = State::Idle;
                }
            }
            GoingOut => 
            {
                // If outer limit is reached
                if self.is_at_position(self.limit_outer, Length::new::<millimeter>(0.01)) {
                    // Put Into Idle
                    self.state = State::Idle;
                }
            }
            Homing(state) => self.update_homing_state(state),
            Traversing(state) => self.update_traversing_state(state),
        };
    }

    fn update_homing_state(&mut self, homing_state: HomingState)
    {
        use HomingState::*;

        match homing_state
        {
            Initialize => 
            {
                // If endstop is triggered, escape the endstop
                if self.endstop_triggered() {
                    self.state = State::Homing(EscapeEndstop);
                } else {
                    // If endstop is not triggered, move to the endstop
                    self.state = State::Homing(FindEndstopCoarse);
                }
            }
            EscapeEndstop => {
                // Move out until endstop is not triggered anymore
                if !self.endstop_triggered() {
                    self.state = State::Homing(FindEndstopFineDistancing);
                }
            }
            FindEndstopFineDistancing => {
                // Move out until endstop is not triggered anymore
                if !self.endstop_triggered() {
                    // Find endstop fine
                    self.state = State::Homing(FindEndtopFine);
                }
            }
            FindEndtopFine => {
                // If endstop is reached change to idle
                if self.endstop_triggered() {
                    // Set poition of traverse to 0
                    self.motor.set_position(0);
                    // Put Into Idle
                    self.state = State::Homing(Validate(Instant::now()));
                }
            }
            FindEndstopCoarse => {
                // Move to endstop
                if self.endstop_triggered() {
                    // Move awaiy from endstop
                    self.state = State::Homing(FindEndstopFineDistancing);
                }
            }
            Validate(instant) => {
                // If 100ms have passed check if position is actually 0.0
                if instant.elapsed().as_millis() > 100 {
                    if self.is_at_position(Length::ZERO, Length::new::<millimeter>(0.01)) {
                        // If position is 0.0, put into idle
                        self.state = State::Idle;
                    } else {
                        // If position is not 0.0, redo homing
                        self.state = State::Homing(Initialize);
                    }
                }
            }
        }
    }

    fn update_traversing_state(&mut self, state: TraversingState)
    {
        use TraversingState::*;

        match state {
            GoingOut => {
                // If outer limit is reached
                if self.position >= self.limit_outer - self.padding {
                    // Turn around
                    self.state = State::Traversing(TraversingIn);
                }
            }
            TraversingIn => {
                // If inner limit is reached
                if self.position <= self.limit_inner + self.padding {
                    // Turn around
                    self.state = State::Traversing(TraversingOut);
                }
            }
            TraversingOut => {
                // If outer limit is reached
                if self.position >= self.limit_outer - self.padding {
                    // Turn around
                    self.state = State::Traversing(TraversingIn);
                }
            }
        }
    }
}

// velocity computation
impl Traverse 
{
    fn veloctity_from_state(&self, spool_speed: AngularVelocity) -> Velocity
    {
        use State::*;

        match self.state {
            NotHomed => Velocity::ZERO, // Not homed, no movement
            Idle => Velocity::ZERO,     // No movement in idle state
            GoingIn => {
                // Move in at a speed of 10-100 mm/s
                self.speed_to_position(
                    self.limit_inner,
                    match self.distance_to_position(self.limit_inner).abs()
                        > Length::new::<millimeter>(1.0)
                    {
                        true => Velocity::new::<millimeter_per_second>(100.0),
                        false => Velocity::new::<millimeter_per_second>(10.0),
                    },
                )
            },
            GoingOut => {
                // Move out at a speed of 10-100 mm/s
                self.speed_to_position(
                    self.limit_outer,
                    match self.distance_to_position(self.limit_outer).abs()
                        > Length::new::<millimeter>(1.0)
                    {
                        true => Velocity::new::<millimeter_per_second>(100.0),
                        false => Velocity::new::<millimeter_per_second>(10.0),
                    },
                )
            }
            Homing(state) => Self::velocity_from_homing_state(state),
            Traversing(state) => self.velocity_from_traversing_state(state, spool_speed),
        }
    }

    fn velocity_from_homing_state(homing_state: HomingState) -> Velocity
    {
        use units::velocity::millimeter_per_second as mmps;
        use HomingState::*;

        match homing_state {
            Initialize => Velocity::ZERO,
            // Move out at a speed of 10 mm/s
            EscapeEndstop => Velocity::new::<mmps>(10.0),
            // Move out at a speed of 2 mm/s
            FindEndstopFineDistancing => Velocity::new::<mmps>(2.0),
            // Move in at a speed of -100 mm/s
            FindEndstopCoarse => Velocity::new::<mmps>(-100.0),
            // move into the endstop at 2 mm/s
            FindEndtopFine => Velocity::new::<mmps>(-2.0),
            // We stand still until the validation cooldown has passed
            Validate(_) => Velocity::ZERO,
        }
    }

    fn velocity_from_traversing_state(
        &self, 
        traversing_state: TraversingState, 
        spool_speed: AngularVelocity
    ) -> Velocity
    {
        use TraversingState::*;

        match traversing_state {
            GoingOut => {
                // Move out at a speed of 100 mm/s
                self.speed_to_position(
                    self.limit_outer - self.padding + Length::new::<millimeter>(0.01),
                    Velocity::new::<millimeter_per_second>(100.0),
                )
            }
            TraversingIn => 
            {
                let offset = Length::new::<millimeter>(0.01);
                let target_position = self.limit_inner + self.padding - offset;
                let absolute_speed = Self::calculate_traverse_speed(spool_speed, self.step_size);
                self.speed_to_position(target_position, absolute_speed)
            }
            TraversingOut => 
            {
                let offset = Length::new::<millimeter>(0.01);
                let target_position = self.limit_outer - self.padding + offset;
                let absolute_speed = Self::calculate_traverse_speed(spool_speed, self.step_size);
                self.speed_to_position(target_position, absolute_speed)
            }
        }
    }
}

// other types
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum State 
{
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
pub enum TraversingState 
{
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
pub enum HomingState 
{
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