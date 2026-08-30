use std::cell::RefCell;
use std::rc::Rc;
use std::time::{Duration, Instant};

use qitech_framework::Machine;
use qitech_framework::MachineIdentification;
use qitech_framework::machine::ActError;
use qitech_framework::machine::ActErrorImpact;
use qitech_framework::machine::ActErrorKind;
use qitech_framework::machine::ActResult;
use qitech_framework::machine::BuildContext;
use qitech_framework::machine::BuildResult;
use qitech_framework::machine::ConfigProperty;
use qitech_framework::machine::EventEmitter;
use qitech_framework::machine::Machine as MachineTrait;
use qitech_framework::machine::MachineBuild;
use qitech_framework::machine::Measurement;
use qitech_framework::machine::StateProperty;
use qitech_framework::machine_build;
use qitech_framework::vendors;
use qitech_lib::modbus::ModbusDevice;
use qitech_lib::units::ThermodynamicTemperature;
use qitech_lib::units::thermodynamic_temperature::degree_celsius;

mod device;
pub use device::DryerDevice;
use device::{ScheduleDay, WeeklySchedule, is_running_status, local_weekday_and_seconds};

#[derive(Machine)]
pub struct DryerV1 {
    // --- hardware ---
    dryer: Rc<RefCell<DryerDevice>>,

    // --- config ---
    target_temperature: ConfigProperty<ThermodynamicTemperature>,
    /// Manual air volume setpoint - raw device unit, no physical quantity documented yet.
    air_volume: ConfigProperty<i64>,
    /// Only used on days with no scheduled stop time.
    drying_timer_minutes: ConfigProperty<i64>,
    /// Minutes since local midnight, 0 = no scheduled action; index 0=Monday..6=Sunday.
    schedule_start: [ConfigProperty<i64>; 7],
    schedule_stop: [ConfigProperty<i64>; 7],

    // --- state ---
    status: StateProperty<i64>,
    alarm: StateProperty<i64>,
    warning: StateProperty<i64>,
    is_smart: StateProperty<bool>,

    // --- measurements ---
    temp_process: Measurement<ThermodynamicTemperature>,
    temp_safety: Measurement<ThermodynamicTemperature>,
    temp_regen_in: Measurement<ThermodynamicTemperature>,
    temp_regen_out: Measurement<ThermodynamicTemperature>,
    temp_fan_inlet: Measurement<ThermodynamicTemperature>,
    temp_return_air: Measurement<ThermodynamicTemperature>,
    temp_dew_point: Measurement<ThermodynamicTemperature>,
    pwm_fan1: Measurement<f64>,
    pwm_fan2: Measurement<f64>,
    power_process: Measurement<f64>,
    power_regen: Measurement<f64>,
    remaining_seconds: Measurement<Option<i64>>,

    // --- events ---
    alarm_raised: EventEmitter<()>,

    // --- misc ---
    /// When the device last transitioned into a running state; `None` while not running.
    /// Drives the drying-timer auto-stop - plain internal bookkeeping, not part of the
    /// schema (nothing external needs to read it directly, only `remaining_seconds`).
    running_since: Option<Instant>,
}

impl MachineBuild for DryerV1 {
    #[machine_build(DryerV1)]
    fn build(ctx: &mut BuildContext<'_>) -> BuildResult<Self> {
        let dryer = ctx.get_modbus_rtu_device::<DryerDevice>(0)?;

        let target_temperature = ctx
            .config::<degree_celsius>("target_temperature")
            .default(60.0)
            .minimum(50.0)
            .maximum(180.0)
            .on_external_changed(Self::push_target_temperature)
            .build()?;

        let air_volume = ctx
            .config::<i64>("air_volume")
            .default(10)
            .minimum(1)
            .on_external_changed(Self::push_air_volume)
            .build()?;

        let drying_timer_minutes = ctx
            .config::<i64>("drying_timer_minutes")
            .default(240)
            .minimum(1)
            .build()?;

        let schedule_start = [
            ctx.config::<i64>("schedule.monday.start_minutes")
                .default(0)
                .minimum(0)
                .maximum(1439)
                .on_external_changed(Self::push_schedule)
                .build()?,
            ctx.config::<i64>("schedule.tuesday.start_minutes")
                .default(0)
                .minimum(0)
                .maximum(1439)
                .on_external_changed(Self::push_schedule)
                .build()?,
            ctx.config::<i64>("schedule.wednesday.start_minutes")
                .default(0)
                .minimum(0)
                .maximum(1439)
                .on_external_changed(Self::push_schedule)
                .build()?,
            ctx.config::<i64>("schedule.thursday.start_minutes")
                .default(0)
                .minimum(0)
                .maximum(1439)
                .on_external_changed(Self::push_schedule)
                .build()?,
            ctx.config::<i64>("schedule.friday.start_minutes")
                .default(0)
                .minimum(0)
                .maximum(1439)
                .on_external_changed(Self::push_schedule)
                .build()?,
            ctx.config::<i64>("schedule.saturday.start_minutes")
                .default(0)
                .minimum(0)
                .maximum(1439)
                .on_external_changed(Self::push_schedule)
                .build()?,
            ctx.config::<i64>("schedule.sunday.start_minutes")
                .default(0)
                .minimum(0)
                .maximum(1439)
                .on_external_changed(Self::push_schedule)
                .build()?,
        ];

        let schedule_stop = [
            ctx.config::<i64>("schedule.monday.stop_minutes")
                .default(0)
                .minimum(0)
                .maximum(1439)
                .on_external_changed(Self::push_schedule)
                .build()?,
            ctx.config::<i64>("schedule.tuesday.stop_minutes")
                .default(0)
                .minimum(0)
                .maximum(1439)
                .on_external_changed(Self::push_schedule)
                .build()?,
            ctx.config::<i64>("schedule.wednesday.stop_minutes")
                .default(0)
                .minimum(0)
                .maximum(1439)
                .on_external_changed(Self::push_schedule)
                .build()?,
            ctx.config::<i64>("schedule.thursday.stop_minutes")
                .default(0)
                .minimum(0)
                .maximum(1439)
                .on_external_changed(Self::push_schedule)
                .build()?,
            ctx.config::<i64>("schedule.friday.stop_minutes")
                .default(0)
                .minimum(0)
                .maximum(1439)
                .on_external_changed(Self::push_schedule)
                .build()?,
            ctx.config::<i64>("schedule.saturday.stop_minutes")
                .default(0)
                .minimum(0)
                .maximum(1439)
                .on_external_changed(Self::push_schedule)
                .build()?,
            ctx.config::<i64>("schedule.sunday.stop_minutes")
                .default(0)
                .minimum(0)
                .maximum(1439)
                .on_external_changed(Self::push_schedule)
                .build()?,
        ];

        ctx.command("start").execute(Self::cmd_start).build()?;
        ctx.command("stop").execute(Self::cmd_stop).build()?;

        Ok(Self {
            dryer,
            target_temperature,
            air_volume,
            drying_timer_minutes,
            schedule_start,
            schedule_stop,
            status: ctx.state::<i64>("status").build()?,
            alarm: ctx.state::<i64>("alarm").build()?,
            warning: ctx.state::<i64>("warning").build()?,
            is_smart: ctx.state::<bool>("is_smart").build()?,
            temp_process: ctx.measurement::<degree_celsius>("temp_process").build()?,
            temp_safety: ctx.measurement::<degree_celsius>("temp_safety").build()?,
            temp_regen_in: ctx.measurement::<degree_celsius>("temp_regen_in").build()?,
            temp_regen_out: ctx
                .measurement::<degree_celsius>("temp_regen_out")
                .build()?,
            temp_fan_inlet: ctx
                .measurement::<degree_celsius>("temp_fan_inlet")
                .build()?,
            temp_return_air: ctx
                .measurement::<degree_celsius>("temp_return_air")
                .build()?,
            temp_dew_point: ctx
                .measurement::<degree_celsius>("temp_dew_point")
                .build()?,
            pwm_fan1: ctx.measurement::<f64>("pwm_fan1").build()?,
            pwm_fan2: ctx.measurement::<f64>("pwm_fan2").build()?,
            power_process: ctx.measurement::<f64>("power_process").build()?,
            power_regen: ctx.measurement::<f64>("power_regen").build()?,
            remaining_seconds: ctx
                .measurement::<Option<i64>>("remaining_seconds")
                .build()?,
            alarm_raised: ctx.event("alarm_raised").build()?,
            running_since: None,
        })
    }
}

impl MachineTrait for DryerV1 {
    fn act(&mut self, _dt: Duration) -> ActResult {
        self.poll_device()?;

        let Some(d) = self.dryer.borrow().data.clone() else {
            return Ok(());
        };

        let was_running = is_running_status(self.status.get() as u16);
        self.status.set(d.status as i64);
        let now_running = is_running_status(d.status);
        if !was_running && now_running {
            self.running_since = Some(Instant::now());
        } else if was_running && !now_running {
            self.running_since = None;
        }

        let alarm_changed = self.alarm.set(d.alarm as i64);
        if alarm_changed && d.alarm != 0 {
            self.alarm_raised.emit(&());
        }
        self.warning.set(d.warning as i64);
        self.is_smart.set(self.dryer.borrow().is_smart);

        self.temp_process
            .set(ThermodynamicTemperature::new::<degree_celsius>(
                d.temp_process,
            ));
        self.temp_safety
            .set(ThermodynamicTemperature::new::<degree_celsius>(
                d.temp_safety,
            ));
        self.temp_regen_in
            .set(ThermodynamicTemperature::new::<degree_celsius>(
                d.temp_regen_in,
            ));
        self.temp_regen_out
            .set(ThermodynamicTemperature::new::<degree_celsius>(
                d.temp_regen_out,
            ));
        self.temp_fan_inlet
            .set(ThermodynamicTemperature::new::<degree_celsius>(
                d.temp_fan_inlet,
            ));
        self.temp_return_air
            .set(ThermodynamicTemperature::new::<degree_celsius>(
                d.temp_return_air,
            ));
        self.temp_dew_point
            .set(ThermodynamicTemperature::new::<degree_celsius>(
                d.temp_dew_point,
            ));
        self.pwm_fan1.set(d.pwm_fan1);
        self.pwm_fan2.set(d.pwm_fan2);
        self.power_process.set(d.power_process);
        self.power_regen.set(d.power_regen);

        self.check_auto_stop();
        let remaining = self.compute_remaining_seconds();
        self.remaining_seconds.set(remaining);

        Ok(())
    }
}

impl DryerV1 {
    pub const IDENTIFICATION: MachineIdentification = MachineIdentification {
        vendor_id: vendors::QITECH.id,
        machine_id: 16,
    };

    /// Polls the modbus actor; returns an error only on a genuine device-gone failure.
    /// The device self-paces its own poll cycle (see `device.rs`), so this can be called
    /// unconditionally every tick without an external request timer.
    fn poll_device(&mut self) -> ActResult {
        let mut dryer = self.dryer.borrow_mut();

        if let Err(e) = dryer.handle_response() {
            return Err(ActError {
                kind: ActErrorKind::HardwareFault(e.to_string()),
                impact: ActErrorImpact::Irrecoverable,
            });
        }
        if let Err(e) = dryer.send_next_request() {
            return Err(ActError {
                kind: ActErrorKind::HardwareFault(e.to_string()),
                impact: ActErrorImpact::Irrecoverable,
            });
        }

        Ok(())
    }

    /// Runs every tick so the dryer stops on schedule even with no TUI/API client
    /// connected. A scheduled stop time today always takes priority over the drying
    /// timer, matching `compute_remaining_seconds`'s precedence exactly.
    fn check_auto_stop(&mut self) {
        if !is_running_status(self.status.get() as u16) {
            return;
        }

        let (weekday, now_sec) = local_weekday_and_seconds();
        let now_minutes = (now_sec / 60) as i64;
        let scheduled_stop = self.schedule_stop[weekday as usize].get();
        if scheduled_stop != 0 {
            if now_minutes >= scheduled_stop {
                self.dryer.borrow_mut().queue_set_start_stop();
            }
            return;
        }

        if let Some(started) = self.running_since {
            let target_secs = (self.drying_timer_minutes.get() as u64) * 60;
            if started.elapsed().as_secs() >= target_secs {
                self.dryer.borrow_mut().queue_set_start_stop();
            }
        }
    }

    /// Seconds left until the dryer stops itself; `None` if nothing is counting down.
    /// Mirrors `check_auto_stop`'s own precedence exactly.
    fn compute_remaining_seconds(&self) -> Option<i64> {
        if !is_running_status(self.status.get() as u16) {
            return None;
        }

        let (weekday, now_sec) = local_weekday_and_seconds();
        let now_sec = now_sec as i64;
        let scheduled_stop = self.schedule_stop[weekday as usize].get();
        if scheduled_stop != 0 {
            let stop_sec = scheduled_stop * 60;
            return if stop_sec > now_sec {
                Some(stop_sec - now_sec)
            } else {
                None
            };
        }

        let started = self.running_since?;
        let target_sec = self.drying_timer_minutes.get() * 60;
        let elapsed_sec = started.elapsed().as_secs() as i64;
        Some((target_sec - elapsed_sec).max(0))
    }

    fn cmd_start(m: &mut Self) -> ActResult {
        if !is_running_status(m.status.get() as u16) {
            m.dryer.borrow_mut().queue_set_start_stop();
        }
        Ok(())
    }

    fn cmd_stop(m: &mut Self) -> ActResult {
        if is_running_status(m.status.get() as u16) {
            m.dryer.borrow_mut().queue_set_start_stop();
        }
        Ok(())
    }

    fn push_target_temperature(m: &mut Self) -> ActResult {
        let celsius = m.target_temperature.get_as::<degree_celsius>();
        m.dryer.borrow_mut().queue_set_target_temperature(celsius);
        Ok(())
    }

    fn push_air_volume(m: &mut Self) -> ActResult {
        m.dryer
            .borrow_mut()
            .queue_set_air_volume(m.air_volume.get() as u16);
        Ok(())
    }

    fn push_schedule(m: &mut Self) -> ActResult {
        let mut schedule = WeeklySchedule::default();
        for i in 0..7 {
            schedule[i] = ScheduleDay {
                start_time: minutes_to_wire(m.schedule_start[i].get()),
                stop_time: minutes_to_wire(m.schedule_stop[i].get()),
            };
        }
        m.dryer.borrow_mut().queue_set_schedule(schedule);
        Ok(())
    }
}

/// Converts minutes-since-midnight (the schema/config representation) to the device's
/// native `HH*100+MM` wire encoding. 0 = no scheduled action, same sentinel on both sides.
fn minutes_to_wire(minutes: i64) -> u16 {
    if minutes <= 0 {
        return 0;
    }
    let h = (minutes / 60) as u16;
    let m = (minutes % 60) as u16;
    h * 100 + m
}
