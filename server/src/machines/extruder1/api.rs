use super::ExtruderV2;
use control_core::{
    machines::api::MachineApi,
    socketio::{
        event::{Event, GenericEvent},
        namespace::{
            cache_duration, cache_one_event, CacheFn, CacheableEvents, Namespace,
            NamespaceCacheingLogic, NamespaceInterface,
        },
    },
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Debug, Clone)]
pub struct FrequencyEvent {
    frequency: f32,
    // is this the Frequency in the eeprom or the one in memory(running)
    is_ram: bool,
}

#[derive(Serialize, Debug, Clone)]
pub struct MotorStateEvent {
    start: bool,
    forward_rotation: bool,
}

#[derive(Serialize, Debug, Clone)]
pub struct OperationModeEvent {
    operation_mode: u8,
    mode_name: String,
}

/// Inverter status Register 40009
// bit 8-14 is unused
#[derive(Serialize, Debug, Clone)]
pub struct InverterStatusEvent {
    /// RUN (Inverter running)
    running: bool,
    /// Forward running motor spins forward
    forward_running: bool,
    /// Reverse running motor spins backwards
    reverse_running: bool,
    /// Up to frequency, SU not completely sure what its for
    up_to_frequency: bool,
    /// overload warning OL
    overload_warning: bool,
    /// No function, its described that way in the datasheet
    no_function: bool,
    /// FU Output Frequency Detection
    output_frequency_detection: bool,
    /// ABC (Fault)
    abc_fault: bool,
    /// is True when a fault occured
    fault_occurence: bool,
}

/// This is used when we just need a simple confirmation, that what we did, didnt cause errors
#[derive(Serialize, Debug, Clone)]
pub struct InverterSuccessEvent {
    success: bool,
}

pub enum ExtruderV2Events {
    InverterStateEvent(Event<InverterStatusEvent>),
    InverterModeEvent(Event<OperationModeEvent>),
    InverterErrorEvent(),
    InverterFrequencyEvent(Event<FrequencyEvent>),
    InverterSuccessEvent(),
}

#[derive(Deserialize, Serialize)]
enum Mutation {
    /// Frequency Control
    /// Set
    SetRunningFrequency(f32),
    SetEepromFrequency(f32),
    SetMinimumFrequency(f32),
    SetMaximumFrequency(f32),
    ///Get
    GetRunningFrequency(),
    GetEepromFrequency(),
    GetMaximumFrequency(),
    GetMinimumFrequency(),

    /// Motor Control
    // true is forward rotation, false reverse rotation
    // Set Rotation also starts the motor
    SetRotation(bool),
    StopMotor(),

    /// Inverter Control
    SetOperationMode(u8),
    WriteParameter(u16, u16),
    ReadParameter(u16),

    // Clears
    ClearAllParameters(),
    ClearParameter(),
    ClearNonCommunicationParameter(),
    ClearNonCommunicationParameters(),
}

#[derive(Debug)]
pub struct ExtruderV2Namespace(Namespace);

impl ExtruderV2Namespace {
    pub fn new() -> Self {
        Self(Namespace::new())
    }
}

impl MachineApi for ExtruderV2 {
    fn api_mutate(&mut self, request_body: Value) -> Result<(), anyhow::Error> {
        // there are multiple Modbus Frames that are "prebuilt"
        let control: Mutation = serde_json::from_value(request_body)?;
        match control {
            Mutation::SetRunningFrequency(frequency) => todo!(),
            Mutation::SetEepromFrequency(frequency) => todo!(),
            Mutation::SetMinimumFrequency(frequency) => todo!(),
            Mutation::SetMaximumFrequency(frequency) => todo!(),
            Mutation::SetRotation(forward) => todo!(),
            Mutation::StopMotor() => todo!(),
            Mutation::SetOperationMode(operation_mode) => todo!(),
            Mutation::WriteParameter(register, value) => todo!(),
            Mutation::ReadParameter(register) => todo!(),
            Mutation::ClearAllParameters() => todo!(),
            Mutation::ClearParameter() => todo!(),
            Mutation::ClearNonCommunicationParameter() => todo!(),
            Mutation::ClearNonCommunicationParameters() => todo!(),
            Mutation::GetRunningFrequency() => todo!(),
            Mutation::GetEepromFrequency() => todo!(),
            Mutation::GetMaximumFrequency() => todo!(),
            Mutation::GetMinimumFrequency() => todo!(),
        }
        Ok(())
    }

    fn api_event_namespace(&mut self) -> &mut dyn NamespaceInterface {
        &mut self.namespace.0
    }
}
