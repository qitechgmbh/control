use control_core::controllers::pid::PidController;
use ethercat_hal::io::{digital_input::DigitalInput, digital_output::DigitalOutput};
use std::{
    sync::{Arc, RwLock},
    time::{Duration, Instant},
};
use uom::si::{f32::Volume, f64::VolumeRate, volume_rate::liter_per_minute};

use crate::machines::aquapath1::Flow;

#[derive(Debug)]
pub struct FlowController {
    pub pid: PidController,
    pump_pid_output: f64,
    window_start: Instant,
    pwm_period: Duration,

    pub flow: Arc<RwLock<Flow>>,

    flow_sensor: DigitalInput,
    pump_relais: DigitalOutput,

    pub target_flow: VolumeRate,
    last_update: Instant,
    last_value: bool,

    pub pump_allowed: bool,

    pub current_flow: VolumeRate,
    pub max_flow: VolumeRate,
}

impl FlowController {
    pub fn new(
        kp: f64,
        ki: f64,
        kd: f64,
        pwm_duration: Duration,
        flow_sensor: DigitalInput,
        pump_relais: DigitalOutput,
        target_flow: VolumeRate,
        flow: Arc<RwLock<Flow>>,
    ) -> Self {
        let now = Instant::now();
        Self {
            // need to tune
            pid: PidController::new(kp, ki, kd),
            pump_pid_output: 0.0,
            window_start: Instant::now(),
            pwm_period: pwm_duration,
            flow: flow,
            last_update: now,
            flow_sensor: flow_sensor,
            pump_relais: pump_relais,
            target_flow: target_flow,
            last_value: false,
            current_flow: VolumeRate::new::<liter_per_minute>(0.0),
            pump_allowed: false,
            max_flow: VolumeRate::new::<liter_per_minute>(10.0),
        }
    }
    pub fn disable(&mut self) {
        if let Ok(mut flow) = self.flow.write() {
            flow.pump = false;
        }
        self.pump_relais.set(false);

        self.disallow_pump();
    }

    pub fn enable(&mut self) {
        if let Ok(mut flow) = self.flow.write() {
            flow.pump = true;
        }
        self.pump_relais.set(true);

        self.allow_pump();
    }
    pub fn disallow_pump(&mut self) {
        self.pump_allowed = false;
    }

    pub fn allow_pump(&mut self) {
        self.pump_allowed = true;
    }

    pub fn set_target_flow(&mut self, target_flow: VolumeRate) {
        self.reset_pid();
        if let Ok(mut flow) = self.flow.write() {
            flow.target_flow = target_flow;
        }
        self.target_flow = target_flow;
    }
    pub fn get_target_flow(&mut self) -> VolumeRate {
        self.target_flow
    }
    pub fn reset_pid(&mut self) {
        self.pid.reset()
    }

    pub fn get_flow(&mut self, now: Instant) -> VolumeRate {
        let value = match self.flow_sensor.get_value() {
            Ok(val) => val,
            Err(e) => {
                tracing::debug!("Error calculating frequency: {}", e);
                return VolumeRate::new::<liter_per_minute>(0.0);
            }
        };
        if value == self.last_value {
            return self.current_flow;
        } else {
            self.last_value = value;
            let freq = 1.0 / (now - self.last_update).as_secs_f64();
            self.last_update = now;
            // Formula: f = 8.1*q - 3, so q = (f + 3) / 8.1
            let actual_flow = (freq + 3.0) / 8.1;
            VolumeRate::new::<liter_per_minute>(actual_flow)
        }
    }

    pub fn update(&mut self, now: Instant) {
        let current_flow = self.get_flow(now);
        self.current_flow = current_flow;
        self.pump_pid_output = 0.0;
        if let Ok(mut flow) = self.flow.write() {
            flow.flow = current_flow;
        }
        if self.current_flow > self.max_flow
            || self.target_flow == VolumeRate::new::<liter_per_minute>(0.0)
        {
            self.pump_relais.set(false);
            if let Ok(mut flow) = self.flow.write() {
                flow.pump = false;
            }
            return;
        }

        if self.pump_allowed {
            let error: f64 = self.target_flow.get::<liter_per_minute>();
            -current_flow.get::<liter_per_minute>();

            let control = self.pid.update(error, now); // PID output
            // Clamp PID output to 0.0 – 1.0 (as duty cycle)
            let duty = control.clamp(0.0, 1.0);

            self.pump_pid_output = duty;

            let elapsed = now.duration_since(self.window_start);

            // Restart window if needed
            if elapsed >= self.pwm_period {
                self.window_start = now;
            }
            // Compare duty cycle to elapsed time
            let on_time = self.pwm_period.mul_f64(duty);

            // Relay is ON if within duty cycle window
            let on = elapsed < on_time;

            self.pump_relais.set(on);
            if let Ok(mut flow) = self.flow.write() {
                flow.pump = on;
            }
        }
    }

    pub fn reset(&mut self) {
        self.pid.reset();
        self.last_update = Instant::now();
    }
}
