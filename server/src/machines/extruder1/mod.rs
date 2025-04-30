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
use serde::Serialize;
use smol::lock::RwLock;
use std::sync::Arc;
pub mod act;
pub mod api;
pub mod new;

#[derive(Serialize, Debug, Clone, PartialEq)]
pub enum ExtruderV2Mode {
    Standby,
    Heat,
    Extrude,
}

#[derive(Debug)]
pub struct ExtruderV2 {
    inverter: MitsubishiInverterRS485Actor,
    namespace: ExtruderV2Namespace,
    mode: ExtruderV2Mode,
    last_response_emit: DateTime<chrono::Utc>,
    pressure_sensor: AnalogInputGetter,
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
        self.mode = match mode {
            ExtruderV2Mode::Standby => ExtruderV2Mode::Standby,
            ExtruderV2Mode::Heat => ExtruderV2Mode::Heat,
            ExtruderV2Mode::Extrude => ExtruderV2Mode::Extrude,
        };
    }

    fn emit_mode_state(&mut self) {
        //let event = api::ModeEvent;
        //self.namespace.emit_cached(event);
    }
}
