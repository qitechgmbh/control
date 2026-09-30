use std::time::Duration;

use qitech_framework::EnumProperty;

use crate::machines::winder_v2::Mode as WinderMode;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, EnumProperty)]
pub enum Mode {
    #[default]
    Standby,
    Hold,
    Traverse,
}

impl From<WinderMode> for Mode {
    fn from(mode: WinderMode) -> Self {
        match mode {
            WinderMode::Standby => Self::Standby,
            WinderMode::Hold => Self::Hold,
            WinderMode::Pull => Self::Hold,
            WinderMode::Wind => Self::Traverse,
        }
    }
}

#[derive(Debug, Default, PartialEq, Eq, Clone, Copy)]
pub enum State {
    #[default]
    Idle,
    GoingIn,
    GoingOut,
    Homing(HomingState),
    Traversing(TraversingState),
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum TraversingState {
    GoingOut,
    TraversingIn,
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
    FindEndstopFine,

    /// In this state we check if th current position is actually 0.0, if not we redo the homing routine
    Validate(Duration),
}

// --- enum property ---
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
            State::Idle => "idle",
            State::GoingIn => "going_in",
            State::GoingOut => "going_out",
            State::Homing(state) => match state {
                HomingState::Initialize => "initialize",
                HomingState::EscapeEndstop => "escape_endstop",
                HomingState::FindEndstopFineDistancing => "find_endstop_fine_distancing",
                HomingState::FindEndstopCoarse => "find_endstop_coarse",
                HomingState::FindEndstopFine => "find_endstop_fine",
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
            "idle" => State::Idle,
            "going_in" => State::GoingIn,
            "going_out" => State::GoingOut,

            "initialize" => State::Homing(HomingState::Initialize),
            "escape_endstop" => State::Homing(HomingState::EscapeEndstop),
            "find_endstop_fine_distancing" => State::Homing(HomingState::FindEndstopFineDistancing),
            "find_endstop_coarse" => State::Homing(HomingState::FindEndstopCoarse),
            "find_endstop_fine" => State::Homing(HomingState::FindEndstopFine),

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
