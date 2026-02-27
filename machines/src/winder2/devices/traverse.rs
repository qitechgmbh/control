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

/// Represents the puller motor
#[derive(Debug)]
pub struct Traverse
{
    motor: StepperVelocityEL70x1,
    laser: DigitalOutput,
    limit_switch: DigitalInput,

    mode:        Mode,
    position:    Length,
    limit_inner: Length,
    limit_outer: Length,
    step_size:   Length,
    padding:     Length,
    state:       State,
    fullstep_converter:  LinearStepConverter,
    microstep_converter: LinearStepConverter,
    // A sticky flag if the [`State`] changed (not the sub states)
    // Needed to send state updates to the UI
    did_change_state: bool,
}

// public interface
impl Traverse
{
    pub const DEFAULT_PADDING: f64 = 0.88;
    pub const DEFAULT_STEP_SIZE: f64 = 1.75;

    pub fn new(
        motor: StepperVelocityEL70x1, 
        laser: DigitalOutput,
        limit_switch: DigitalInput,
    ) -> Self
    {
        let limit_inner = Length::new::<millimeter>(22.0);
        let limit_outer = Length::new::<millimeter>(92.0);
        let microsteps  = 64 as u8;

        let steps_per_revolution = 200;
        let circumference = Length::new::<millimeter>(35.0);

        let fullstep_converter = LinearStepConverter::from_circumference(
            steps_per_revolution,
            circumference,
        );

        let microstep_converter = LinearStepConverter::from_circumference(
            steps_per_revolution * microsteps as i16,
            circumference,
        );

        let step_size = Length::new::<millimeter>(Self::DEFAULT_STEP_SIZE);
        let padding   = Length::new::<millimeter>(Self::DEFAULT_PADDING);

        Self {
            mode: Mode::Standby,
            motor,
            laser,
            limit_switch,
            fullstep_converter,
            microstep_converter,
            limit_inner,
            limit_outer,
            step_size, 
            padding,
            position: Length::ZERO,
            state: State::NotHomed,
            did_change_state: false,
        }
    }

    pub fn update(&mut self, spool: &Spool)
    {
        if self.mode == Mode::Standby { return; }

        self.update_position();
        self.update_state();

        let steps_per_second = self.compute_output_steps(spool.speed());
        // ignoring error is probably not ideal but well I don't code this...
        let _ = self.motor.set_speed(steps_per_second);
    }
}

// getters + setters
impl Traverse
{
    pub const fn mode(&self) -> Mode { self.mode }

    pub fn set_mode(&mut self, mode: Mode)
    {
        use Mode::*;

        // No change, nothing to do
        if self.mode == mode { return; }

        // Leaving standby, enable motor
        if self.mode == Standby
        {
            self.motor.set_enabled(true);
        }

        match mode // guranteed state change
        {
            Standby  => self.motor.set_enabled(false),
            Hold     => self.goto_home(),
            Traverse => self.start_traversing(),
        }

        self.mode = mode;
    }

    pub fn limit_inner(&self) -> Length { self.limit_inner }
    pub fn set_limit_inner(&mut self, value: Length) { self.limit_inner = value; }

    pub fn limit_outer(&self) -> Length { self.limit_outer }
    pub fn set_limit_outer(&mut self, value: Length) { self.limit_outer = value; }

    pub fn step_size(&self) -> Length { self.step_size }
    pub fn set_step_size(&mut self, value: Length) { self.step_size = value; }

    pub fn padding(&self) -> Length { self.padding }
    pub fn set_padding(&mut self, value: Length) { self.padding = value; }

    pub fn current_position(&self) -> Option<Length> 
    {
        match self.is_homed() 
        {
            true  => Some(self.position),
            false => None,
        }
    }

    pub const fn consume_state_changed(&mut self) -> bool 
    {
        let did_change = self.did_change_state;
        // Reset the flag
        self.did_change_state = false;
        did_change
    }

    pub fn laser_pointer_enabled(&self) -> bool
    {
        self.laser.get()
    }

    pub fn set_laser_pointer_enabled(&mut self, value: bool)
    {
        self.laser.set(value);
    }
}

// State management
impl Traverse 
{
    pub fn can_goto_limit_inner(&self) -> bool
    {
        self.mode == Mode::Standby
            || !self.is_homed() 
            || self.is_going_in() 
            || self.is_going_home()
            || self.is_traversing()
    }

    pub fn goto_limit_inner(&mut self) -> Result<(), ()>
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
        self.mode == Mode::Standby
            || !self.is_homed() 
            || self.is_going_out() 
            || self.is_going_home()
            || self.is_traversing()
    }

    pub fn goto_limit_outer(&mut self) -> Result<(), ()>
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
        self.mode == Mode::Standby
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

    pub fn is_standby(&self) -> bool {
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
        return self.limit_switch.get_value().unwrap_or(false);
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
}

// state updates
impl Traverse
{
    fn update_state(&mut self)
    {
        use State::*;

        let old_state = self.state.clone();

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

        // update flag of state changed
        self.did_change_state = self.state != old_state;
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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Standby,
    Hold,
    Traverse,
}

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