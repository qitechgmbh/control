use std::time::Instant;

use control_core::{
    controllers::first_degree_motion::linear_acceleration_speed_controller::LinearAccelerationLimitingController,
    converters::linear_step_converter::LinearStepConverter,
    uom_extensions::velocity::meter_per_minute,
};
use ethercat_hal::io::{
    digital_input::DigitalInput, stepper_velocity_el70x1::StepperVelocityEL70x1,
};
use tracing::info;
use uom::{
    ConstZero,
    si::{
        acceleration::centimeter_per_second_squared,
        f64::{Acceleration, Length, Velocity},
        length::millimeter,
        velocity::millimeter_per_second,
    },
};

#[derive(Debug)]
pub struct BufferLiftController {
    /// Whether the speed controller is enabled or not
    enabled: bool,
    position: Length,
    limit_top: Length,
    limit_bottom: Length,
    step_size: Length,
    padding: Length,
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

    // A sticky flag if the [`State`] changed (not the sub states)
    // Needed to send state updates to the UI
    did_change_state: bool,
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
    /// In this state the lift is not moving but checks if the endstop si triggered
    /// If the endstop is triggered we go into [`HomingState::EscapeEndstop`]
    /// If the endstop is not triggered we go into [`HomingState::FindEndstop`]
    Initialize,

    /// In this state the lift is moving out away from the endstop until it's not triggered anymore
    /// The it goes into [`HomingState::FindEnstopFineDistancing`]
    EscapeEndstop,

    /// Moving out away from the endstop
    /// Then Transition into [`HomingState::FindEndtopFine`]
    FindEnstopFineDistancing,

    /// In this state the lift is fast until it reaches the endstop
    FindEndstopCoarse,

    /// In this state the lift is moving slowly until it reaches the endstop
    FindEndStopFine,

    /// In this state we check if th current position is actually 0.0, if not we redo the homing routine
    Validate(Instant),
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
    pub fn new(
        driver: StepperVelocityEL70x1,
        limit_top: Length,
        limit_bot: Length,
        microsteps: u8,
    ) -> Self {
        Self {
            enabled: false,
            position: Length::ZERO,
            limit_top,
            limit_bottom: limit_bot,
            step_size: Length::new::<millimeter>(1.75), // Default padding
            padding: Length::new::<millimeter>(0.9),    // Default padding
            did_change_state: false,
            stepper_driver: driver,
            fullstep_converter: LinearStepConverter::from_diameter(
                200,
                Length::new::<millimeter>(32.22),
            ),
            microstep_converter: LinearStepConverter::from_diameter(
                200 * microsteps as i16,
                Length::new::<millimeter>(32.22),
            ),
            spool_amount: 13,
            state: State::NotHomed,
            forward: true,
            acceleration_controller: LinearAccelerationLimitingController::new_simple(
                Acceleration::new::<centimeter_per_second_squared>(1.0),
                Velocity::new::<millimeter_per_second>(5.0),
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
    /// Formula: input_speed / ( 2 * spool_amount - 1 )
    pub fn calculate_buffer_lift_speed(&self) -> Velocity {
        let lift_speed = Velocity::new::<millimeter_per_second>(
            (self.current_input_speed.get::<millimeter_per_second>()
                - self.target_output_speed.get::<millimeter_per_second>())
                / (2.0 * self.spool_amount as f64 - 1.0),
        );
        lift_speed
    }

    pub fn update_speed(
        &mut self,
        stepper_driver: &mut StepperVelocityEL70x1,
        lift_end_stop: &DigitalInput,
        t: Instant,
    ) -> Velocity {
        let speed = match self.enabled {
            true => self.get_speed(stepper_driver, lift_end_stop),
            false => Velocity::ZERO,
        };

        // Stepper is installed in reverse direction so we change to sign
        let speed = speed;

        self.acceleration_controller.update(speed, t)
    }
}

/// Getter & Setter
impl BufferLiftController {
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        self.stepper_driver.set_enabled(enabled);
    }

    pub fn set_limit_top(&mut self, limit: Length) {
        self.limit_top = limit;
    }

    pub fn set_limit_bottom(&mut self, limit: Length) {
        self.limit_bottom = limit;
    }

    pub fn set_step_size(&mut self, step_size: Length) {
        self.step_size = step_size;
    }

    pub fn set_padding(&mut self, padding: Length) {
        self.padding = padding;
    }

    pub fn get_limit_top(&self) -> Length {
        self.limit_top
    }

    pub fn get_limit_bottom(&self) -> Length {
        self.limit_bottom
    }

    pub fn get_step_size(&self) -> Length {
        self.step_size
    }

    pub fn get_padding(&self) -> Length {
        self.padding
    }

    pub fn set_forward(&mut self, forward: bool) {
        self.forward = forward;
    }

    pub fn get_current_position(&self) -> Option<Length> {
        match self.is_homed() {
            true => Some(self.position),
            false => None,
        }
    }

    pub fn did_change_state(&mut self) -> bool {
        let did_change = self.did_change_state;
        // Reset the flag
        self.did_change_state = false;
        did_change
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
    pub fn goto_limit_top(&mut self) {
        self.state = State::GoingUp
    }

    pub fn goto_limit_bottom(&mut self) {
        self.state = State::GoingDown
    }

    pub fn goto_home(&mut self) {
        self.state = State::Homing(HomingState::Initialize);
    }

    pub fn start_buffering(&mut self) {
        self.state = State::Buffering(BufferingState::GoingUp);
    }

    pub fn is_going_up(&self) -> bool {
        // [`State::GoingUp`]
        matches!(self.state, State::GoingUp)
    }

    pub fn is_going_down(&self) -> bool {
        // [`State::GoingDown`]
        matches!(self.state, State::GoingDown)
    }

    pub fn is_homed(&self) -> bool {
        // if not [`State::NotHomed`], then it is homed
        !matches!(self.state, State::NotHomed)
    }

    pub fn is_filling(&self) -> bool {
        // [`State::Filling`]
        matches!(self.state, State::Buffering(BufferingState::Filling))
    }

    pub fn is_emptying(&self) -> bool {
        // [`State::Emptying`]
        matches!(self.state, State::Buffering(BufferingState::Emptying))
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

    /// Gets the current lift position as a [`Length`].
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
                // If lower limit is reached
                if self.is_at_position(self.limit_bottom, Length::new::<millimeter>(0.01)) {
                    // Put Into Idle
                    self.state = State::Idle;
                }
            }
            State::GoingUp => {
                // If upper limit is reached
                if self.is_at_position(self.limit_top, Length::new::<millimeter>(0.01)) {
                    // Put Into Idle
                    self.state = State::Idle;
                }
            }
            State::Homing(homing_state) => match homing_state {
                HomingState::Initialize => {
                    // If endstop is triggered, escape the endstop
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
                        self.state = State::Homing(HomingState::FindEndStopFine);
                    }
                }
                HomingState::FindEndStopFine => {
                    // If endstop is reached change to idle
                    if lift_end_stop.get_value().unwrap_or(false) == true {
                        // Set poition of lift to 0
                        stepper_driver.set_position(0);
                        // Put Into Idle
                        self.state = State::Homing(HomingState::Validate(Instant::now()));
                    }
                }
                HomingState::FindEndstopCoarse => {
                    // Move to endstop
                    if lift_end_stop.get_value().unwrap_or(false) == true {
                        // Move awaiy from endstop
                        self.state = State::Homing(HomingState::FindEnstopFineDistancing);
                    }
                }
                HomingState::Validate(instant) => {
                    // If 100ms have passed check if position is actually 0.0
                    if instant.elapsed().as_millis() > 100 {
                        if self.is_at_position(Length::ZERO, Length::new::<millimeter>(0.01)) {
                            // If position is 0.0, put into idle
                            self.state = State::Idle;
                        } else {
                            // If position is not 0.0, redo homing
                            self.state = State::Homing(HomingState::Initialize);
                        }
                    }
                }
            },

            // If state changed we
            State::Buffering(buffering_state) => match buffering_state {
                BufferingState::GoingUp => {}
                BufferingState::Emptying => {}
                BufferingState::Filling => {}
            },
        }

        // Set the [`did_change_state`] flag
        if self.did_change_state == false {
            self.did_change_state = self.update_did_change_state(&old_state);
            info!("{:?}", self.state);
        }

        // Speed
        let speed = match &self.state {
            State::NotHomed => Velocity::ZERO, // Not homed, no movement
            State::Idle => Velocity::ZERO,     // No movement in idle state
            State::GoingDown => {
                // Move down at a speed of 10-100 mm/s
                self.speed_to_position(
                    self.limit_bottom,
                    match self.distance_to_position(self.limit_bottom).abs()
                        > Length::new::<millimeter>(1.0)
                    {
                        true => Velocity::new::<millimeter_per_second>(100.0),
                        false => Velocity::new::<millimeter_per_second>(10.0),
                    },
                )
            }
            State::GoingUp => {
                // Move up at a speed of 10-100 mm/s
                self.speed_to_position(
                    self.limit_top,
                    match self.distance_to_position(self.limit_top).abs()
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
                    // Move down at a speed of 10 mm/s
                    Velocity::new::<millimeter_per_second>(10.0)
                }
                HomingState::FindEnstopFineDistancing => {
                    // Move up at a speed of 2 mm/s
                    Velocity::new::<millimeter_per_second>(2.0)
                }
                HomingState::FindEndstopCoarse => {
                    // Move in at a speed of -100 mm/s
                    Velocity::new::<millimeter_per_second>(-100.0)
                }
                HomingState::FindEndStopFine => {
                    // move into the endstop at 2 mm/s
                    Velocity::new::<millimeter_per_second>(-2.0)
                }
                HomingState::Validate(_) => {
                    // We stand still until the validation cooldown has passed
                    Velocity::ZERO
                }
            }, // Homing speed
            State::Buffering(buffering_state) => match buffering_state {
                BufferingState::GoingUp => {
                    // Move top at a speed of 100 mm/s
                    self.speed_to_position(
                        self.limit_top - self.padding + Length::new::<millimeter>(0.01),
                        Velocity::new::<millimeter_per_second>(100.0),
                    )
                }
                BufferingState::Filling => self.speed_to_position(
                    self.limit_top + self.padding - Length::new::<millimeter>(0.01),
                    Self::calculate_buffer_lift_speed(self),
                ),
                BufferingState::Emptying => self.speed_to_position(
                    self.limit_bottom - self.padding + Length::new::<millimeter>(0.01),
                    Self::calculate_buffer_lift_speed(self),
                ),
            },
        };
        info!("{}", speed.get::<millimeter_per_second>());
        speed
    }
}
