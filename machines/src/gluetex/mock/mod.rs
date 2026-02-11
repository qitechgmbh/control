#[cfg(feature = "gluetex-mock")]
pub mod act;
#[cfg(feature = "gluetex-mock")]
pub mod api;
#[cfg(feature = "gluetex-mock")]
pub mod mock_emit;
#[cfg(feature = "gluetex-mock")]
pub mod new;

#[cfg(feature = "gluetex-mock")]
use super::api::{
    AddonMotor5State, AddonMotorState, AddonMotorTensionControlState, GluetexNamespace,
    HeatingPidStates, HeatingStates, LiveValuesEvent, ModeState, OrderInfoState, PullerState,
    SleepTimerState, SpoolAutomaticActionState, SpoolSpeedControllerState, StateEvent,
    TensionArmMonitorState, TensionArmState, TraverseState, ValveState, VoltageMonitorState,
};
#[cfg(feature = "gluetex-mock")]
use crate::machine_identification::{MachineIdentification, MachineIdentificationUnique};
#[cfg(feature = "gluetex-mock")]
use crate::{
    AsyncThreadMessage, MACHINE_GLUETEX_V1, Machine, MachineConnection,
    MachineCrossConnectionState, MachineMessage, VENDOR_QITECH,
};
#[cfg(feature = "gluetex-mock")]
use smol::channel::{Receiver, Sender};
#[cfg(feature = "gluetex-mock")]
use std::time::Instant;

#[cfg(feature = "gluetex-mock")]
#[derive(Debug)]
pub struct Gluetex {
    machine_identification_unique: MachineIdentificationUnique,
    namespace: GluetexNamespace,
    last_measurement_emit: Instant,
    emitted_default_state: bool,
    status_out: bool,

    traverse_state: TraverseState,
    puller_state: PullerState,
    spool_automatic_action_state: SpoolAutomaticActionState,
    mode_state: ModeState,
    tension_arm_state: TensionArmState,
    spool_speed_controller_state: SpoolSpeedControllerState,
    heating_states: HeatingStates,
    heating_pid_settings: HeatingPidStates,
    connected_machine_state: MachineCrossConnectionState,
    addon_motor_3_state: AddonMotor5State,
    addon_motor_4_state: AddonMotorState,
    addon_motor_5_state: AddonMotorState,
    addon_motor_5_tension_control_state: AddonMotorTensionControlState,
    slave_puller_state: super::api::SlavePullerState,
    addon_tension_arm_state: TensionArmState,
    winder_tension_arm_monitor_state: TensionArmMonitorState,
    addon_tension_arm_monitor_state: TensionArmMonitorState,
    slave_tension_arm_monitor_state: TensionArmMonitorState,
    optris_1_monitor_state: VoltageMonitorState,
    optris_2_monitor_state: VoltageMonitorState,
    sleep_timer_state: SleepTimerState,
    order_info_state: OrderInfoState,
    valve_state: ValveState,

    live_values: LiveValuesEvent,

    api_receiver: Receiver<MachineMessage>,
    api_sender: Sender<MachineMessage>,
    main_sender: Option<Sender<AsyncThreadMessage>>,

    connected_machines: Vec<MachineConnection>,
    max_connected_machines: usize,
}

#[cfg(feature = "gluetex-mock")]
impl Machine for Gluetex {
    fn get_machine_identification_unique(&self) -> MachineIdentificationUnique {
        self.machine_identification_unique.clone()
    }

    fn get_main_sender(&self) -> Option<Sender<AsyncThreadMessage>> {
        self.main_sender.clone()
    }
}

#[cfg(feature = "gluetex-mock")]
impl Gluetex {
    pub const MACHINE_IDENTIFICATION: MachineIdentification = MachineIdentification {
        vendor: VENDOR_QITECH,
        machine: MACHINE_GLUETEX_V1,
    };

    pub fn build_state_event(&mut self) -> StateEvent {
        let is_default_state = !std::mem::replace(&mut self.emitted_default_state, true);
        StateEvent {
            is_default_state,
            status_out: self.status_out,
            traverse_state: self.traverse_state.clone(),
            puller_state: self.puller_state.clone(),
            spool_automatic_action_state: self.spool_automatic_action_state.clone(),
            mode_state: self.mode_state.clone(),
            tension_arm_state: self.tension_arm_state.clone(),
            spool_speed_controller_state: self.spool_speed_controller_state.clone(),
            heating_states: self.heating_states.clone(),
            heating_pid_settings: self.heating_pid_settings.clone(),
            connected_machine_state: self.connected_machine_state.clone(),
            addon_motor_3_state: self.addon_motor_3_state.clone(),
            addon_motor_4_state: self.addon_motor_4_state.clone(),
            addon_motor_5_state: self.addon_motor_5_state.clone(),
            addon_motor_5_tension_control_state: self.addon_motor_5_tension_control_state.clone(),
            slave_puller_state: self.slave_puller_state.clone(),
            addon_tension_arm_state: self.addon_tension_arm_state.clone(),
            winder_tension_arm_monitor_state: self.winder_tension_arm_monitor_state.clone(),
            addon_tension_arm_monitor_state: self.addon_tension_arm_monitor_state.clone(),
            slave_tension_arm_monitor_state: self.slave_tension_arm_monitor_state.clone(),
            optris_1_monitor_state: self.optris_1_monitor_state.clone(),
            optris_2_monitor_state: self.optris_2_monitor_state.clone(),
            sleep_timer_state: self.sleep_timer_state.clone(),
            order_info_state: self.order_info_state.clone(),
            valve_state: self.valve_state.clone(),
        }
    }

    pub fn build_live_values_event(&self) -> LiveValuesEvent {
        self.live_values.clone()
    }
}
