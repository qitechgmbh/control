use super::device::{DryerDevice, WeeklySchedule, is_running_status, local_weekday_and_minutes};
use super::material_presets::MATERIAL_PRESETS;
use qitech_lib::machines::MachineError;
use qitech_lib::modbus::ModbusDevice;
use std::cell::RefCell;
use std::rc::Rc;
use std::time::{Duration, Instant};

/// State and behavior shared by `DryerMachine` (V1) and `DryerSmartMachine`, which both
/// drive the same physical device and differ only in the Smart timer-table extras layered
/// on top.
pub struct DryerCore {
    pub dryer: Rc<RefCell<DryerDevice>>,
    pub received_data: bool,
    pub last_emit: Instant,

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
    pub schedule: WeeklySchedule,
    /// Set when a SetSchedule write is in flight; suppresses device read-back for 5s
    pub schedule_write_ts: Option<Instant>,
    /// Set when a SetTargetTemperature write is in flight; suppresses device read-back for 3s
    pub target_temp_write_ts: Option<Instant>,

    /// When the device last transitioned into a running state; None while not running.
    /// Drives the drying-timer auto-stop below - tracked here (not just in the frontend)
    /// so the timer keeps working even if no UI client is connected.
    pub running_since: Option<Instant>,
    /// Configured drying-timer duration; only applies on days with no scheduled stop time.
    pub drying_timer_minutes: u32,
}

impl DryerCore {
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
}
