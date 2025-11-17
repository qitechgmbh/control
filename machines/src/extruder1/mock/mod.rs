#[cfg(feature = "mock-machine")]
use smol::channel::Receiver;

#[cfg(feature = "mock-machine")]
use smol::channel::Sender;

#[cfg(feature = "mock-machine")]
use crate::{AsyncThreadMessage, Machine};

#[cfg(feature = "mock-machine")]
use crate::{
    MachineIdentificationUnique, MachineMessage, machine_identification::MachineIdentification,
};
#[cfg(feature = "mock-machine")]
use std::time::Instant;

#[cfg(feature = "mock-machine")]
use crate::{
    MACHINE_EXTRUDER_V1, VENDOR_QITECH,
    extruder1::{
        ExtruderV2Mode,
        api::{
            ExtruderSettingsState, ExtruderV2Namespace, HeatingStates, InverterStatusState,
            ModeState, MotorStatusValues, PidSettingsStates, PressureState, RegulationState,
            RotationState, ScrewState,
        },
    },
};

// Just checking mock-machine feature here to exclude these modules from compilation entirely
#[cfg(feature = "mock-machine")]
pub mod act;
#[cfg(feature = "mock-machine")]
pub mod api;
#[cfg(feature = "mock-machine")]
pub mod mock_emit;
#[cfg(feature = "mock-machine")]
pub mod new;

#[cfg(feature = "mock-machine")]
impl Machine for ExtruderV2 {
    fn get_machine_identification_unique(&self) -> MachineIdentificationUnique {
        self.machine_identification_unique.clone()
    }

    fn get_main_sender(&self) -> Option<Sender<AsyncThreadMessage>> {
        self.main_sender.clone()
    }
}

#[cfg(feature = "mock-machine")]
#[derive(Debug)]
pub struct ExtruderV2 {
    api_receiver: Receiver<MachineMessage>,
    api_sender: Sender<MachineMessage>,
    main_sender: Option<Sender<AsyncThreadMessage>>,

    machine_identification_unique: MachineIdentificationUnique,

    namespace: ExtruderV2Namespace,
    last_measurement_emit: Instant,
    last_status_hash: Option<u64>,
    mode: ExtruderV2Mode,
    /// Energy tracking for total consumption calculation
    pub total_energy_kwh: f64,
    pub last_energy_calculation_time: Option<Instant>,
    /// will be initalized as false and set to true by `emit_state`
    /// This way we can signal to the client that the first state emission is a default state
    emitted_default_state: bool,
    pub target_pressure: f64,

    pub is_default_state: bool,
    /// rotation state
    pub rotation_state: RotationState,
    /// mode state
    pub mode_state: ModeState,
    /// regulation state
    pub regulation_state: RegulationState,
    /// pressure state
    pub pressure_state: PressureState,
    /// screw state
    pub screw_state: ScrewState,
    /// heating states
    pub heating_states: HeatingStates,
    /// extruder settings state
    pub extruder_settings_state: ExtruderSettingsState,
    /// inverter status state
    pub inverter_status_state: InverterStatusState,
    /// pid settings
    pub pid_settings: PidSettingsStates,

    pub motor_status: MotorStatusValues,
    /// pressure in bar
    pub pressure: f64,
    /// nozzle temperature in celsius
    pub nozzle_temperature: f64,
    /// front temperature in celsius
    pub front_temperature: f64,
    /// back temperature in celsius
    pub back_temperature: f64,
    /// middle temperature in celsius
    pub middle_temperature: f64,
    /// nozzle heating power in watts
    pub nozzle_power: f64,
    /// front heating power in watts
    pub front_power: f64,
    /// back heating power in watts
    pub back_power: f64,
    /// middle heating power in watts
    pub middle_power: f64,
    /// combined power consumption in watts
    pub combined_power: f64,

    pub nozzle_heating_allowed: bool,

    pub front_heating_allowed: bool,

    pub back_heating_allowed: bool,

    pub middle_heating_allowed: bool,
}

#[cfg(feature = "mock-machine")]
impl ExtruderV2 {
    pub const MACHINE_IDENTIFICATION: MachineIdentification = MachineIdentification {
        vendor: VENDOR_QITECH,
        machine: MACHINE_EXTRUDER_V1,
    };
}
