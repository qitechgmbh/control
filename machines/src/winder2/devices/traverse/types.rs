use std::time::Instant;

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

impl State
{
    #[allow(dead_code)]
    pub const fn is_idle(self) -> bool 
    {
        matches!(self,State::Idle)
    }

    pub const fn is_homed(self) -> bool 
    {
        !matches!(self, State::NotHomed)
    }

    pub const fn is_going_in(self) -> bool 
    {
        matches!(self,State::GoingIn)
    }

    pub const fn is_going_out(self) -> bool 
    {
        matches!(self,State::GoingOut)
    }

    pub const fn is_going_home(self) -> bool 
    {
        matches!(self, State::Homing(_))
    }

    pub const fn is_traversing(self) -> bool 
    {
        matches!(self, State::Traversing(_))
    }
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
    FindEndstopFine,

    /// In this state we check if th current position is actually 0.0, if not we redo the homing routine
    Validate(Instant),
}