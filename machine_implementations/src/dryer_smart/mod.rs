use crate::dryer::device::{DryerDevice, SmartData, SmartTimerEntry, WeeklySchedule};
use crate::dryer::material_presets::MATERIAL_PRESETS;
use crate::{MACHINE_DRYER_SMART, QiTechMachine, VENDOR_QITECH};
use api::{DryerSmartEvents, DryerSmartMachineNamespace, LiveValuesEvent, StateEvent};
use control_core::socketio::namespace::NamespaceCacheingLogic;
use qitech_lib::machines::{MachineIdentification, MachineIdentificationUnique};
use std::cell::RefCell;
use std::rc::Rc;
use std::time::{Duration, Instant};
use tokio::sync::mpsc::{Receiver, Sender};

pub mod act;
pub mod api;
pub mod new;

pub struct DryerSmartMachine {
    api_receiver: Receiver<crate::MachineMessage>,
    api_sender: Sender<crate::MachineMessage>,
    machine_identification_unique: MachineIdentificationUnique,
    namespace: DryerSmartMachineNamespace,
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
    schedule_write_ts: Option<Instant>,
    target_temp_write_ts: Option<Instant>,
    smart_data: SmartData,
    /// Set when a timer-entry write is in flight; suppresses smart_data read-back for 5s
    smart_data_write_ts: Option<Instant>,
}

impl DryerSmartMachine {
    pub const MACHINE_IDENTIFICATION: MachineIdentification = MachineIdentification {
        vendor: VENDOR_QITECH,
        machine: MACHINE_DRYER_SMART,
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
            smart_data: self.smart_data.clone(),
        }
    }

    pub fn emit_live_values(&mut self) {
        if !self.received_data {
            return;
        }
        let event = self.get_live_values().build();
        self.namespace.emit(DryerSmartEvents::LiveValues(event));
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

        let smart_write_settled = self
            .smart_data_write_ts
            .is_none_or(|ts| ts.elapsed() > Duration::from_secs(5));
        if smart_write_settled {
            self.smart_data = dryer.smart_data.clone();
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

    pub fn sync_system_clock(&mut self) {
        self.dryer.borrow_mut().queue_sync_clock();
    }

    pub fn set_timer_enabled(&mut self, enabled: bool) {
        self.smart_data.timer_enabled = enabled;
        self.smart_data_write_ts = Some(Instant::now());
        self.dryer.borrow_mut().queue_set_timer_enabled(enabled);
    }

    pub fn write_timer_entry(&mut self, index: u8, entry: SmartTimerEntry) {
        let idx = index as usize;
        while self.smart_data.timer_entries.len() <= idx {
            self.smart_data.timer_entries.push(SmartTimerEntry::default());
        }
        self.smart_data.timer_entries[idx] = entry;
        self.smart_data_write_ts = Some(Instant::now());
        self.dryer.borrow_mut().queue_write_timer_entry(index, entry);
    }

    pub fn write_new_timer_entry(&mut self, entry: SmartTimerEntry) {
        self.smart_data.timer_entries.push(entry);
        self.smart_data_write_ts = Some(Instant::now());
        self.dryer.borrow_mut().queue_write_new_timer_entry(entry);
    }

    pub fn delete_timer_entry(&mut self, index: u8) {
        let idx = index as usize;
        if idx < self.smart_data.timer_entries.len() {
            self.smart_data.timer_entries.remove(idx);
        }
        self.smart_data_write_ts = Some(Instant::now());
        self.dryer.borrow_mut().queue_delete_timer_entry(index);
    }
}

impl QiTechMachine for DryerSmartMachine {}
