use core::fmt;

use qitech_framework::MachineInstanceIdentification;
use qitech_framework::RuntimeRequestKind;
use qitech_framework::ScalarValue;
use serde::Deserialize;

use crate::api::legacy::types::MachineIdentificationUnique;

pub fn convert_request(
    ident: MachineInstanceIdentification,
    data: serde_json::Value,
) -> Result<RuntimeRequestKind, serde_json::Error> {
    Ok(match serde_json::from_value(data)? {
        // ------------------------------------------------------------
        // Traverse config
        // ------------------------------------------------------------
        Mutation::SetTraverseLimitOuter(v) => RuntimeRequestKind::SetConfigProperty {
            target: ident,
            path: "traverse.limit_outer".to_string(),
            value: ScalarValue::Float(v),
        },

        Mutation::SetTraverseLimitInner(v) => RuntimeRequestKind::SetConfigProperty {
            target: ident,
            path: "traverse.limit_inner".to_string(),
            value: ScalarValue::Float(v),
        },

        Mutation::SetTraverseStepSize(v) => RuntimeRequestKind::SetConfigProperty {
            target: ident,
            path: "traverse.step_size".to_string(),
            value: ScalarValue::Float(v),
        },

        Mutation::SetTraversePadding(v) => RuntimeRequestKind::SetConfigProperty {
            target: ident,
            path: "traverse.padding".to_string(),
            value: ScalarValue::Float(v),
        },

        // ------------------------------------------------------------
        // Traverse commands
        // ------------------------------------------------------------
        Mutation::GotoTraverseLimitOuter => RuntimeRequestKind::ExecuteCommand {
            target: ident,
            path: "traverse.goto_limit_outer".to_string(),
        },

        Mutation::GotoTraverseLimitInner => RuntimeRequestKind::ExecuteCommand {
            target: ident,
            path: "traverse.goto_limit_inner".to_string(),
        },

        Mutation::GotoTraverseHome => RuntimeRequestKind::ExecuteCommand {
            target: ident,
            path: "traverse.goto_home".to_string(),
        },

        Mutation::EnableTraverseLaserpointer(enable) => RuntimeRequestKind::ExecuteCommand {
            target: ident,
            path: if enable {
                "laser_pointer.enable".to_string()
            } else {
                "laser_pointer.disable".to_string()
            },
        },

        // ------------------------------------------------------------
        // Puller config
        // ------------------------------------------------------------
        Mutation::SetPullerRegulationMode(v) => RuntimeRequestKind::SetConfigProperty {
            target: ident,
            path: "puller.speed_controller.algorithm".to_string(),
            value: ScalarValue::Enum(v.to_string()),
        },

        Mutation::SetPullerTargetSpeed(v) => RuntimeRequestKind::SetConfigProperty {
            target: ident,
            path: "puller.speed_controller.speed_desired".to_string(),
            value: ScalarValue::Float(v),
        },

        Mutation::SetPullerTargetDiameter(v) => RuntimeRequestKind::SetConfigProperty {
            target: ident,
            path: "diameter.target".to_string(),
            value: ScalarValue::Float(v),
        },

        Mutation::SetPullerForward(forward) => RuntimeRequestKind::SetConfigProperty {
            target: ident,
            path: "puller.direction".to_string(),
            value: ScalarValue::Enum(if forward { "forward" } else { "reverse" }.to_string()),
        },

        Mutation::SetPullerGearRatio(v) => RuntimeRequestKind::SetConfigProperty {
            target: ident,
            path: "puller.gear_ratio".to_string(),
            value: ScalarValue::Enum(v.to_string()),
        },

        // ------------------------------------------------------------
        // Spool speed controller config
        // ------------------------------------------------------------
        Mutation::SetSpoolRegulationMode(v) => RuntimeRequestKind::SetConfigProperty {
            target: ident,
            path: "spool.speed_controller.algorithm".to_string(),
            value: ScalarValue::Enum(v.to_string()),
        },

        Mutation::SetSpoolMinMaxMinSpeed(v) => RuntimeRequestKind::SetConfigProperty {
            target: ident,
            path: "spool.speed_controller.speed_min".to_string(),
            value: ScalarValue::Float(v),
        },

        Mutation::SetSpoolMinMaxMaxSpeed(v) => RuntimeRequestKind::SetConfigProperty {
            target: ident,
            path: "spool.speed_controller.speed_max".to_string(),
            value: ScalarValue::Float(v),
        },

        Mutation::SetSpoolForward(forward) => RuntimeRequestKind::SetConfigProperty {
            target: ident,
            path: "spool.direction".to_string(),
            value: ScalarValue::Enum(if forward { "forward" } else { "reverse" }.to_string()),
        },

        Mutation::SetSpoolAdaptiveTensionTarget(v) => RuntimeRequestKind::SetConfigProperty {
            target: ident,
            path: "spool.speed_controller.adaptive.tension_target".to_string(),
            value: ScalarValue::Float(v),
        },

        Mutation::SetSpoolAdaptiveRadiusLearningRate(v) => RuntimeRequestKind::SetConfigProperty {
            target: ident,
            path: "spool.speed_controller.adaptive.radius_learning_rate".to_string(),
            value: ScalarValue::Float(v),
        },

        Mutation::SetSpoolAdaptiveMaxSpeedMultiplier(v) => RuntimeRequestKind::SetConfigProperty {
            target: ident,
            path: "spool.speed_controller.adaptive.max_speed_multiplier".to_string(),
            value: ScalarValue::Float(v),
        },

        Mutation::SetSpoolAdaptiveAccelerationFactor(v) => RuntimeRequestKind::SetConfigProperty {
            target: ident,
            path: "spool.speed_controller.adaptive.acceleration_factor".to_string(),
            value: ScalarValue::Float(v),
        },

        Mutation::SetSpoolAdaptiveDeaccelerationUrgencyMultiplier(v) => {
            RuntimeRequestKind::SetConfigProperty {
                target: ident,
                path: "spool.speed_controller.adaptive.deacceleration_urgency_multiplier"
                    .to_string(),
                value: ScalarValue::Float(v),
            }
        }

        // ------------------------------------------------------------
        // Spool automatic config (not in runtime schema — no-op)
        // ------------------------------------------------------------
        Mutation::SetSpoolAutomaticRequiredMeters(v) => RuntimeRequestKind::SetConfigProperty {
            target: ident,
            path: "spool_automatic.required_meters".to_string(),
            value: ScalarValue::Float(v),
        },

        Mutation::SetSpoolAutomaticAction(v) => RuntimeRequestKind::SetConfigProperty {
            target: ident,
            path: "spool_automatic.action".to_string(),
            value: ScalarValue::Enum(v.to_string()),
        },

        // ------------------------------------------------------------
        // Spool commands
        // ------------------------------------------------------------
        Mutation::ResetSpoolProgress => RuntimeRequestKind::ExecuteCommand {
            target: ident,
            path: "spool.reset_progress".to_string(),
        },

        // ------------------------------------------------------------
        // Tension arm commands
        // ------------------------------------------------------------
        Mutation::ZeroTensionArmAngle => RuntimeRequestKind::ExecuteCommand {
            target: ident,
            path: "tension_arm.set_zero".to_string(),
        },

        // ------------------------------------------------------------
        // Mode commands
        // ------------------------------------------------------------
        Mutation::SetMode(v) => RuntimeRequestKind::ExecuteCommand {
            target: ident,
            path: format!("mode.{}", v),
        },

        // ------------------------------------------------------------
        // Puller adaptive config
        // ------------------------------------------------------------
        Mutation::SetPullerAdaptiveMaxSpeedChangePercent(v) => {
            RuntimeRequestKind::SetConfigProperty {
                target: ident,
                path: "puller.speed_controller.adaptive.speed_delta_max".to_string(),
                value: ScalarValue::Float(v),
            }
        }

        Mutation::SetPullerAdaptiveAdjustmentIntervalMeters(v) => {
            RuntimeRequestKind::SetConfigProperty {
                target: ident,
                path: "puller.speed_controller.adaptive.adjustment_distance".to_string(),
                value: ScalarValue::Float(v),
            }
        }

        Mutation::SetPullerAdaptiveStepPercent(v) => RuntimeRequestKind::SetConfigProperty {
            target: ident,
            path: "puller.speed_controller.adaptive.increase_per_step".to_string(),
            value: ScalarValue::Float(v),
        },

        Mutation::SetPullerAdaptiveAcceptedDifference(v) => RuntimeRequestKind::SetConfigProperty {
            target: ident,
            path: "puller.speed_controller.adaptive.tolerance_limit".to_string(),
            value: ScalarValue::Float(v),
        },

        Mutation::SetPullerAdaptiveReferenceMachine(v) => {
            match v.map(MachineInstanceIdentification::from) {
                Some(provider) => RuntimeRequestKind::SubscribeMachine {
                    provider,
                    subscriber: ident,
                },
                None => unreachable!("oops"),
            }
        }
    })
}

#[derive(Deserialize)]
#[allow(clippy::enum_variant_names)]
enum Mutation {
    // Traverse config
    SetTraverseLimitOuter(f64),
    SetTraverseLimitInner(f64),
    SetTraverseStepSize(f64),
    SetTraversePadding(f64),

    // Traverse commands
    GotoTraverseLimitOuter,
    GotoTraverseLimitInner,
    GotoTraverseHome,
    EnableTraverseLaserpointer(bool),

    // Puller config
    SetPullerRegulationMode(PullerRegulationMode),
    SetPullerTargetSpeed(f64),
    SetPullerTargetDiameter(f64),
    SetPullerForward(bool),
    SetPullerGearRatio(GearRatio),

    // Spool speed controller config
    SetSpoolRegulationMode(SpoolSpeedControllerType),
    SetSpoolMinMaxMinSpeed(f64),
    SetSpoolMinMaxMaxSpeed(f64),
    SetSpoolForward(bool),

    SetSpoolAdaptiveTensionTarget(f64),
    SetSpoolAdaptiveRadiusLearningRate(f64),
    SetSpoolAdaptiveMaxSpeedMultiplier(f64),
    SetSpoolAdaptiveAccelerationFactor(f64),
    SetSpoolAdaptiveDeaccelerationUrgencyMultiplier(f64),

    // Spool automatic config
    SetSpoolAutomaticRequiredMeters(f64),
    SetSpoolAutomaticAction(SpoolAutomaticActionMode),

    // Spool commands
    ResetSpoolProgress,

    // Tension arm command
    ZeroTensionArmAngle,

    // Mode config
    SetMode(Mode),

    // Puller adaptive config
    SetPullerAdaptiveMaxSpeedChangePercent(f64),
    SetPullerAdaptiveAdjustmentIntervalMeters(f64),
    SetPullerAdaptiveStepPercent(f64),
    SetPullerAdaptiveAcceptedDifference(f64),
    SetPullerAdaptiveReferenceMachine(Option<MachineIdentificationUnique>),
}

#[derive(Deserialize, Debug, Clone, Default)]
pub enum PullerRegulationMode {
    #[default]
    Speed,
    Diameter,
}

#[derive(Deserialize, Debug, Clone, Copy, PartialEq)]
#[allow(clippy::enum_variant_names)]
pub enum GearRatio {
    OneToOne,
    OneToFive,
    OneToTen,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub enum SpoolSpeedControllerType {
    #[default]
    Adaptive,
    MinMax,
}

#[derive(Deserialize, Debug, Clone, Default)]
pub enum Mode {
    #[default]
    Standby,
    Hold,
    Pull,
    Wind,
}

#[derive(Deserialize, Debug, Clone, Default)]
pub enum SpoolAutomaticActionMode {
    #[default]
    NoAction,
    Pull,
    Hold,
}

// --- Display impls: output runtime schema enum values ---

impl std::fmt::Display for PullerRegulationMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PullerRegulationMode::Speed => write!(f, "direct"),
            PullerRegulationMode::Diameter => write!(f, "adaptive"),
        }
    }
}

impl fmt::Display for GearRatio {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GearRatio::OneToOne => write!(f, "one_to_one"),
            GearRatio::OneToFive => write!(f, "one_to_five"),
            GearRatio::OneToTen => write!(f, "one_to_ten"),
        }
    }
}

impl fmt::Display for SpoolSpeedControllerType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SpoolSpeedControllerType::Adaptive => write!(f, "adaptive"),
            SpoolSpeedControllerType::MinMax => write!(f, "min_max"),
        }
    }
}

impl fmt::Display for Mode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Mode::Standby => write!(f, "standby"),
            Mode::Hold => write!(f, "hold"),
            Mode::Pull => write!(f, "pull"),
            Mode::Wind => write!(f, "wind"),
        }
    }
}

impl fmt::Display for SpoolAutomaticActionMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SpoolAutomaticActionMode::NoAction => write!(f, "no_action"),
            SpoolAutomaticActionMode::Pull => write!(f, "pull"),
            SpoolAutomaticActionMode::Hold => write!(f, "hold"),
        }
    }
}
