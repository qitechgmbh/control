use crate::{
    MachineNewHardware, MachineNewParams, MachineNewTrait,
    extruder1::{
        ExtruderV2Mode,
        api::{
            ExtruderSettingsState, ExtruderV2Namespace, HeatingState, HeatingStates,
            InverterStatusState, ModeState, MotorStatusValues, PidSettings, PidSettingsStates,
            PressureState, RegulationState, RotationState, ScrewState, TemperaturePid,
            TemperaturePidStates,
        },
        mock::ExtruderV2,
    },
};

impl MachineNewTrait for ExtruderV2 {
    fn new(params: &MachineNewParams<'_, '_, '_, '_, '_, '_, '_>) -> Result<Self, anyhow::Error>
    where
        Self: Sized,
    {
        // Mock machine can work with either Serial or Ethercat hardware
        // For the mock machine, we don't need to actually use the hardware
        // We just validate that we have the expected hardware type
        match params.hardware {
            MachineNewHardware::Serial(_) => {
                // For serial mode, we could potentially use the serial device if needed
                // but for a mock machine, we'll just note it and proceed
            }
            MachineNewHardware::Ethercat(_) => {
                // For ethercat mode, we could potentially use the ethercat devices
                // but for a mock machine, we'll just note it and proceed
            }
        }

        let now = std::time::Instant::now();
        let (sender, receiver) = smol::channel::unbounded();

        let mut extruder_mock_machine = Self {
            main_sender: params.main_thread_channel.clone(),
            api_receiver: receiver,
            api_sender: sender,
            machine_identification_unique: params.get_machine_identification_unique(),
            namespace: ExtruderV2Namespace {
                namespace: params.namespace.clone(),
            },
            last_measurement_emit: now,

            mode: ExtruderV2Mode::Standby, // Start in standby mode
            emitted_default_state: false,
            last_status_hash: None,
            total_energy_kwh: 0.0,
            last_energy_calculation_time: None,
            is_default_state: false,
            rotation_state: RotationState { forward: false },
            mode_state: ModeState {
                mode: ExtruderV2Mode::Standby,
            },
            regulation_state: RegulationState { uses_rpm: true },
            pressure_state: PressureState {
                target_bar: 0.0,
                wiring_error: false,
            },
            screw_state: ScrewState { target_rpm: 0.0 },
            heating_states: HeatingStates {
                nozzle: HeatingState {
                    target_temperature: 0.0,
                    wiring_error: false,
                },
                front: HeatingState {
                    target_temperature: 0.0,
                    wiring_error: false,
                },
                back: HeatingState {
                    target_temperature: 0.0,
                    wiring_error: false,
                },
                middle: HeatingState {
                    target_temperature: 0.0,
                    wiring_error: false,
                },
            },
            extruder_settings_state: ExtruderSettingsState {
                pressure_limit: 200.0,
                pressure_limit_enabled: false,
            },
            inverter_status_state: InverterStatusState {
                running: false,
                forward_running: true,
                reverse_running: true,
                up_to_frequency: false,
                overload_warning: false,
                no_function: false,
                output_frequency_detection: false,
                abc_fault: false,
                fault_occurence: false,
            },
            pid_settings: PidSettingsStates {
                temperature: TemperaturePidStates {
                    front: TemperaturePid {
                        ki: 0.1,
                        kp: 0.1,
                        kd: 0.1,
                        zone: String::from("front"),
                    },
                    middle: TemperaturePid {
                        ki: 0.1,
                        kp: 0.1,
                        kd: 0.1,
                        zone: String::from("middle"),
                    },
                    back: TemperaturePid {
                        ki: 0.1,
                        kp: 0.1,
                        kd: 0.1,
                        zone: String::from("back"),
                    },
                    nozzle: TemperaturePid {
                        ki: 0.1,
                        kp: 0.1,
                        kd: 0.1,
                        zone: String::from("nozzle"),
                    },
                },
                pressure: PidSettings {
                    ki: 0.0,
                    kp: 0.0,
                    kd: 0.0,
                },
            },
            motor_status: MotorStatusValues {
                screw_rpm: 0.0,
                frequency: 0.0,
                voltage: 0.0,
                current: 0.0,
                power: 0.0,
            },
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
            nozzle_heating_allowed: false,
            front_heating_allowed: false,
            back_heating_allowed: false,
            middle_heating_allowed: false,
            target_pressure: 0.0,
        };

        extruder_mock_machine.emit_state();

        Ok(extruder_mock_machine)
    }
}
