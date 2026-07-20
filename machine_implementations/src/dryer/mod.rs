use crate::{MACHINE_DRYER_V1, QiTechMachine, VENDOR_QITECH};
use api::{DryerEvents, DryerMachineNamespace, LiveValuesEvent, StateEvent};
use control_core::socketio::namespace::NamespaceCacheingLogic;
use device::{DryerDevice, WeeklySchedule};
use material_presets::MATERIAL_PRESETS;
use qitech_lib::machines::{MachineIdentification, MachineIdentificationUnique};
use std::cell::RefCell;
use std::rc::Rc;
use std::time::{Duration, Instant};
use tokio::sync::mpsc::{Receiver, Sender};

pub mod act;
pub mod api;
pub mod device;
pub mod material_presets;
pub mod new;

pub struct DryerMachine {
    api_receiver: Receiver<crate::MachineMessage>,
    api_sender: Sender<crate::MachineMessage>,
    machine_identification_unique: MachineIdentificationUnique,
    namespace: DryerMachineNamespace,
    last_emit: Instant,
    received_data: bool,

    dryer: Rc<RefCell<DryerDevice>>,

    status: u16,
    temp_process: f64,
    temp_safety: f64,
    temp_regen_in: f64,
    temp_regen_out: f64,
    temp_fan_inlet: f64,
    temp_return_air: f64,
    temp_dew_point: f64,
    pwm_fan1: f64,
    pwm_fan2: f64,
    power_process: f64,
    power_regen: f64,
    alarm: u16,
    warning: u16,
    target_temperature: f64,
    schedule: WeeklySchedule,
    /// Set when a SetSchedule write is in flight; suppresses device read-back for 5s
    schedule_write_ts: Option<Instant>,
    /// Set when a SetTargetTemperature write is in flight; suppresses device read-back for 3s
    target_temp_write_ts: Option<Instant>,
}

impl DryerMachine {
    pub const MACHINE_IDENTIFICATION: MachineIdentification = MachineIdentification {
        vendor: VENDOR_QITECH,
        machine: MACHINE_DRYER_V1,
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
            schedule: self.schedule,
        }
    }

    pub fn emit_live_values(&mut self) {
        if !self.received_data {
            return;
        }
        let event = self.get_live_values().build();
        self.namespace.emit(DryerEvents::LiveValues(event));
    }

    pub fn get_state(&self) -> StateEvent {
        StateEvent {
            is_default_state: !self.received_data,
        }
    }

    pub fn update(&mut self) {
        let dryer = self.dryer.borrow();
        if let Some(d) = &dryer.data {
            self.received_data = true;
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

            let temp_write_settled = self
                .target_temp_write_ts
                .is_none_or(|ts| ts.elapsed() > Duration::from_secs(3));
            if temp_write_settled {
                self.target_temperature = d.target_temperature;
            }

            let schedule_write_settled = self
                .schedule_write_ts
                .is_none_or(|ts| ts.elapsed() > Duration::from_secs(5));
            if schedule_write_settled {
                self.schedule = d.schedule;
            }
        }
    }

    pub fn set_start_stop(&mut self) {
        self.dryer.borrow_mut().queue_set_start_stop();
    }

    pub fn set_target_temperature(&mut self, temp_celsius: f64) {
        self.target_temperature = temp_celsius;
        self.target_temp_write_ts = Some(Instant::now());
        self.dryer.borrow_mut().queue_set_target_temperature(temp_celsius);
    }

    pub fn set_schedule(&mut self, schedule: WeeklySchedule) {
        self.schedule = schedule;
        self.schedule_write_ts = Some(Instant::now());
        self.dryer.borrow_mut().queue_set_schedule(schedule);
    }

    pub fn apply_material_preset(&mut self, abbrev: &str, throughput_kg_per_h: f64) {
        match MATERIAL_PRESETS.iter().find(|p| p.abbrev == abbrev) {
            Some(preset) => {
                self.target_temp_write_ts = Some(Instant::now());
                let temp = self
                    .dryer
                    .borrow_mut()
                    .queue_apply_material_preset(preset, throughput_kg_per_h);
                self.target_temperature = temp as f64;
            }
            None => tracing::warn!("Unknown dryer material preset abbrev: {abbrev}"),
        }
    }
}

impl QiTechMachine for DryerMachine {}
