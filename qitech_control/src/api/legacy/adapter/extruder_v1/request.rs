use qitech_framework::MachineInstanceIdentification;
use qitech_framework::RuntimeRequestKind;
use qitech_framework::ScalarValue;
use serde::Deserialize;

use crate::machines::Zone;

pub fn convert_request(
    ident: MachineInstanceIdentification,
    data: serde_json::Value,
) -> Result<Vec<RuntimeRequestKind>, serde_json::Error> {
    let config = |path: &str, value: ScalarValue| RuntimeRequestKind::SetConfigProperty {
        target: ident,
        path: path.to_string(),
        value,
    };

    let command = |path: &str| RuntimeRequestKind::ExecuteCommand {
        target: ident,
        path: path.to_string(),
    };

    if let Ok(mutation) = serde_json::from_value::<CompoundMutation>(data.clone()) {
        return Ok(match mutation {
            CompoundMutation::SetPressurePidSettings { kp, ki, kd } => vec![
                config("pid.pressure.kp", ScalarValue::Float(kp)),
                config("pid.pressure.ki", ScalarValue::Float(ki)),
                config("pid.pressure.kd", ScalarValue::Float(kd)),
            ],

            CompoundMutation::SetTemperaturePidSettings { kp, ki, kd, zone } => {
                let gains = zone.paths().gains;
                vec![
                    config(gains.kp, ScalarValue::Float(kp)),
                    config(gains.ki, ScalarValue::Float(ki)),
                    config(gains.kd, ScalarValue::Float(kd)),
                ]
            }

            // The two config writes must precede the start command: the runtime reads them
            // synchronously when the command fires.
            CompoundMutation::StartPressurePidAutoTune {
                tune_delta,
                frequency_step_hz,
            } => vec![
                config(
                    "pressure.autotune.tune_delta",
                    ScalarValue::Float(tune_delta),
                ),
                config(
                    "pressure.autotune.frequency_step",
                    ScalarValue::Float(frequency_step_hz),
                ),
                command("pressure.autotune.start"),
            ],
        });
    }

    Ok(vec![match serde_json::from_value(data)? {
        // ------------------------------------------------------------
        // Screw config
        // ------------------------------------------------------------
        //
        // `EnumProperty::from_scalar` only accepts the snake_case spelling of a variant, even
        // though it reads back as the variant ident itself. See `init_state_event`.
        Mutation::SetInverterRotationDirection(forward) => config(
            "screw.direction",
            ScalarValue::Enum(if forward { "forward" } else { "reverse" }.to_string()),
        ),

        Mutation::SetInverterRegulation(uses_rpm) => config(
            "screw.regulation",
            ScalarValue::Enum(if uses_rpm { "rpm" } else { "pressure" }.to_string()),
        ),

        Mutation::SetInverterTargetRpm(v) => config("screw.target_rpm", ScalarValue::Float(v)),

        Mutation::SetInverterTargetPressure(v) => {
            config("screw.target_pressure", ScalarValue::Float(v))
        }

        // ------------------------------------------------------------
        // Heating config
        // ------------------------------------------------------------
        Mutation::SetNozzleHeatingTemperature(v) => {
            config("heating.nozzle.target_temperature", ScalarValue::Float(v))
        }

        Mutation::SetFrontHeatingTargetTemperature(v) => {
            config("heating.front.target_temperature", ScalarValue::Float(v))
        }

        Mutation::SetMiddleHeatingTemperature(v) => {
            config("heating.middle.target_temperature", ScalarValue::Float(v))
        }

        Mutation::SetBackHeatingTargetTemperature(v) => {
            config("heating.back.target_temperature", ScalarValue::Float(v))
        }

        Mutation::SetNozzleTemperatureTargetEnabled(v) => {
            config("heating.nozzle.target_enabled", ScalarValue::Boolean(v))
        }

        // ------------------------------------------------------------
        // Pressure config
        // ------------------------------------------------------------
        Mutation::SetExtruderPressureLimit(v) => config("pressure.limit", ScalarValue::Float(v)),

        Mutation::SetExtruderPressureLimitIsEnabled(v) => {
            config("pressure.limit_enabled", ScalarValue::Boolean(v))
        }

        // ------------------------------------------------------------
        // Extrusion config
        // ------------------------------------------------------------
        Mutation::SetMinExtrusionTemperature(v) => {
            config("extrusion.min_temperature", ScalarValue::Float(v))
        }

        // ------------------------------------------------------------
        // Mode commands
        // ------------------------------------------------------------
        Mutation::SetExtruderMode(mode) => command(match mode {
            Mode::Standby => "mode.standby",
            Mode::Heat => "mode.heat",
            Mode::Extrude => "mode.extrude",
        }),

        // ------------------------------------------------------------
        // Inverter commands
        // ------------------------------------------------------------
        Mutation::ResetInverter(_) => command("inverter.reset"),

        // ------------------------------------------------------------
        // Pressure auto-tune commands
        // ------------------------------------------------------------
        Mutation::StopPressurePidAutoTune {} => command("pressure.autotune.stop"),
    }])
}

/// Mirrors the payloads emitted by `useExtruder.ts`.
#[derive(Deserialize)]
enum Mutation {
    // Screw config
    SetInverterRotationDirection(bool),
    SetInverterRegulation(bool),
    SetInverterTargetRpm(f64),
    SetInverterTargetPressure(f64),

    // Heating config
    SetNozzleHeatingTemperature(f64),
    SetFrontHeatingTargetTemperature(f64),
    SetMiddleHeatingTemperature(f64),
    SetBackHeatingTargetTemperature(f64),
    SetNozzleTemperatureTargetEnabled(bool),

    // Pressure config
    SetExtruderPressureLimit(f64),
    SetExtruderPressureLimitIsEnabled(bool),

    // Extrusion config
    SetMinExtrusionTemperature(f64),

    // Mode command
    SetExtruderMode(Mode),

    // Inverter command
    /// The frontend always sends `true` here; the flag carries no meaning, only the request does.
    ResetInverter(#[allow(dead_code)] bool),

    // Pressure auto-tune command
    StopPressurePidAutoTune {},
}

/// The three mutations that expand into more than one `RuntimeRequestKind`. Tried first; anything
/// else falls through to the single-request `Mutation` match.
#[derive(Deserialize)]
enum CompoundMutation {
    SetPressurePidSettings {
        kp: f64,
        ki: f64,
        kd: f64,
    },
    SetTemperaturePidSettings {
        kp: f64,
        ki: f64,
        kd: f64,
        zone: Zone,
    },
    StartPressurePidAutoTune {
        tune_delta: f64,
        frequency_step_hz: f64,
    },
}

#[derive(Deserialize)]
enum Mode {
    Standby,
    Heat,
    Extrude,
}
