use std::time::Duration;
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

use crate::winder::devices::{OperationState, Spool};

mod types;
pub use types::HomingState;
pub use types::State;
pub use types::TraversingState;

#[derive(Debug)]
pub struct Traverse {
    config: Config,

    // hardware
    motor: StepperVelocityEL70x1,
    laser_pointer: DigitalOutput,
    limit_switch: DigitalInput,

    // config
    operation_state: OperationState,
    state: State,
    position: Length,
    limit_inner: Length,
    limit_outer: Length,
    step_size: Length,
    padding: Length,

    // converters
    fullstep_converter: LinearStepConverter,
    microstep_converter: LinearStepConverter,
}

#[derive(Debug, Clone)]
pub struct Config {
    // physical properties
    pub circumference: Length,
    pub steps_per_revolution: i16,
    pub micro_steps_per_step: i16,

    // virtual properties
    pub length_tolerance:    Length,

    // defaults
    pub limit_inner_default: Length,
    pub limit_outer_default: Length,
    pub padding_default:     Length,
    pub step_size_default:   Length,

    pub speed_config: SpeedConfig,

    pub validation_delay: Duration,
}

#[derive(Debug, Clone)]
pub struct SpeedConfig
{
    pub move_close:                          Velocity,
    pub move_not_close:                      Velocity,
    pub homing_escape_end_stop:              Velocity,
    pub homing_find_endstop_fine_distancing: Velocity,
    pub homing_find_endstop_coarse:          Velocity,
    pub homing_find_endstop_fine:            Velocity,
    pub traverse_going_out:                  Velocity,
}

// public interface
impl Traverse {
    pub fn new(
        config: Config,
        motor: StepperVelocityEL70x1,
        limit_switch: DigitalInput,
        laser_pointer: DigitalOutput,
    ) -> Self {
        let limit_inner = config.limit_inner_default;
        let limit_outer = config.limit_outer_default;
        let circumference = config.circumference;

        let step_size = config.step_size_default;
        let padding = config.padding_default;

        let fullstep_converter = LinearStepConverter::from_circumference(
            config.steps_per_revolution, 
            circumference
        );

        let microstep_converter = LinearStepConverter::from_circumference(
            config.steps_per_revolution * config.micro_steps_per_step,
            circumference,
        );

        Self {
            config,
            operation_state: OperationState::Disabled,
            state: State::NotHomed,
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

    pub fn update(&mut self, spool: &Spool) {
        if self.operation_state == OperationState::Disabled {
            return;
        }

        self.update_position();
        self.update_state();
        self.update_speed(spool.speed());
    }
}

// getters + setters
impl Traverse {
    pub fn set_operation_state(&mut self, operation_state: OperationState) {
        use OperationState::*;

        // No change, nothing to do
        if self.operation_state == operation_state {
            return;
        }

        // Leaving standby, enable motor
        if self.operation_state == Disabled {
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

    pub fn state(&self) -> State {
        self.state
    }

    pub fn limit_inner(&self) -> Length {
        self.limit_inner
    }

    pub fn try_set_limit_inner(&mut self, limit_inner: Length) -> Result<(), ()> {
        // validate the new inner limit against current outer limit
        if !Self::validate_traverse_limits(limit_inner, self.limit_outer) {
            // don't update if validation fails - keep the current value
            return Err(());
        }

        self.limit_inner = limit_inner;
        Ok(())
    }

    pub fn limit_outer(&self) -> Length {
        self.limit_outer
    }

    pub fn try_set_limit_outer(&mut self, limit_outer: Length) -> Result<(), ()> {
        // validate the new outer limit against current inner limit
        if !Self::validate_traverse_limits(self.limit_inner, limit_outer) {
            // don't update if validation fails - keep the current value
            return Err(());
        }

        self.limit_outer = limit_outer;
        Ok(())
    }

    pub fn step_size(&self) -> Length {
        self.step_size
    }
    pub fn set_step_size(&mut self, step_size: Length) {
        self.step_size = step_size;
    }

    pub fn padding(&self) -> Length {
        self.padding
    }
    pub fn set_padding(&mut self, padding: Length) {
        self.padding = padding;
    }

    pub fn current_position(&self) -> Option<Length> {
        match self.state.is_homed() {
            true => Some(self.position),
            false => None,
        }
    }

    pub fn laser_pointer_enabled(&self) -> bool {
        self.laser_pointer.get()
    }

    pub fn set_laser_pointer_enabled(&mut self, value: bool) {
        self.laser_pointer.set(value);
    }
}

// State management
impl Traverse {
    pub fn can_goto_limit_inner(&self) -> bool {
        self.operation_state == OperationState::Disabled
            || !self.state.is_homed()
            || self.state.is_going_in()
            || self.state.is_going_home()
            || self.state.is_traversing()
    }

    pub fn try_goto_limit_inner(&mut self) -> Result<(), ()> {
        if !self.can_goto_limit_inner() {
            return Err(());
        }

        self.state = State::GoingIn;
        Ok(())
    }

    pub fn can_goto_limit_outer(&self) -> bool {
        let state = self.state;
        self.operation_state == OperationState::Disabled
            || !state.is_homed()
            || state.is_going_out()
            || state.is_going_home()
            || state.is_traversing()
    }

    pub fn try_goto_limit_outer(&mut self) -> Result<(), ()> {
        if !self.can_goto_limit_outer() {
            return Err(());
        }

        self.state = State::GoingOut;
        Ok(())
    }

    pub fn can_go_home(&self) -> bool {
        let state = self.state;
        self.operation_state == OperationState::Disabled
            || state.is_going_out()
            || state.is_going_home()
            || state.is_traversing()
    }

    pub fn try_goto_home(&mut self) -> Result<(), ()> {
        if !self.can_go_home() {
            return Err(());
        }

        self.goto_home();
        Ok(())
    }

    fn goto_home(&mut self) {
        self.state = State::Homing(HomingState::Initialize);
    }

    fn start_traversing(&mut self) {
        self.state = State::Traversing(TraversingState::GoingOut);
    }
}

// helpers
impl Traverse {
    fn update_position(&mut self) {
        let steps = self.motor.get_position() as f64;
        self.position = self.microstep_converter.steps_to_distance(steps);
    }

    fn update_state(&mut self) {
        if let Some(next_state) = self.next_state() {
            self.state = next_state;
        }
    }

    fn update_speed(&mut self, spool_speed: AngularVelocity) {
        let steps_per_second = self.compute_output_steps(spool_speed);
        _ = self.motor.set_speed(steps_per_second);
    }

    fn compute_output_steps(&self, spool_speed: AngularVelocity) -> f64 {
        let speed = self.speed_from_state(spool_speed);
        self.fullstep_converter.velocity_to_steps(speed)
    }

    fn endstop_triggered(&self) -> bool {
        self.limit_switch.get_value().unwrap_or(false)
    }

    fn calculate_traverse_speed(spool_speed: AngularVelocity, step_size: Length) -> Velocity {
        let spool_speed = spool_speed.get::<revolution_per_second>();
        let step_size = step_size.get::<millimeter>();

        // Calculate the traverse speed directly from spool speed and step size
        Velocity::new::<millimeter_per_second>(spool_speed * step_size)
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

    /// Calculate distance to position
    fn distance_to_position(&self, target_position: Length) -> Length {
        (self.position - target_position).abs()
    }

    fn is_at_position(&self, target_position: Length) -> bool {
        let tolerance = self.config.length_tolerance;
        let upper_tolerance = target_position + tolerance.abs();
        let lower_tolerance = target_position - tolerance.abs();
        lower_tolerance <= self.position && self.position <= upper_tolerance
    }

    /// Validates that traverse limits maintain proper constraints:
    /// - Inner limit must be smaller than outer limit
    /// - At least 0.9mm difference between inner and outer limits
    fn validate_traverse_limits(inner: Length, outer: Length) -> bool {
        outer > inner + Length::new::<millimeter>(0.9)
    }

    fn is_close_to_target(&self, target: Length) -> bool {
        self.distance_to_position(target).abs() <= Length::new::<millimeter>(1.0)
    }
}

// state updates
impl Traverse {
    fn next_state(&mut self) -> Option<State> {
        use State::*;

        match self.state {
            NotHomed | Idle => None,
            GoingIn => {
                // wait until we reach the inner limit
                if !self.is_at_position(self.limit_inner) {
                    return None;
                }

                Some(State::Idle)
            }
            GoingOut => {
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

    fn next_state_from_homing_state(&mut self, homing_state: HomingState) -> Option<State> {
        use HomingState::*;

        match homing_state {
            Initialize => {
                if self.endstop_triggered() {
                    // If endstop is triggered, escape the endstop
                    Some(State::Homing(EscapeEndstop))
                } else {
                    // If endstop is not triggered, move to the endstop
                    Some(State::Homing(FindEndstopCoarse))
                }
            }
            EscapeEndstop => {
                // move out until endstop is not triggered anymore
                if self.endstop_triggered() {
                    return None;
                }

                // now start finding
                Some(State::Homing(FindEndstopFineDistancing))
            }
            FindEndstopFineDistancing => {
                // move out until endstop is not triggered anymore
                if self.endstop_triggered() {
                    return None;
                }

                Some(State::Homing(FindEndstopFine))
            }
            FindEndstopFine => {
                // move to endstop
                if !self.endstop_triggered() {
                    return None;
                }

                // set poition of traverse to 0
                self.motor.set_position(0);

                // now validate
                Some(State::Homing(Validate(Instant::now())))
            }
            FindEndstopCoarse => {
                // move to endstop
                if !self.endstop_triggered() {
                    return None;
                }

                // now move away from endstop
                Some(State::Homing(FindEndstopFineDistancing))
            }
            Validate(instant) => {
                if instant.elapsed() <= self.config.validation_delay {
                    return None;
                }

                // should be at zero now
                if self.is_at_position(Length::ZERO) {
                    Some(State::Idle)
                } else
                // validation failed. retry
                {
                    Some(State::Homing(Initialize))
                }
            }
        }
    }

    fn next_state_from_traversing_state(&mut self, state: TraversingState) -> Option<State> {
        use TraversingState::*;

        match state {
            TraversingIn => {
                // inner limit not reached yet
                if self.position > self.limit_inner + self.padding {
                    return None;
                }

                // now traverse to out
                Some(State::Traversing(TraversingOut))
            }
            GoingOut | TraversingOut => {
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
impl Traverse {
    fn speed_from_state(&self, spool_speed: AngularVelocity) -> Velocity {
        use State::*;

        match self.state {
            // Not homed, no movement
            NotHomed => Velocity::ZERO,
            // No movement in idle state
            Idle => Velocity::ZERO,
            GoingIn => {
                let position = self.limit_inner;
                let speed = match self.is_close_to_target(position) {
                    true => self.config.speed_config.move_close,
                    false => self.config.speed_config.move_not_close,
                };

                self.speed_to_position(position, speed)
            }
            GoingOut => {
                let position = self.limit_outer;
                let speed = match self.is_close_to_target(position) {
                    true => self.config.speed_config.move_close,
                    false => self.config.speed_config.move_not_close,
                };

                self.speed_to_position(position, speed)
            }
            Homing(state) => self.speed_from_homing_state(state),
            Traversing(state) => self.speed_from_traversing_state(state, spool_speed),
        }
    }

    fn speed_from_homing_state(&self, homing_state: HomingState) -> Velocity {
        use HomingState::*;

        let sc = &self.config.speed_config;

        match homing_state {
            Initialize => Velocity::ZERO,
            EscapeEndstop => sc.homing_escape_end_stop,
            FindEndstopFineDistancing => sc.homing_find_endstop_fine_distancing,
            FindEndstopCoarse => sc.homing_find_endstop_coarse,
            FindEndstopFine => sc.homing_find_endstop_fine,
            Validate(_) => Velocity::ZERO,
        }
    }

    fn speed_from_traversing_state(
        &self,
        traversing_state: TraversingState,
        spool_speed: AngularVelocity,
    ) -> Velocity {
        use TraversingState::*;

        let offset = Length::new::<millimeter>(0.01);

        let (target_position, speed) = match traversing_state {
            // Move out at a speed of 100 mm/s initially
            GoingOut => {
                let position = self.limit_outer - self.padding + offset;
                let speed = self.config.speed_config.traverse_going_out;
                (position, speed)
            }
            TraversingIn => {
                let position = self.limit_inner + self.padding - offset;
                let speed = Self::calculate_traverse_speed(spool_speed, self.step_size);
                (position, speed)
            }
            TraversingOut => {
                let position = self.limit_outer - self.padding + offset;
                let speed = Self::calculate_traverse_speed(spool_speed, self.step_size);
                (position, speed)
            }
        };

        self.speed_to_position(target_position, speed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn test_calculate_traverse_speed() {
        let spool_speed = AngularVelocity::new::<revolution_per_second>(1.0); // 1 revolution per second
        let step_size = Length::new::<millimeter>(1.75); // 1.75 mm step size

        let traverse_speed = Traverse::calculate_traverse_speed(spool_speed, step_size);

        let speed = traverse_speed.get::<millimeter_per_second>();

        assert_relative_eq!(speed, 1.75, epsilon = f64::EPSILON);
    }
}
