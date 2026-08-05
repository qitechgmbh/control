use crate::{MACHINE_DRYER_V1, QiTechMachine, VENDOR_QITECH};
use api::{DryerEvents, DryerMachineNamespace, LiveValuesEvent, StateEvent};
use control_core::socketio::namespace::NamespaceCacheingLogic;
use core::DryerCore;
use qitech_lib::machines::{MachineIdentification, MachineIdentificationUnique};
use tokio::sync::mpsc::{Receiver, Sender};

pub mod act;
pub mod api;
pub mod core;
pub mod device;
pub mod material_presets;
pub mod new;

pub struct DryerMachine {
    api_receiver: Receiver<crate::MachineMessage>,
    api_sender: Sender<crate::MachineMessage>,
    machine_identification_unique: MachineIdentificationUnique,
    namespace: DryerMachineNamespace,

    core: DryerCore,
}

impl DryerMachine {
    pub const MACHINE_IDENTIFICATION: MachineIdentification = MachineIdentification {
        vendor: VENDOR_QITECH,
        machine: MACHINE_DRYER_V1,
    };

    pub fn get_live_values(&self) -> LiveValuesEvent {
        LiveValuesEvent {
            status: self.core.status,
            temp_process: self.core.temp_process,
            temp_safety: self.core.temp_safety,
            temp_regen_in: self.core.temp_regen_in,
            temp_regen_out: self.core.temp_regen_out,
            temp_fan_inlet: self.core.temp_fan_inlet,
            temp_return_air: self.core.temp_return_air,
            temp_dew_point: self.core.temp_dew_point,
            pwm_fan1: self.core.pwm_fan1,
            pwm_fan2: self.core.pwm_fan2,
            power_process: self.core.power_process,
            power_regen: self.core.power_regen,
            alarm: self.core.alarm,
            warning: self.core.warning,
            target_temperature: self.core.target_temperature,
            schedule: self.core.schedule,
            drying_timer_minutes: self.core.drying_timer_minutes,
        }
    }

    pub fn emit_live_values(&mut self) {
        if !self.core.received_data {
            return;
        }
        let event = self.get_live_values().build();
        self.namespace.emit(DryerEvents::LiveValues(event));
    }

    pub fn get_state(&self) -> StateEvent {
        StateEvent {
            is_default_state: !self.core.received_data,
        }
    }
}

impl QiTechMachine for DryerMachine {}
