use super::ExtruderV2;
use crate::{
    MachineApi,
    extruder1::{HeatingType, api::Mutation},
};

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

    fn api_event_namespace(
        &mut self,
    ) -> std::option::Option<control_core::socketio::namespace::Namespace> {
        self.namespace.namespace.clone()
    }

    fn api_get_sender(&self) -> smol::channel::Sender<crate::MachineMessage> {
        self.api_sender.clone()
    }
}
