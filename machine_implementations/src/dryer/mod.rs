use crate::{MACHINE_DRYER_V1, QiTechMachine, VENDOR_QITECH};
use api::{DryerEvents, DryerMachineNamespace, LiveValuesEvent, StateEvent};
use control_core::socketio::namespace::NamespaceCacheingLogic;
use device::{
    DryerDevice, SmartData, SmartTimerEntry, WeeklySchedule, is_running_status,
    local_weekday_and_minutes,
};
use material_presets::MATERIAL_PRESETS;
use qitech_lib::machines::{MachineError, MachineIdentification, MachineIdentificationUnique};
use qitech_lib::modbus::ModbusDevice;
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

    dryer: Rc<RefCell<DryerDevice>>,
    received_data: bool,
    last_emit: Instant,

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

    /// When the device last transitioned into a running state; None while not running.
    /// Drives the drying-timer auto-stop below - tracked here (not just in the frontend)
    /// so the timer keeps working even if no UI client is connected.
    running_since: Option<Instant>,
    /// Configured drying-timer duration; only applies on days with no scheduled stop time.
    drying_timer_minutes: u32,

    // Smart-only; stays default/empty on non-Smart hardware since DryerDevice never
    // populates smart_data for a unit that isn't Smart. is_smart is purely runtime state
    // (probed from the device, see DryerDevice::is_smart) - there's no separate machine
    // type for it.
    smart_data: SmartData,
    smart_data_write_ts: Option<Instant>,
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
            drying_timer_minutes: self.drying_timer_minutes,
            is_smart: self.dryer.borrow().is_smart,
            smart_data: self.smart_data.clone(),
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

    /// Polls the modbus actor; returns an error only on a genuine device-gone failure.
    pub fn poll_device(&mut self) -> Result<(), MachineError> {
        let mut dryer = self.dryer.borrow_mut();
        if let Err(e) = dryer.handle_response() {
            return Err(MachineError::IrrecoverableFailure(e.to_string()));
        }
        if let Err(e) = dryer.send_next_request() {
            return Err(MachineError::IrrecoverableFailure(e.to_string()));
        }
        Ok(())
    }

    /// Returns true (and resets the timer) once per second, gating the update/emit work
    /// in `act()` so it doesn't run on every real-time tick.
    pub fn tick_due(&mut self, now: Instant) -> bool {
        if now.duration_since(self.last_emit) > Duration::from_secs(1) {
            self.last_emit = now;
            true
        } else {
            false
        }
    }

    /// Updates temp/status/schedule from the device, then layers the Smart-only
    /// `smart_data` (firmware version, timer table) read-back on top. The latter is a
    /// no-op on non-Smart hardware, since `DryerDevice::smart_data` stays default when
    /// `!is_smart`.
    pub fn update(&mut self) {
        let dryer = self.dryer.borrow();
        if let Some(d) = &dryer.data {
            self.received_data = true;
            let was_running = is_running_status(self.status);
            self.status = d.status;
            let now_running = is_running_status(self.status);
            if !was_running && now_running {
                self.running_since = Some(Instant::now());
            } else if was_running && !now_running {
                self.running_since = None;
            }
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
        drop(dryer);

        let smart_write_settled = self
            .smart_data_write_ts
            .is_none_or(|ts| ts.elapsed() > Duration::from_secs(5));
        if smart_write_settled {
            self.smart_data = self.dryer.borrow().smart_data.clone();
        }
    }

    /// COIL_START_STOP is a pulse that toggles the device, not a setpoint - only queue
    /// the pulse if the device isn't already in the requested state, otherwise a
    /// redundant call (e.g. a scheduled stop firing while already stopped) would
    /// actually flip the device to the opposite state instead of doing nothing.
    pub fn set_start_stop(&mut self, running: bool) {
        if is_running_status(self.status) != running {
            self.dryer.borrow_mut().queue_set_start_stop();
        }
    }

    pub fn set_drying_timer_minutes(&mut self, minutes: u32) {
        self.drying_timer_minutes = minutes;
    }

    /// Runs every tick from `act()` so the dryer stops on schedule even with no UI
    /// connected. A scheduled stop time today takes priority over the drying timer,
    /// matching the frontend's own precedence (the timer UI is hidden whenever a
    /// schedule applies).
    pub fn check_auto_stop(&mut self) {
        if !is_running_status(self.status) {
            return;
        }

        let (weekday, now_minutes) = local_weekday_and_minutes();
        let scheduled_stop = self.schedule[weekday as usize].stop_time;
        if scheduled_stop != 0 {
            let stop_minutes = (scheduled_stop / 100) as u32 * 60 + (scheduled_stop % 100) as u32;
            if now_minutes >= stop_minutes {
                self.set_start_stop(false);
            }
            return;
        }

        if let Some(started) = self.running_since {
            let target = Duration::from_secs(self.drying_timer_minutes as u64 * 60);
            if started.elapsed() >= target {
                self.set_start_stop(false);
            }
        }
    }

    pub fn set_target_temperature(&mut self, temp_celsius: f64) {
        self.target_temperature = temp_celsius;
        self.target_temp_write_ts = Some(Instant::now());
        self.dryer
            .borrow_mut()
            .queue_set_target_temperature(temp_celsius);
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
        if index as u16 >= self.dryer.borrow().smart_timer_slots() {
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
        self.dryer
            .borrow_mut()
            .queue_write_timer_entry(index, entry);
    }

    pub fn write_new_timer_entry(&mut self, entry: SmartTimerEntry) {
        self.smart_data.timer_entries.push(entry);
        self.smart_data_write_ts = Some(Instant::now());
        self.dryer.borrow_mut().queue_write_new_timer_entry(entry);
    }

    pub fn delete_timer_entry(&mut self, index: u8) {
        if index as u16 >= self.dryer.borrow().smart_timer_slots() {
            tracing::warn!("dryer timer index {index} out of bounds, ignoring delete");
            return;
        }
        let idx = index as usize;
        if idx < self.smart_data.timer_entries.len() {
            self.smart_data.timer_entries.remove(idx);
        }
        self.smart_data_write_ts = Some(Instant::now());
        self.dryer.borrow_mut().queue_delete_timer_entry(index);
    }
}

impl QiTechMachine for DryerMachine {}
