use crate::dryer::core::DryerCore;
use crate::dryer::device::SmartTimerEntry;
use crate::{MACHINE_DRYER_SMART, QiTechMachine, VENDOR_QITECH};
use api::{DryerSmartEvents, DryerSmartMachineNamespace, LiveValuesEvent, StateEvent};
use control_core::socketio::namespace::NamespaceCacheingLogic;
use qitech_lib::machines::{MachineIdentification, MachineIdentificationUnique};
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

    core: DryerCore,

    smart_data: crate::dryer::device::SmartData,
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
            smart_data: self.smart_data.clone(),
            drying_timer_minutes: self.core.drying_timer_minutes,
        }
    }

    pub fn emit_live_values(&mut self) {
        if !self.core.received_data {
            return;
        }
        let event = self.get_live_values().build();
        self.namespace.emit(DryerSmartEvents::LiveValues(event));
    }

    pub fn get_state(&self) -> StateEvent {
        StateEvent {
            is_default_state: !self.core.received_data,
        }
    }

    /// Runs the shared temp/status/schedule update, then layers the Smart-only
    /// `smart_data` (firmware version, timer table) read-back on top.
    pub fn update(&mut self) {
        self.core.update();

        let smart_write_settled = self
            .smart_data_write_ts
            .is_none_or(|ts| ts.elapsed() > Duration::from_secs(5));
        if smart_write_settled {
            self.smart_data = self.core.dryer.borrow().smart_data.clone();
        }
    }

    pub fn sync_system_clock(&mut self) {
        self.core.dryer.borrow_mut().queue_sync_clock();
    }

    pub fn set_timer_enabled(&mut self, enabled: bool) {
        self.smart_data.timer_enabled = enabled;
        self.smart_data_write_ts = Some(Instant::now());
        self.core
            .dryer
            .borrow_mut()
            .queue_set_timer_enabled(enabled);
    }

    pub fn write_timer_entry(&mut self, index: u8, entry: SmartTimerEntry) {
        if index as u16 >= self.core.dryer.borrow().smart_timer_slots() {
            tracing::warn!("dryer timer index {index} out of bounds, ignoring write");
            return;
        }
        let idx = index as usize;
        while self.smart_data.timer_entries.len() <= idx {
            self.smart_data
                .timer_entries
                .push(SmartTimerEntry::default());
        }
        self.smart_data.timer_entries[idx] = entry;
        self.smart_data_write_ts = Some(Instant::now());
        self.core
            .dryer
            .borrow_mut()
            .queue_write_timer_entry(index, entry);
    }

    pub fn write_new_timer_entry(&mut self, entry: SmartTimerEntry) {
        self.smart_data.timer_entries.push(entry);
        self.smart_data_write_ts = Some(Instant::now());
        self.core
            .dryer
            .borrow_mut()
            .queue_write_new_timer_entry(entry);
    }

    pub fn delete_timer_entry(&mut self, index: u8) {
        if index as u16 >= self.core.dryer.borrow().smart_timer_slots() {
            tracing::warn!("dryer timer index {index} out of bounds, ignoring delete");
            return;
        }
        let idx = index as usize;
        if idx < self.smart_data.timer_entries.len() {
            self.smart_data.timer_entries.remove(idx);
        }
        self.smart_data_write_ts = Some(Instant::now());
        self.core.dryer.borrow_mut().queue_delete_timer_entry(index);
    }
}

impl QiTechMachine for DryerSmartMachine {}
