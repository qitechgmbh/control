use crate::machines::extruder1::{HeatingType, api::Mutation, mock::ExtruderV2};
use control_core::machines::api::MachineApi;
use control_core::socketio::namespace::Namespace;
use smol::lock::Mutex;
use std::sync::Arc;

impl MachineApi for ExtruderV2 {
    fn api_mutate(&mut self, request_body: serde_json::Value) -> Result<(), anyhow::Error> {
        // there are multiple Modbus Frames that are "prebuilt"
        let control: Mutation = serde_json::from_value(request_body)?;
        match control {
            Mutation::SetExtruderMode(mode) => self.set_mode_state(mode),
            Mutation::SetInverterRotationDirection(forward) => self.set_rotation_state(forward),
            Mutation::SetInverterRegulation(uses_rpm) => self.set_regulation(uses_rpm),
            Mutation::SetInverterTargetPressure(bar) => self.set_target_pressure(bar),
            Mutation::SetInverterTargetRpm(rpm) => self.set_target_rpm(rpm),
            Mutation::ResetInverter(_) => (),
            Mutation::SetFrontHeatingTargetTemperature(temp) => {
                self.set_target_temperature(temp, HeatingType::Front)
            }
            Mutation::SetMiddleHeatingTemperature(temp) => {
                self.set_target_temperature(temp, HeatingType::Middle)
            }
            Mutation::SetBackHeatingTargetTemperature(temp) => {
                self.set_target_temperature(temp, HeatingType::Back)
            }
            Mutation::SetNozzleHeatingTemperature(temp) => {
                self.set_target_temperature(temp, HeatingType::Nozzle)
            }
            Mutation::SetExtruderPressureLimit(pressure_limit) => {
                self.set_nozzle_pressure_limit(pressure_limit);
            }
            Mutation::SetExtruderPressureLimitIsEnabled(enabled) => {
                self.set_nozzle_pressure_limit_is_enabled(enabled);
            }

            Mutation::SetPressurePidSettings(settings) => {
                self.configure_pressure_pid(settings);
            }

            Mutation::SetTemperaturePidSettings(settings) => {
                self.configure_temperature_pid(settings);
            }
        }
        Ok(())
    }

    fn api_event_namespace(&mut self) -> Arc<Mutex<Namespace>> {
        self.namespace.namespace.clone()
    }

    fn api_query(&mut self, fields: &[String]) -> Result<serde_json::Value, anyhow::Error> {
        let live_values = super::super::api::LiveValuesEvent {
            motor_status: Default::default(),
            pressure: 0.0,
            nozzle_temperature: 0.0,
            front_temperature: 0.0,
            back_temperature: 0.0,
            middle_temperature: 0.0,
            nozzle_power: 0.0,
            front_power: 0.0,
            back_power: 0.0,
            middle_power: 0.0,
            combined_power: 0.0,
            total_energy_kwh: 0.0,
        };

        let state = self.build_state_event();

        let full_data = serde_json::json!({
            "live_values": live_values,
            "state": state,
        });

        // Filter based on requested fields
        crate::rest::field_filter::filter_fields(full_data, fields)
    }
}
