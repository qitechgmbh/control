use control_core::{
    actors::{
        digital_input_getter::DigitalInputGetter, stepper_driver_el70x1::StepperDriverEL70x1,
    },
    converters::linear_step_converter::LinearStepConverter,
};
use uom::{
    ConstZero,
    si::{
        f64::{Length, Velocity},
        length::millimeter,
        velocity::millimeter_per_second,
    },
};

#[derive(Debug)]
pub struct TraverseController {
    enabled: bool,
    limit_inner: f64,
    limit_outer: f64,
    current_position: f64,
    state: State,
    converter: LinearStepConverter,
}

#[derive(Debug, PartialEq, Eq)]
pub enum State {
    /// Initial state
    NotHomed,

    /// Homing is in progress
    ///
    /// After homing is done, the state will change to [`State::Idle`]
    Homing,

    /// Doing nothing
    /// Already homed
    Idle,

    /// Move between inner and outer limits
    Traversing(TraversingState),

    /// Going to inner limit
    ///
    /// After reaching the inner limit, the state will change to [`State::Idle`]
    GoingIn,

    /// Going to outer limit
    ///
    /// After reaching the outer limit, the state will change to [`State::Idle`]
    GoingOut,
}

#[derive(Debug, PartialEq, Eq)]
pub enum TraversingState {
    /// Like [`State::GoingIn`] but will go into [`State::GoingOut`] after reaching the inner limit
    GoingIn,

    /// Like [`State::GoingOut`] but will go into [`State::GoingIn`] after reaching the outer limit
    GoingOut,
}

impl TraverseController {
    pub fn new(limit_inner: f64, limit_outer: f64) -> Self {
        Self {
            enabled: false,
            limit_inner,
            limit_outer,
            current_position: 0.0,
            state: State::NotHomed,
            converter: LinearStepConverter::from_circumference(
                200,
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

    pub fn set_limit_inner(&mut self, limit: f64) {
        self.limit_inner = limit;
    }

    pub fn set_limit_outer(&mut self, limit: f64) {
        self.limit_outer = limit;
    }

    pub fn get_limit_inner(&self) -> f64 {
        self.limit_inner
    }

    pub fn get_limit_outer(&self) -> f64 {
        self.limit_outer
    }

    pub fn get_current_position(&self) -> f64 {
        self.current_position
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
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
        self.state = State::Homing;
    }

    pub fn start_traversing(&mut self) {
        self.state = State::Traversing(TraversingState::GoingIn);
    }

    pub fn is_homed(&self) -> bool {
        // if not [`State::NotHomed`], then it is homed
        !matches!(self.state, State::NotHomed)
    }

    pub fn is_going_in(&self) -> bool {
        // [`State::GoingIn`] or [`State::Traversing(TraversingState::GoingIn)`] matches!
        matches!(
            self.state,
            State::GoingIn | State::Traversing(TraversingState::GoingIn)
        )
    }

    pub fn is_going_out(&self) -> bool {
        // [`State::GoingOut`] or [`State::Traversing(TraversingState::GoingOut)`] matches!
        matches!(
            self.state,
            State::GoingOut | State::Traversing(TraversingState::GoingOut)
        )
    }

    pub fn is_going_home(&self) -> bool {
        // [`State::Homing`]
        matches!(self.state, State::Homing)
    }

    pub fn is_traversing(&self) -> bool {
        // [`State::Traversing`]
        matches!(self.state, State::Traversing(_))
    }
}

impl TraverseController {
    /// Gets the current traverse position as a [`Length`].
    #[allow(unused)]
    fn get_position(&self, traverse: &StepperDriverEL70x1) -> Length {
        let steps = traverse.get_position();
        self.converter.steps_to_distance(steps as f64)
    }

    /// Calculates a desired speed based on the current state and the end stop status.
    ///
    /// Positive speed moved out, negative speed moves in.
    fn get_speed(
        &mut self,
        traverse: &mut StepperDriverEL70x1,
        traverse_end_stop: &DigitalInputGetter,
    ) -> Velocity {
        // Don't move if not enabled or in a state that doesn't result in movement
        if !self.enabled {
            return Velocity::ZERO;
        }

        // let position = self.get_position(traverse);

        // Check state transitions
        if self.state == State::Homing && traverse_end_stop.value() == true {
            // Set poition of traverse to 0
            traverse.set_position(0);
            // Put Into Idle
            self.state = State::Idle;
        }

        match self.state {
            State::Homing => Velocity::new::<millimeter_per_second>(-50.0), // Homing speed
            State::Idle => Velocity::ZERO, // No movement in idle state
            State::NotHomed => Velocity::ZERO, // Not homed, no movement
            State::Traversing(_) => todo!(),
            State::GoingIn => todo!(),
            State::GoingOut => todo!(),
        }
    }

    pub fn update_speed(
        &mut self,
        traverse: &mut StepperDriverEL70x1,
        traverse_end_stop: &DigitalInputGetter,
    ) {
        let speed = self.get_speed(traverse, traverse_end_stop);
        let steps_per_second = self.converter.velocity_to_steps(speed);
        traverse.set_speed(steps_per_second as i32);
    }
}
