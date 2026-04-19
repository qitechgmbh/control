use self::api::{DrywellEvents, DrywellMachineNamespace, LiveValuesEvent};
use crate::serial::devices::drywell::Drywell;
use crate::{
    AsyncThreadMessage, MACHINE_DRYWELL_V1, Machine, MachineMessage, VENDOR_QITECH,
    machine_identification::{MachineIdentification, MachineIdentificationUnique},
};
use control_core::socketio::namespace::NamespaceCacheingLogic;
use smol::channel::{Receiver, Sender};
use smol::lock::RwLock;
use std::sync::Arc;
use std::time::Instant;

pub mod act;
pub mod api;
pub mod new;

#[derive(Debug)]
pub struct DrywellMachine {
    pub api_receiver: Receiver<MachineMessage>,
    pub api_sender: Sender<MachineMessage>,
    pub machine_identification_unique: MachineIdentificationUnique,
    pub main_sender: Option<Sender<AsyncThreadMessage>>,
    pub namespace: DrywellMachineNamespace,
    pub last_emit: Instant,

    // driver
    pub drywell: Arc<RwLock<Drywell>>,

    // live values
    pub status: u16,
    pub temp_process: f64,
    pub temp_safety: f64,
    pub temp_regen_in: f64,
    pub temp_regen_out: f64,
    pub temp_fan_inlet: f64,
    pub temp_return_air: f64,
    pub temp_dew_point: f64,
    pub pwm_fan1: f64,
    pub pwm_fan2: f64,
    pub power_process: f64,
    pub power_regen: f64,
    pub alarm: u16,
    pub warning: u16,
    pub target_temperature: f64,
}

impl Machine for DrywellMachine {
    fn get_machine_identification_unique(&self) -> MachineIdentificationUnique {
        self.machine_identification_unique.clone()
    }

    fn get_main_sender(&self) -> Option<Sender<AsyncThreadMessage>> {
        self.main_sender.clone()
    }
}

impl DrywellMachine {
    pub const MACHINE_IDENTIFICATION: MachineIdentification = MachineIdentification {
        vendor: VENDOR_QITECH,
        machine: MACHINE_DRYWELL_V1,
    };

    pub fn get_live_values(&self) -> LiveValuesEvent {
        LiveValuesEvent {
            status: self.status,
            temp_process: self.temp_process,
            temp_safety: self.temp_safety,
            temp_regen_in: self.temp_regen_in,
            temp_regen_out: self.temp_regen_out,
            temp_fan_inlet: self.temp_fan_inlet,
            temp_return_air: self.temp_return_air,
            temp_dew_point: self.temp_dew_point,
            pwm_fan1: self.pwm_fan1,
            pwm_fan2: self.pwm_fan2,
            power_process: self.power_process,
            power_regen: self.power_regen,
            alarm: self.alarm,
            warning: self.warning,
            target_temperature: self.target_temperature,
        }
    }

    pub fn emit_live_values(&mut self) {
        let event = self.get_live_values().build();
        self.namespace.emit(DrywellEvents::LiveValues(event));
    }

    pub fn update(&mut self) {
        let data = smol::block_on(async { self.drywell.read().await.get_data().await });

        if let Some(d) = data {
            self.status = d.status;
            self.temp_process = d.temp_process;
            self.temp_safety = d.temp_safety;
            self.temp_regen_in = d.temp_regen_in;
            self.temp_regen_out = d.temp_regen_out;
            self.temp_fan_inlet = d.temp_fan_inlet;
            self.temp_return_air = d.temp_return_air;
            self.temp_dew_point = d.temp_dew_point;
            self.pwm_fan1 = d.pwm_fan1;
            self.pwm_fan2 = d.pwm_fan2;
            self.power_process = d.power_process;
            self.power_regen = d.power_regen;
            self.alarm = d.alarm;
            self.warning = d.warning;
            self.target_temperature = d.target_temperature;
        }
    }

    pub fn set_start_stop(&mut self) {
        let res = smol::block_on(async { self.drywell.read().await.set_start_stop().await });
        if let Err(e) = res {
            tracing::warn!("Drywell set_start_stop failed: {}", e);
        }
    }

    pub fn set_target_temperature(&mut self, temp_celsius: f64) {
        self.target_temperature = temp_celsius;
        let res = smol::block_on(async {
            self.drywell
                .read()
                .await
                .set_target_temperature(temp_celsius)
                .await
        });
        if let Err(e) = res {
            tracing::warn!("Drywell set_target_temperature failed: {}", e);
        }
    }
}
