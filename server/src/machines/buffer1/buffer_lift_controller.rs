use std::time::Instant;

use control_core::{
    controllers::{
        first_degree_motion::linear_acceleration_speed_controller::LinearAccelerationLimitingController,
        second_degree_motion::linear_jerk_speed_controller::LinearJerkSpeedController,
    },
    converters::linear_step_converter::LinearStepConverter,
    uom_extensions::{
        acceleration::meter_per_minute_per_second, jerk::meter_per_minute_per_second_squared,
        velocity::meter_per_minute,
    },
};
use ethercat_hal::io::{
    digital_input::DigitalInput, stepper_velocity_el70x1::StepperVelocityEL70x1,
};
use uom::{
    ConstZero,
    si::{
        acceleration::centimeter_per_second_squared,
        f64::{Acceleration, Jerk, Length, Velocity},
        length::millimeter,
        velocity::{mile_per_minute, millimeter_per_second},
    },
};

#[derive(Debug)]
pub struct BufferLiftController {
    /// Whether the speed controller is enabled or not
    enabled: bool,
    position: Length,
    limit_top: Length,
    /// Stepper driver. Controls buffer stepper motor
    pub stepper_driver: StepperVelocityEL70x1,
    // Step Converter
    pub fullstep_converter: LinearStepConverter,
    pub microstep_converter: LinearStepConverter,

    /// Linear acceleration controller to dampen speed change
    acceleration_controller: LinearAccelerationLimitingController,

    /// Forward rotation direction. If false, applies negative sign to speed
    forward: bool,
    state: State,

    /// Fixed constants
    spool_amount: u8,

    /// Variables
    current_input_speed: Velocity,
    target_output_speed: Velocity,
    lift_speed: Velocity,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum State {
    /// Initial state
    NotHomed,

    /// Doing nothing
    /// Already homed
    Idle,

    /// Going to upper limit
    ///
    /// After reaching the upper limit, the state will change to [`State::Idle`]
    GoingUp,

    /// Going to lower limit
    ///
    /// After reaching the lower limit, the state will change to [`State::Idle`]
    GoingDown,

    /// Homing in progress
    ///
    /// After homing is done, the state will change to [`State::Idle`]
    Homing(HomingState),

    /// Move at target speed to Buffer
    Buffering(BufferingState),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum BufferingState {
    GoingUp,
    Filling,
    Emptying,
}

#[derive(Debug, PartialEq, Eq, Clone)]
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
    FindEnstopFineDistancing,

    /// In this state the traverse is fast until it reaches the endstop
    FindEndstopCoarse,

    /// In this state the traverse is moving slowly until it reaches the endstop
    FindEndtopFine,

    /// In this state we check if th current position is actually 0.0, if not we redo the homing routine
    Validate(Instant),
}

impl BufferLiftController {
    pub fn new(driver: StepperVelocityEL70x1, limit_top: Length, microsteps: u8) -> Self {
        Self {
            enabled: false,
            position: Length::ZERO,
            limit_top,
            stepper_driver: driver,
            fullstep_converter: LinearStepConverter::from_circumference(
                200,
                Length::new::<millimeter>(35.0),
            ),
            microstep_converter: LinearStepConverter::from_circumference(
                200 * microsteps as i16,
                Length::new::<millimeter>(35.0),
            ),
            spool_amount: 13,
            state: State::NotHomed,
            forward: true,
            acceleration_controller: LinearAccelerationLimitingController::new_simple(
                Acceleration::new::<centimeter_per_second_squared>(1.0),
                Velocity::new::<millimeter_per_second>(10.0),
            ),
            current_input_speed: Velocity::ZERO,
            target_output_speed: Velocity::ZERO,
            lift_speed: Velocity::ZERO,
        }
    }
}

impl BufferLiftController {
    /// Calculate the speed of the buffer lift from current input speed
    ///
    /// Formula: input_speed / ( 2 * spool_amount )
    pub fn calculate_buffer_lift_speed(&mut self) -> Velocity {
        self.lift_speed = Velocity::new::<millimeter_per_second>(
            (self.current_input_speed.get::<millimeter_per_second>()
                - self.target_output_speed.get::<millimeter_per_second>())
                / (2.0 * self.spool_amount as f64 - 1.0),
        );
        self.lift_speed
    }

    pub fn update_speed(&mut self, t: Instant) -> Velocity {
        let speed = match self.enabled {
            true => self.calculate_buffer_lift_speed(),
            false => Velocity::ZERO,
        };

        let speed = if self.forward { speed } else { -speed };

        self.acceleration_controller.update(speed, t)
    }
}

/// Getter & Setter
impl BufferLiftController {
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        self.stepper_driver.set_enabled(enabled);
    }
    pub fn set_forward(&mut self, forward: bool) {
        self.forward = forward;
    }

    pub fn set_current_input_speed(&mut self, speed: f64) {
        self.current_input_speed = Velocity::new::<meter_per_minute>(speed);
    }
    pub fn set_target_output_speed(&mut self, speed: f64) {
        self.target_output_speed = Velocity::new::<meter_per_minute>(speed);
    }
    pub fn get_current_input_speed(&self) -> Velocity {
        self.current_input_speed
    }
    pub fn get_target_output_speed(&self) -> Velocity {
        self.target_output_speed
    }
    pub fn get_lift_speed(&self) -> Velocity {
        self.lift_speed
    }
}

/// State management
impl BufferLiftController {
    pub fn goto_home(&mut self) {
        self.state = State::Homing(HomingState::Initialize);
    }

    pub fn start_buffering(&mut self) {
        self.state = State::Buffering(BufferingState::GoingUp);
    }

    pub fn is_homed(&self) -> bool {
        // if not [`State::NotHomed`], then it is homed
        !matches!(self.state, State::NotHomed)
    }

    pub fn is_filling(&self) -> bool {
        // [`State::Filling`]
        matches!(self.state, State::GoingUp)
    }

    pub fn is_emptying(&self) -> bool {
        // [`State::Emptying`]
        matches!(self.state, State::GoingDown)
    }

    pub fn is_going_home(&self) -> bool {
        // [`State::Homing`]
        matches!(self.state, State::Homing(_))
    }

    pub fn is_buffering(&self) -> bool {
        // [`State::Buffering`]
        matches!(self.state, State::Buffering(_))
    }
}

impl BufferLiftController {
    // If at inner limit within a tolerance
    fn is_at_position(&self, target_position: Length, tolerance: Length) -> bool {
        let upper_tolerance = target_position + tolerance.abs();
        let lower_tolerance = target_position - tolerance.abs();
        if self.position >= lower_tolerance && self.position <= upper_tolerance {
            return true;
        } else {
            return false;
        }
    }

    /// Calculates distance to position
    fn distance_to_position(&self, target_position: Length) -> Length {
        if self.position > target_position {
            return self.position - target_position;
        } else if self.position < target_position {
            return target_position - self.position;
        } else {
            return Length::ZERO;
        }
    }

    // Changes the direction of the speed based on the current position and target position
    fn speed_to_position(&self, target_position: Length, absolute_speed: Velocity) -> Velocity {
        // If we are over the target position we need to move negative
        if self.position > target_position {
            return -absolute_speed.abs();
        } else if self.position < target_position {
            return absolute_speed.abs();
        } else {
            return Velocity::ZERO;
        }
    }

    /// Gets the current traverse position as a [`Length`].
    pub fn sync_position(&mut self, stepper_driver: &StepperVelocityEL70x1) {
        let steps = stepper_driver.get_position();
        self.position = self.microstep_converter.steps_to_distance(steps as f64);
    }

    /// Update the [`did_change_state`] flag
    /// Only consider the major state not the sub states
    fn update_did_change_state(&mut self, old_state: &State) -> bool {
        match self.state {
            State::NotHomed => !matches!(old_state, State::NotHomed),
            State::Idle => !matches!(old_state, State::Idle),
            State::GoingUp => !matches!(old_state, State::GoingUp),
            State::GoingDown => !matches!(old_state, State::GoingDown),
            State::Homing(_) => !matches!(old_state, State::Homing(_)),
            State::Buffering(_) => !matches!(old_state, State::Buffering(_)),
        }
    }

    /// Calculates a desired speed based on the current state and the end stop status.
    ///
    /// Positive speed moves Up, negative speed moves down
    fn get_speed(
        &mut self,
        stepper_driver: &mut StepperVelocityEL70x1,
        lift_end_stop: &DigitalInput,
    ) -> Velocity {
        // Don't move if not enabled or in a state that doesn't result in movement
        if !self.enabled {
            return Velocity::ZERO;
        }

        self.sync_position(stepper_driver);

        // save state before
        let old_state = self.state.clone();

        // Automatic Transitions
        match &self.state {
            State::NotHomed => {}
            State::Idle => {}
            State::GoingDown => {
                // If top limit is reached
                if self.is_at_position(Length::new<centimeter>(0.0),Length::new<millimeter>(0.1)) {
                    // Put Into Idle
                    self.state = State::Idle;
                }
            }
            State::GoingUp => {
                if self.is_at_position(self.limit_top,Length::new::<millimeter>(0.1)) {
                    self.state = State::Idle;
                }
            }
            State::Homing(homing_state) => match homing_state {
                HomingState::Initialize => {
                                // If endstop is triggered, escape endstop
                                if lift_end_stop.get_value().unwrap_or(false) == true {
                                    self.state = State::Homing(HomingState::EscapeEndstop);
                                } else {
                                    // If endstop is not triggered, move to the endstop
                                    self.state = State::Homing(HomingState::FindEndstopCoarse);
                                }
                            }
                HomingState::EscapeEndstop => {
                    // Move out until endstop is not triggered anymore
                    if lift_end_stop.get_value().unwrap_or(false) == false {
                        self.state = State::Homing(HomingState::FindEnstopFineDistancing);
                    }
                }
                HomingState::FindEnstopFineDistancing => {
                    // Move out until endstop is not triggered anymore
                    if lift_end_stop.get_value().unwrap_or(false) == false {
                        // Find endstop fine
                        self.state = State::Homing(HomingState::FindEndtopFine);
                    }
                }
                HomingState::FindEndtopFine => {
                    // If endstop is reached change to idle
                    if lift_end_stop.get_value().unwrap_or(false) == true {
                        // Set position of lift to 0
                        stepper_driver.set_position(0);
                        // Put into Idle
                        self.state = State::Homing(HomingState::Validate(Instant::now()));
                    }
                }
                HomingState::FindEndstopCoarse => {
                    // Move to endstop
                    if lift_end_stop.get_value().unwrap_or(false) == true {
                        // Move away from endstop
                        self.state = State::Homing(HomingState::FindEnstopFineDistancing)
                    }
                }
                HomingState::Validate(instant) => todo!(),
            }
        }
    }
}
