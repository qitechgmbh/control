use control_core::controllers::pid::PidController;
use ethercat_hal::io::{digital_input::DigitalInput, digital_output::DigitalOutput};
use std::{
    sync::{Arc, RwLock},
    time::{Duration, Instant},
};
use uom::si::{f64::VolumeRate, volume_rate::liter_per_minute};

use crate::machines::aquapath1::Flow;

#[derive(Debug)]
pub struct FlowController {
    pub flow: Arc<RwLock<Flow>>,
    // flow_sensor: DigitalInput,
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
        //flow_sensor: DigitalInput,
        pump_relais: DigitalOutput,
        target_flow: VolumeRate,
        flow: Arc<RwLock<Flow>>,
    ) -> Self {
        let now = Instant::now();
        Self {
            // need to tune
            flow: flow,
            last_update: now,
            // flow_sensor: flow_sensor,
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
        if let Ok(mut flow) = self.flow.write() {
            flow.target_flow = target_flow;
        }
        self.target_flow = target_flow;
    }
    pub fn get_target_flow(&mut self) -> VolumeRate {
        self.target_flow
    }

    pub fn get_flow(&mut self, now: Instant) -> VolumeRate {
        let value = false;
        // match self.flow_sensor.get_value() {
        //     Ok(val) => val,
        //     Err(e) => {
        //         tracing::debug!("Error calculating frequency: {}", e);
        //         return VolumeRate::new::<liter_per_minute>(0.0);
        //     }
        // };
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
        if let Ok(mut flow) = self.flow.write() {
            flow.flow = current_flow;
        }
        if self.current_flow > self.max_flow
            || self.target_flow == VolumeRate::new::<liter_per_minute>(0.0)
            || !self.pump_allowed
        {
            self.pump_relais.set(false);
            if let Ok(mut flow) = self.flow.write() {
                flow.pump = false;
            }
            return;
        }

        if self.pump_allowed {
            self.pump_relais.set(true);
            if let Ok(mut flow) = self.flow.write() {
                flow.pump = true;
            }
        }
    }
}
