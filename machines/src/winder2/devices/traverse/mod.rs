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

use crate::winder2::devices::{Spool, OperationState};

mod types;
pub use types::State;
pub use types::HomingState;
pub use types::TraversingState;

#[derive(Debug)]
pub struct Traverse
{
    // hardware
    motor: StepperVelocityEL70x1,
    laser_pointer: DigitalOutput,
    limit_switch: DigitalInput,

    // config
    operation_state: OperationState,
    state:           State,
    position:        Length,
    limit_inner:     Length,
    limit_outer:     Length,
    step_size:       Length,
    padding:         Length,

    // converters
    fullstep_converter:  LinearStepConverter,
    microstep_converter: LinearStepConverter,
}

// constants
impl Traverse
{
    // physical properties
    const CIRCUMFERENCE:        f64 = 35.0; // in mm
    const STEPS_PER_REVOLUTION: i16 = 200;
    const MICRO_STEPS_COUNT:    i16 = 64;

    // virtual properties
    const LENGTH_TOLERANCE:     f64 = 0.01;

    // defaults
    const DEFAULT_LIMIT_INNER:  f64 = 22.0; // in mm
    const DEFAULT_LIMIT_OUTER:  f64 = 92.0; // in mm
    const DEFAULT_PADDING:      f64 = 0.88; // in mm
    const DEFAULT_STEP_SIZE:    f64 = 1.75; // in mm
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
        use millimeter as mm;

        let limit_inner   = Length::new::<mm>(Self::DEFAULT_LIMIT_INNER);
        let limit_outer   = Length::new::<mm>(Self::DEFAULT_LIMIT_OUTER);
        let circumference = Length::new::<mm>(Self::CIRCUMFERENCE);

        let fullstep_converter = LinearStepConverter::from_circumference(
            Self::STEPS_PER_REVOLUTION,
            circumference,
        );

        let microstep_converter = LinearStepConverter::from_circumference(
            Self::STEPS_PER_REVOLUTION * Self::MICRO_STEPS_COUNT,
            circumference,
        );

        let step_size = Length::new::<mm>(Self::DEFAULT_STEP_SIZE);
        let padding   = Length::new::<mm>(Self::DEFAULT_PADDING);

        Self {
            operation_state: OperationState::Disabled,
            state:           State::NotHomed,
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
        }
    }

    pub fn update(&mut self, spool: &Spool)
    {
        if self.operation_state == OperationState::Disabled { return; }

        self.update_position();
        self.update_state();
        self.update_speed(spool.speed());
    }
}

// getters + setters
impl Traverse
{
    pub fn set_operation_state(&mut self, operation_state: OperationState)
    {
        use OperationState::*;

        // No change, nothing to do
        if self.operation_state == operation_state { return; }

        // Leaving standby, enable motor
        if self.operation_state == Disabled
        {
            self.motor.set_enabled(true);
        }

        match operation_state // guranteed state change
        {
            Disabled => self.motor.set_enabled(false),
            Holding  => self.goto_home(),
            Running  => self.start_traversing(),
        }

        self.operation_state = operation_state;
    }

    pub fn state(&self) -> State
    {
        self.state
    }

    pub fn limit_inner(&self) -> Length 
    { 
        self.limit_inner 
    }

    pub fn try_set_limit_inner(&mut self, limit_inner: Length) -> Result<(), ()>
    { 
        // validate the new inner limit against current outer limit
        if !Self::validate_traverse_limits(limit_inner, self.limit_outer)
        {
            // don't update if validation fails - keep the current value
            return Err(());
        }

        self.limit_inner = limit_inner;
        Ok(())
    }

    pub fn limit_outer(&self) -> Length 
    { 
        self.limit_outer 
    }

    pub fn try_set_limit_outer(&mut self, limit_outer: Length) -> Result<(), ()>
    { 
        // validate the new outer limit against current inner limit
        if !Self::validate_traverse_limits(self.limit_inner, limit_outer)
        {
            // don't update if validation fails - keep the current value
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
        match self.state.is_homed() 
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
            || !self.state.is_homed() 
            || self.state.is_going_in() 
            || self.state.is_going_home()
            || self.state.is_traversing()
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
        let state = self.state;
        self.operation_state == OperationState::Disabled
            || !state.is_homed() 
            || state.is_going_out() 
            || state.is_going_home()
            || state.is_traversing()
    }

    pub fn try_goto_limit_outer(&mut self) -> Result<(), ()>
    {
        if !self.can_goto_limit_outer() {
            return Err(());
        }

        self.state = State::GoingOut;
        Ok(())
    }

    pub fn can_go_home(&self)-> bool
    {
        let state = self.state;
        self.operation_state == OperationState::Disabled
            || state.is_going_out() 
            || state.is_going_home()
            || state.is_traversing()
    }

    pub fn try_goto_home(&mut self) -> Result<(), ()>
    {
        if !self.can_go_home() {
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
}

// helpers
impl Traverse 
{
    fn update_position(&mut self)
    {
        let steps = self.motor.get_position() as f64;
        self.position = self.microstep_converter.steps_to_distance(steps);
    }

    fn update_state(&mut self)
    {
        if let Some(next_state) = self.next_state()
        {
            self.state = next_state;
        }
    }

    fn update_speed(&mut self, spool_speed: AngularVelocity)
    {
        let steps_per_second = self.compute_output_steps(spool_speed);
        _ = self.motor.set_speed(steps_per_second);
    }

    fn compute_output_steps(&self, spool_speed: AngularVelocity) -> f64
    {
        let speed = self.veloctity_from_state(spool_speed);
        self.fullstep_converter.velocity_to_steps(speed)
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

    fn is_at_position(&self, target_position: Length) -> bool 
    {
        let tolerance = Length::new::<millimeter>(Self::LENGTH_TOLERANCE);
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

    fn is_close_to_target(&self, target: Length) -> bool 
    {
        self.distance_to_position(target).abs() <= Length::new::<millimeter>(1.0)
    }
}

// state updates
impl Traverse
{
    fn next_state(&mut self) -> Option<State>
    {
        use State::*;

        match self.state
        {
            NotHomed | Idle => None,
            GoingIn => 
            {
                // wait until we reach the inner limit
                if !self.is_at_position(self.limit_inner) {
                    return None;
                }

                Some(State::Idle)
            }
            GoingOut => 
            {
                // wait until we reach the outer limit
                if !self.is_at_position(self.limit_outer) {
                    return None;
                }

                Some(State::Idle)
            }
            Homing(state) => self.next_state_from_homing_state(state),
            Traversing(state) => self.next_state_from_traversing_state(state),
        }
    }

    fn next_state_from_homing_state(&mut self, homing_state: HomingState) -> Option<State>
    {
        use HomingState::*;

        match homing_state
        {
            Initialize => 
            {
                if self.endstop_triggered() {
                    // If endstop is triggered, escape the endstop
                    Some(State::Homing(EscapeEndstop))
                } else {
                    // If endstop is not triggered, move to the endstop
                    Some(State::Homing(FindEndstopCoarse))
                }
            }
            EscapeEndstop => 
            {
                // move out until endstop is not triggered anymore
                if self.endstop_triggered() { return None; }

                // now start finding
                Some(State::Homing(FindEndstopFineDistancing))
            }
            FindEndstopFineDistancing => 
            {
                // move out until endstop is not triggered anymore
                if self.endstop_triggered() { return None; }

                Some(State::Homing(FindEndstopFine))
            }
            FindEndstopFine => 
            {
                // move to endstop
                if !self.endstop_triggered() { return None; }

                // set poition of traverse to 0
                self.motor.set_position(0);

                // now validate
                Some(State::Homing(Validate(Instant::now())))
            }
            FindEndstopCoarse => 
            {
                // move to endstop
                if !self.endstop_triggered() { return None; }

                // now move away from endstop
                Some(State::Homing(FindEndstopFineDistancing))
            }
            Validate(instant) => 
            {
                const VALIDATION_DELAY_MS: u128 = 100;

                if instant.elapsed().as_millis() <= VALIDATION_DELAY_MS {
                    return None;
                }

                // should be at zero now
                if self.is_at_position(Length::ZERO)
                    { Some(State::Idle) }
                else // validation failed. retry
                    { Some(State::Homing(Initialize)) }
            }
        }
    }

    fn next_state_from_traversing_state(&mut self, state: TraversingState) -> Option<State>
    {
        use TraversingState::*;

        match state 
        {
            TraversingIn => 
            {
                // inner limit not reached yet
                if self.position > self.limit_inner + self.padding {
                    return None;
                }

                // now traverse to out
                Some(State::Traversing(TraversingOut))
            }
            GoingOut | TraversingOut => 
            {
                // outer limit not reached yet
                if self.position < self.limit_outer - self.padding {
                    return None;
                }

                // now traverse to in
                Some(State::Traversing(TraversingIn))
            }
        }
    }
}

// velocity computation
impl Traverse 
{
    fn veloctity_from_state(&self, spool_speed: AngularVelocity) -> Velocity
    {
        use millimeter_per_second as mmps;
        use State::*;

        match self.state 
        {
            // Not homed, no movement
            NotHomed => Velocity::ZERO,
            // No movement in idle state
            Idle => Velocity::ZERO,
            GoingIn => 
            {
                let position = self.limit_inner;
                // Move in at a speed of 10-100 mm/s
                let speed = match self.is_close_to_target(position)
                {
                    true  => Velocity::new::<mmps>(10.0),
                    false => Velocity::new::<mmps>(100.0),
                };

                self.speed_to_position(position, speed)
            }
            GoingOut => 
            {
                let position = self.limit_outer;
                // Move in at a speed of 10-100 mm/s
                let speed = match self.is_close_to_target(position)
                {
                    true  => Velocity::new::<mmps>(10.0),
                    false => Velocity::new::<mmps>(100.0),
                };

                self.speed_to_position(position, speed)
            }
            Homing(state) => 
                Self::velocity_from_homing_state(state),
            Traversing(state) => 
                self.velocity_from_traversing_state(state, spool_speed),
        }
    }

    fn velocity_from_homing_state(homing_state: HomingState) -> Velocity
    {
        use millimeter_per_second as mmps;
        use HomingState::*;

        match homing_state 
        {
            Initialize => Velocity::ZERO,
            // Move out at a speed of 10 mm/s
            EscapeEndstop => Velocity::new::<mmps>(10.0),
            // Move out at a speed of 2 mm/s
            FindEndstopFineDistancing => Velocity::new::<mmps>(2.0),
            // Move in at a speed of -100 mm/s
            FindEndstopCoarse => Velocity::new::<mmps>(-100.0),
            // move into the endstop at 2 mm/s
            FindEndstopFine => Velocity::new::<mmps>(-2.0),
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

        let offset = Length::new::<millimeter>(0.01);

        let (target_position, speed) = match traversing_state 
        {
            // Move out at a speed of 100 mm/s initially
            GoingOut =>
            {
                let position = self.limit_outer - self.padding + offset;
                let speed = Velocity::new::<millimeter_per_second>(100.0);
                (position, speed)
            }
            TraversingIn =>
            {
                let position = self.limit_inner + self.padding - offset;
                let speed = Self::calculate_traverse_speed(spool_speed, self.step_size);
                (position, speed)
            }
            TraversingOut =>
            {
                let position = self.limit_outer - self.padding + offset;
                let speed = Self::calculate_traverse_speed(spool_speed, self.step_size);
                (position, speed)
            }
        };

        self.speed_to_position(target_position, speed)
    }
}

#[cfg(test)]
mod tests 
{
    use approx::assert_relative_eq;
    use super::*;

    #[test]
    fn test_calculate_traverse_speed() 
    {
        let spool_speed = AngularVelocity::new::<revolution_per_second>(1.0); // 1 revolution per second
        let step_size = Length::new::<millimeter>(1.75); // 1.75 mm step size

        let traverse_speed = Traverse::calculate_traverse_speed(spool_speed, step_size);

        let speed = traverse_speed.get::<millimeter_per_second>();

        assert_relative_eq!(speed, 1.75, epsilon = f64::EPSILON);
    }
}