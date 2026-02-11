use super::Gluetex;
use crate::MachineCrossConnectionState;
use crate::gluetex::api::{
    AddonMotor5State, AddonMotorState, AddonMotorTensionControlState, GluetexNamespace,
    HeatingPidSettings, HeatingPidStates, HeatingStates, LiveValuesEvent, ModeState,
    OrderInfoState, PullerState, SleepTimerState, SpoolAutomaticActionState,
    SpoolSpeedControllerState, SlavePullerState, TensionArmMonitorState, TensionArmState,
    TraverseState, ValveState, VoltageMonitorState,
};
use crate::machine_identification::MachineIdentificationUnique;
use crate::{MachineNewHardware, MachineNewParams, MachineNewTrait};

impl MachineNewTrait for Gluetex {
    fn new(params: &MachineNewParams<'_, '_, '_, '_, '_, '_, '_>) -> Result<Self, anyhow::Error>
    where
        Self: Sized,
    {
        match params.hardware {
            MachineNewHardware::Serial(_) => (),
            MachineNewHardware::Ethercat(_) => (),
        }

        let (sender, receiver) = smol::channel::unbounded();
        let now = std::time::Instant::now();

        let live_values = LiveValuesEvent {
            traverse_position: Some(0.0),
            puller_speed: 0.0,
            spool_rpm: 0.0,
            tension_arm_angle: 0.0,
            spool_progress: 0.0,
            temperature_1: 0.0,
            temperature_2: 0.0,
            temperature_3: 0.0,
            temperature_4: 0.0,
            temperature_5: 0.0,
            temperature_6: 0.0,
            heater_1_power: 0.0,
            heater_2_power: 0.0,
            heater_3_power: 0.0,
            heater_4_power: 0.0,
            heater_5_power: 0.0,
            heater_6_power: 0.0,
            slave_puller_speed: 0.0,
            slave_tension_arm_angle: 0.0,
            addon_tension_arm_angle: 0.0,
            optris_1_voltage: 0.0,
            optris_2_voltage: 0.0,
        };

        let connected_machine_state = MachineCrossConnectionState {
            machine_identification_unique: None,
            is_available: false,
        };

        let machine_identification_unique: MachineIdentificationUnique =
            params.get_machine_identification_unique();

        let heating_pid_settings = HeatingPidStates {
            zone_1: HeatingPidSettings {
                zone: "zone_1".to_string(),
                ..Default::default()
            },
            zone_2: HeatingPidSettings {
                zone: "zone_2".to_string(),
                ..Default::default()
            },
            zone_3: HeatingPidSettings {
                zone: "zone_3".to_string(),
                ..Default::default()
            },
            zone_4: HeatingPidSettings {
                zone: "zone_4".to_string(),
                ..Default::default()
            },
            zone_5: HeatingPidSettings {
                zone: "zone_5".to_string(),
                ..Default::default()
            },
            zone_6: HeatingPidSettings {
                zone: "zone_6".to_string(),
                ..Default::default()
            },
        };

        let mut gluetex_mock_machine = Self {
            machine_identification_unique,
            namespace: GluetexNamespace {
                namespace: params.namespace.clone(),
            },
            last_measurement_emit: now,
            emitted_default_state: false,
            status_out: false,
            traverse_state: TraverseState {
                is_homed: true,
                can_go_in: true,
                can_go_out: true,
                can_go_home: true,
                ..Default::default()
            },
            puller_state: PullerState::default(),
            spool_automatic_action_state: SpoolAutomaticActionState::default(),
            mode_state: ModeState {
                can_wind: true,
                ..Default::default()
            },
            tension_arm_state: TensionArmState::default(),
            spool_speed_controller_state: SpoolSpeedControllerState::default(),
            heating_states: HeatingStates::default(),
            heating_pid_settings,
            connected_machine_state,
            addon_motor_3_state: AddonMotor5State::default(),
            addon_motor_4_state: AddonMotorState::default(),
            addon_motor_5_state: AddonMotorState::default(),
            addon_motor_5_tension_control_state: AddonMotorTensionControlState::default(),
            slave_puller_state: SlavePullerState::default(),
            addon_tension_arm_state: TensionArmState::default(),
            winder_tension_arm_monitor_state: TensionArmMonitorState::default(),
            addon_tension_arm_monitor_state: TensionArmMonitorState::default(),
            slave_tension_arm_monitor_state: TensionArmMonitorState::default(),
            optris_1_monitor_state: VoltageMonitorState::default(),
            optris_2_monitor_state: VoltageMonitorState::default(),
            sleep_timer_state: SleepTimerState::default(),
            order_info_state: OrderInfoState::default(),
            valve_state: ValveState::default(),
            live_values,
            api_receiver: receiver,
            api_sender: sender,
            main_sender: params.main_thread_channel.clone(),
            connected_machines: Vec::new(),
            max_connected_machines: 2,
        };

        gluetex_mock_machine.emit_state();

        Ok(gluetex_mock_machine)
    }
}
