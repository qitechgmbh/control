use api::{ExtruderV2Events, ExtruderV2Namespace};
use chrono::DateTime;
use control_core::{
    actors::{
        analog_input_getter::AnalogInputGetter,
        mitsubishi_inverter_rs485::{MitsubishiControlRequests, MitsubishiInverterRS485Actor},
    },
    machines::Machine,
    modbus::ModbusRequest,
    socketio::namespace::NamespaceCacheingLogic,
};
use ethercat_hal::{devices::el3021::EL3021, io::analog_input::AnalogInput};
use serde::{Deserialize, Serialize};
use smol::lock::RwLock;
use std::sync::Arc;
pub mod act;
pub mod api;
pub mod new;

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub enum ExtruderV2Mode {
    Standby,
    Heat,
    Extrude,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct Heating {
    pub temperature: f32,
    pub heating: bool,
    pub target_temperature: f32,
}

pub enum HeatingType {
    Front,
    Back,
    Middle,
}

#[derive(Debug)]
pub struct ExtruderV2 {
    inverter: MitsubishiInverterRS485Actor,
    namespace: ExtruderV2Namespace,
    mode: ExtruderV2Mode,
    heating_front: Heating,
    heating_back: Heating,
    heating_middle: Heating,
    last_response_emit: DateTime<chrono::Utc>,
    pressure_sensor: AnalogInputGetter,
    uses_rpm: bool,
}

impl std::fmt::Display for ExtruderV2 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ExtruderV2")
    }
}

impl Machine for ExtruderV2 {}

impl ExtruderV2 {
    fn set_rotation_state(&mut self, forward: bool) {
        if forward {
            let req: ModbusRequest = MitsubishiControlRequests::StartForwardRotation.into();
            self.inverter.add_request(req);
        } else {
            let req: ModbusRequest = MitsubishiControlRequests::StartReverseRotation.into();
            self.inverter.add_request(req);
        }
        self.emit_rotation_state(forward);
    }

    fn emit_rotation_state(&mut self, forward: bool) {
        let event = api::RotationStateEvent { forward: forward }.build();
        self.namespace
            .emit_cached(ExtruderV2Events::RotationStateEvent(event))
    }

    fn set_mode_state(&mut self, mode: ExtruderV2Mode) {
        self.mode = mode.clone();
        self.emit_mode_state(mode);
    }

    fn emit_mode_state(&mut self, mode: ExtruderV2Mode) {
        let event = api::ModeEvent { mode: mode }.build();
        self.namespace
            .emit_cached(ExtruderV2Events::ModeEvent(event));
    }

    // Heating
    fn set_heating_front(&mut self, heating: Heating) {
        self.heating_front = heating.clone();
        self.emit_heating(heating, HeatingType::Front);
    }

    fn set_heating_back(&mut self, heating: Heating) {
        self.heating_back = heating.clone();
        self.emit_heating(heating, HeatingType::Back);
    }

    fn set_heating_middle(&mut self, heating: Heating) {
        self.heating_middle = heating.clone();
        self.emit_heating(heating, HeatingType::Middle);
    }

    fn emit_heating(&mut self, heating: Heating, heating_type: HeatingType) {
        let event = api::HeatingStateEvent {
            temperature: heating.temperature,
            heating: heating.heating,
            target_temperature: heating.target_temperature,
        }
        .build(heating_type);
    }
}

// Motor
impl ExtruderV2 {
    fn set_regulation(&mut self, uses_rpm: bool) {
        self.uses_rpm = uses_rpm.clone();
        self.emit_regulation(uses_rpm);
    }

    fn emit_regulation(&mut self, uses_rpm: bool) {
        let event = api::RegulationStateEvent { uses_rpm: uses_rpm }.build();
    }
}
