use crate::dryer::DryerMachine;
use crate::dryer::api::LiveValuesEvent;
use crate::{
    MACHINE_DRYER_SMART, MachineApi, MachineHardware, MachineMessage, MachineNew, QiTechMachine,
    VENDOR_QITECH,
};
use anyhow::Error;
use control_core::socketio::namespace::Namespace;
use qitech_lib::machines::{
    Machine, MachineDataRegistry, MachineError, MachineIdentification, MachineIdentificationUnique,
};
use serde_json::Value;
use tokio::sync::mpsc::Sender;

/// The Smart variant is the exact same implementation as `DryerMachine` - `DryerDevice`
/// already knows whether the connected unit is Smart (see `is_smart`), and `DryerMachine`
/// carries the Smart-only fields/behavior unconditionally (harmless no-ops on V1 hardware).
/// This type exists only so V1 and Smart keep separate machine identities/REST routes, per
/// the physical hardware IDs (0x0010 / 0x0012) - not because the logic itself differs.
pub struct DryerSmartMachine(DryerMachine);

impl DryerSmartMachine {
    pub const MACHINE_IDENTIFICATION: MachineIdentification = MachineIdentification {
        vendor: VENDOR_QITECH,
        machine: MACHINE_DRYER_SMART,
    };

    pub fn get_live_values(&self) -> LiveValuesEvent {
        self.0.get_live_values()
    }
}

impl MachineNew for DryerSmartMachine {
    fn new(hw: MachineHardware) -> Result<Self, Error> {
        Ok(Self(DryerMachine::new(hw)?))
    }
}

impl Machine for DryerSmartMachine {
    fn act(&mut self, reg: Option<&mut MachineDataRegistry>) -> Result<(), MachineError> {
        self.0.act(reg)
    }

    fn react(&mut self, registry: &MachineDataRegistry) {
        self.0.react(registry)
    }

    fn get_identification(&self) -> MachineIdentificationUnique {
        self.0.get_identification()
    }
}

impl MachineApi for DryerSmartMachine {
    fn api_mutate(&mut self, request_body: Value) -> Result<(), anyhow::Error> {
        self.0.api_mutate(request_body)
    }

    fn api_event_namespace(&mut self) -> Option<Namespace> {
        self.0.api_event_namespace()
    }

    fn get_api_sender(&self) -> Sender<MachineMessage> {
        self.0.get_api_sender()
    }

    fn act_machine_message(&mut self, msg: MachineMessage) {
        self.0.act_machine_message(msg)
    }
}

impl QiTechMachine for DryerSmartMachine {}
