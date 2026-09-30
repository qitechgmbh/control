use std::fmt;

use qitech_framework::EnumProperty;
use qitech_framework::Machine;
use qitech_framework::machine::ActResult;
use qitech_framework::machine::ConfigProperty;
use qitech_framework::machine::EventEmitter;
use qitech_framework::machine::StateProperty;
use qitech_framework::units::ThermodynamicTemperature;
use qitech_framework::units::thermodynamic_temperature::degree_celsius;
use serde::Deserialize;
use serde::Serialize;

use crate::machines::aquapath::api::NoticeEvent;
use crate::machines::aquapath::controller::ControlResetReason;
use crate::machines::aquapath::controller::ControllerNotice;
use crate::machines::aquapath::reservoir::Reservoir;

pub mod act;
pub mod api;
pub mod controller;
pub mod new;
pub mod reservoir;

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, Default, EnumProperty)]
pub enum AquapathV1Mode {
    #[default]
    Standby,
    Auto,
}

impl fmt::Display for AquapathV1Mode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

#[derive(Machine)]
pub struct AquapathV1 {
    mode: AquapathV1Mode,
    mode_state: StateProperty<AquapathV1Mode>,
    ambient_temperature_calibration: ThermodynamicTemperature,
    ambient_temperature_calibration_config: ConfigProperty<f64>,
    left: Reservoir,
    right: Reservoir,
    notice_event_emitter: EventEmitter<NoticeEvent>,
}

impl fmt::Display for AquapathV1 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "AquapathV1")
    }
}

impl AquapathV1 {
    pub const DEFAULT_PID_KP: f64 = 0.16;
    pub const DEFAULT_PID_KI: f64 = 0.02;
    pub const DEFAULT_PID_KD: f64 = 0.0;

    fn reservoirs_mut(&mut self) -> [&mut Reservoir; 2] {
        [&mut self.left, &mut self.right]
    }

    fn switch_to_standby(&mut self) -> ActResult {
        if self.mode == AquapathV1Mode::Auto {
            for reservoir in self.reservoirs_mut() {
                reservoir.enter_standby();
            }
        }
        self.mode = AquapathV1Mode::Standby;
        Ok(())
    }

    fn switch_to_auto(&mut self) -> ActResult {
        if self.mode == AquapathV1Mode::Standby {
            for reservoir in self.reservoirs_mut() {
                reservoir.enter_auto();
            }
        }
        self.mode = AquapathV1Mode::Auto;
        Ok(())
    }

    /// Thermal safety timing may only be retuned while nothing is running.
    fn allows_safety_config_changes(&self) -> bool {
        self.mode == AquapathV1Mode::Standby
    }

    /// Targets may not be set below ambient: the loop has no way to cool beneath it.
    /// Both reservoirs share the same hard limits, so either one can supply them.
    fn min_settable_temperature(&self) -> ThermodynamicTemperature {
        let ambient = self.ambient_temperature_calibration.get::<degree_celsius>();
        let min = self.left.min_temperature().get::<degree_celsius>();
        let max = self.left.max_temperature().get::<degree_celsius>();
        ThermodynamicTemperature::new::<degree_celsius>(ambient.max(min).min(max))
    }

    fn set_ambient_temperature_calibration(&mut self, ambient_temperature: f64) {
        let clamped = ambient_temperature
            .max(self.left.min_temperature().get::<degree_celsius>())
            .min(self.left.max_temperature().get::<degree_celsius>());
        self.ambient_temperature_calibration =
            ThermodynamicTemperature::new::<degree_celsius>(clamped);

        // Enforce the new minimum on targets that were already configured.
        let min_settable = self.min_settable_temperature();
        for reservoir in self.reservoirs_mut() {
            reservoir.raise_target_to(min_settable);
        }
    }

    fn emit_pending_notices(&mut self) {
        let pending: Vec<_> = self
            .reservoirs_mut()
            .into_iter()
            .flat_map(|reservoir| {
                let label = reservoir.label();
                reservoir
                    .drain_notices()
                    .into_iter()
                    .map(move |notice| (label, notice))
            })
            .collect();

        for (label, notice) in pending {
            self.emit_controller_notice(label, notice);
        }
    }

    fn emit_controller_notice(&mut self, side_label: &str, notice: ControllerNotice) {
        let (title, message) = match notice {
            ControllerNotice::ControlReset(reason) => {
                let message = match reason {
                    ControlResetReason::TargetTemperatureChanged => {
                        "Target temperature changed. PID control state and heater PWM timing were reset."
                    }
                    ControlResetReason::HeatingToleranceChanged => {
                        "Heating tolerance changed. PID control state and heater PWM timing were reset."
                    }
                    ControlResetReason::CoolingToleranceChanged => {
                        "Cooling tolerance changed. PID control state and heater PWM timing were reset."
                    }
                    ControlResetReason::PidParametersChanged => {
                        "PID settings changed. PID control state and heater PWM timing were reset."
                    }
                    ControlResetReason::PumpCommandChanged => {
                        "Pump command changed. PID control state and heater PWM timing were reset."
                    }
                };
                (format!("{side_label}: Thermal Control Reset"), message)
            }
            ControllerNotice::PumpStoppedLowFlow => (
                format!("{side_label}: Pump Turned Off"),
                "Flow fell below the minimum thermal threshold while the pump was enabled. The pump was turned off and PID control state was reset.",
            ),
        };

        self.notice_event_emitter.emit(&NoticeEvent {
            title,
            message: message.to_owned(),
        });
    }
}
