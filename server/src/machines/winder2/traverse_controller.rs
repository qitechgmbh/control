use control_core::{
    actors::{
        digital_input_getter::DigitalInputGetter, stepper_driver_el70x1::StepperDriverEL70x1,
    },
    converters::linear_step_converter::LinearStepConverter,
};
use uom::{
    ConstZero,
    si::{
        f64::{AngularVelocity, Length, Velocity},
        length::millimeter,
        velocity::millimeter_per_second,
    },
};

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

#[derive(Debug, PartialEq, Eq, Clone)]
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
}

impl TraverseController {
    pub fn new(limit_inner: Length, limit_outer: Length, microsteps: u8) -> Self {
        Self {
            enabled: false,
            position: Length::ZERO,
            limit_inner,
            limit_outer,
            step_size: Length::new::<millimeter>(1.0), // Default step size
            padding: Length::new::<millimeter>(0.01),  // Default padding
            state: State::NotHomed,
            did_change_state: false,
            fullstep_converter: LinearStepConverter::from_circumference(
                200,
                Length::new::<millimeter>(35.0),
            ),
            microstep_converter: LinearStepConverter::from_circumference(
                200 * microsteps as i16,
                Length::new::<millimeter>(35.0),
            ),
        }
    }
}

// Getter & Setter
impl TraverseController {
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
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
        match self.is_homed() {
            true => Some(self.position),
            false => None,
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn dif_change_state(&mut self) -> bool {
        let did_change = self.did_change_state;
        // Reset the flag
        self.did_change_state = false;
        did_change
    }
}

// State management
impl TraverseController {
    pub fn goto_limit_inner(&mut self) {
        self.state = State::GoingIn;
    }

    pub fn goto_limit_outer(&mut self) {
        self.state = State::GoingOut;
    }

    pub fn goto_home(&mut self) {
        self.state = State::Homing(HomingState::Initialize);
    }

    pub fn start_traversing(&mut self) {
        self.state = State::Traversing(TraversingState::GoingOut);
    }

    pub fn is_homed(&self) -> bool {
        // if not [`State::NotHomed`], then it is homed
        !matches!(self.state, State::NotHomed)
    }

    pub fn is_going_in(&self) -> bool {
        // [`State::GoingIn`]
        matches!(self.state, State::GoingIn)
    }

    pub fn is_going_out(&self) -> bool {
        // [`State::GoingOut`]
        matches!(self.state, State::GoingOut)
    }

    pub fn is_going_home(&self) -> bool {
        // [`State::Homing`]
        matches!(self.state, State::Homing(_))
    }

    pub fn is_traversing(&self) -> bool {
        // [`State::Traversing`]
        matches!(self.state, State::Traversing(_))
    }
}

impl TraverseController {
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

    /// Calculate distance to position
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
    pub fn sync_position(&mut self, traverse: &StepperDriverEL70x1) {
        let steps = traverse.get_position();
        self.position = self.microstep_converter.steps_to_distance(steps as f64);
    }

    /// Update the [`did_change_state`] flag
    /// Only considers the major state not the sub states
    fn update_did_change_state(&mut self, old_state: &State) -> bool {
        match self.state {
            State::NotHomed => !matches!(old_state, State::NotHomed),
            State::Idle => !matches!(old_state, State::Idle),
            State::GoingIn => !matches!(old_state, State::GoingIn),
            State::GoingOut => !matches!(old_state, State::GoingOut),
            State::Homing(_) => !matches!(old_state, State::Homing(_)),
            State::Traversing(_) => !matches!(old_state, State::Traversing(_)),
        }
    }

    /// Calculates a desired speed based on the current state and the end stop status.
    ///
    /// Positive speed moved out, negative speed moves in.
    fn get_speed(
        &mut self,
        traverse: &mut StepperDriverEL70x1,
        traverse_end_stop: &DigitalInputGetter,
        spool_speed: AngularVelocity,
    ) -> Velocity {
        // Don't move if not enabled or in a state that doesn't result in movement
        if !self.enabled {
            return Velocity::ZERO;
        }

        self.sync_position(traverse);

        // save state before
        let old_state = self.state.clone();

        // Automatic Transitions
        match &self.state {
            State::NotHomed => {}
            State::Idle => {}
            State::GoingIn => {
                // If inner limit is reached
                if self.is_at_position(self.limit_inner, Length::new::<millimeter>(0.01)) {
                    // Put Into Idle
                    self.state = State::Idle;
                }
            }
            State::GoingOut => {
                // If outer limit is reached
                if self.is_at_position(self.limit_outer, Length::new::<millimeter>(0.01)) {
                    // Put Into Idle
                    self.state = State::Idle;
                }
            }
            State::Homing(homing_state) => match homing_state {
                HomingState::Initialize => {
                    // If endstop is triggered, escape the endstop
                    if traverse_end_stop.value() == true {
                        self.state = State::Homing(HomingState::EscapeEndstop);
                    } else {
                        // If endstop is not triggered, move to the endstop
                        self.state = State::Homing(HomingState::FindEndstopCoarse);
                    }
                }
                HomingState::EscapeEndstop => {
                    // Move out until endstop is not triggered anymore
                    if traverse_end_stop.value() == false {
                        self.state = State::Homing(HomingState::FindEnstopFineDistancing);
                    }
                }
                HomingState::FindEnstopFineDistancing => {
                    // Move out until endstop is not triggered anymore
                    if traverse_end_stop.value() == false {
                        // Find endstop fine
                        self.state = State::Homing(HomingState::FindEndtopFine);
                    }
                }
                HomingState::FindEndtopFine => {
                    // If endstop is reached change to idle
                    if traverse_end_stop.value() == true {
                        // Set poition of traverse to 0
                        traverse.set_position(0);
                        // Put Into Idle
                        self.state = State::Idle;
                    }
                }
                HomingState::FindEndstopCoarse => {
                    // Move to endstop
                    if traverse_end_stop.value() == true {
                        // Set poition of traverse to 0
                        traverse.set_position(0);
                        // Move awaiy from endstop
                        self.state = State::Homing(HomingState::FindEnstopFineDistancing);
                    }
                }
            },

            // If state changed we
            State::Traversing(traversing_state) => match traversing_state {
                TraversingState::GoingOut => {
                    // If outer limit is reached
                    if self.is_at_position(
                        self.limit_outer - self.padding + Length::new::<millimeter>(0.01),
                        Length::new::<millimeter>(0.02),
                    ) {
                        // Turn around
                        self.state = State::Traversing(TraversingState::TraversingIn);
                    }
                }
                TraversingState::TraversingIn => {
                    // If inner limit is reached
                    if self.is_at_position(
                        self.limit_inner + self.padding - Length::new::<millimeter>(0.01),
                        Length::new::<millimeter>(0.02),
                    ) {
                        // Turn around
                        self.state = State::Traversing(TraversingState::TraversingOut);
                    }
                }
                TraversingState::TraversingOut => {
                    // If outer limit is reached
                    if self.is_at_position(
                        self.limit_outer - self.padding + Length::new::<millimeter>(0.01),
                        Length::new::<millimeter>(0.02),
                    ) {
                        // Turn around
                        self.state = State::Traversing(TraversingState::TraversingIn);
                    }
                }
            },
        }

        // Set the [`did_change_state`] flag
        if self.did_change_state == false {
            self.did_change_state = self.update_did_change_state(&old_state);
        }

        // Speed
        let speed = match &self.state {
            State::NotHomed => Velocity::ZERO, // Not homed, no movement
            State::Idle => Velocity::ZERO,     // No movement in idle state
            State::GoingIn => {
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
            }
            State::GoingOut => {
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
            State::Homing(homing_state) => match homing_state {
                HomingState::Initialize => unreachable!(
                    "Initialize always transitions to another state before speed is set"
                ),
                HomingState::EscapeEndstop => {
                    // Move out at a speed of 10 mm/s
                    Velocity::new::<millimeter_per_second>(10.0)
                }
                HomingState::FindEnstopFineDistancing => {
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
            }, // Homing speed
            State::Traversing(traversing_state) => match traversing_state {
                TraversingState::GoingOut => {
                    // Move out at a speed of 100 mm/s
                    self.speed_to_position(
                        self.limit_outer - self.padding + Length::new::<millimeter>(0.01),
                        Velocity::new::<millimeter_per_second>(100.0),
                    )
                }
                TraversingState::TraversingIn => self.speed_to_position(
                    self.limit_inner + self.padding - Length::new::<millimeter>(0.01),
                    self.calculate_traverse_speed(spool_speed),
                ),
                TraversingState::TraversingOut => self.speed_to_position(
                    self.limit_outer - self.padding + Length::new::<millimeter>(0.01),
                    self.calculate_traverse_speed(spool_speed),
                ),
            },
        };

        speed
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
    pub fn calculate_traverse_speed(&self, spool_speed: AngularVelocity) -> Velocity {
        // Calculate the traverse speed directly from spool speed and step size
        let traverse_speed: Velocity = spool_speed * self.step_size;

        traverse_speed
    }

    pub fn update_speed(
        &mut self,
        traverse: &mut StepperDriverEL70x1,
        traverse_end_stop: &DigitalInputGetter,
        spool_speed: AngularVelocity,
    ) {
        let speed = self.get_speed(traverse, traverse_end_stop, spool_speed);
        let steps_per_second = self.fullstep_converter.velocity_to_steps(speed);
        traverse.set_speed(steps_per_second as i32);
    }
}
