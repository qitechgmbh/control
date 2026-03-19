use std::time::{Duration, Instant};

use anyhow::anyhow;
use control_core::socketio::namespace::NamespaceCacheingLogic;

use crate::{ MachineChannel, MachineIdentification, MachineWithChannel, VENDOR_QITECH };
use super::machine_registry::FF01;

mod new;

mod api;
use api::{LiveValues, Mutation, State};

mod devices;
use devices::{Light, Scales, SignalLights};

mod tasks;
use tasks::PlateDetectTask;

mod services;
use services::WorkorderService;

#[derive(Debug)]
pub struct FF01 {
    // devices
    scale: Scales,
    lights: SignalLights,

    // tasks
    plate_detect_task: PlateDetectTask,

    // services
    service: WorkorderService,

    // state
    plates_counted: u32,

    // repeating machine junk
    state_changed: bool,
    channel: MachineChannel,
    emitted_default_state: bool,
    last_measurement_emit: Instant,
}

// constants
impl FF01 {
    const EXPIRY_DURATION: Duration = Duration::from_millis(500);
}

// public interface
impl FF01 {
    pub fn new(
        scale: Scales, 
        lights: SignalLights, 
        service: WorkorderService,
        channel: MachineChannel,
    ) -> Self {
        Self { 
            scale, 
            lights, 
            service, 
            plate_detect_task: PlateDetectTask::new(),
            plates_counted: 0,
            // junk
            state_changed: false, 
            channel, 
            emitted_default_state: false, 
            last_measurement_emit: Instant::now(), 
        }
    }

    pub fn update_impl(&mut self, now: Instant) -> anyhow::Result<()> {
        
        self.scale.update();
        self.lights.update(now);
        // self.service.update(now)?;

        let Some(weight) = self.scale.weight() else {
            // received no data from scales
            self.plate_detect_task.reset();
            return Ok(());
        };

        if !self.plate_detect_task.check(weight) {
            // no plate detected
            return Ok(());
        }

        let Some(weight_peak) = self.scale.weight_peak() else {
            // unreachable
            return Err(anyhow!("No weight_peak but should have"));
        };

        // current peak is the plate measurement, so reset it
        self.scale.weight_peak_reset();

        let Some(entry) = self.service.current_entry() else {
            // no active entry
            return Ok(());
        };

        let bounds = &entry.weight_bounds;
        let expires_at = now + Self::EXPIRY_DURATION;

        if weight_peak < bounds.min || bounds.max < weight_peak {
            // out of bounds
            self.lights.enable_light(Light::Red, Some(expires_at));
            return Ok(());
        }

        self.lights.enable_light(Light::Green, Some(expires_at));
        self.plates_counted += 1;

        Ok(())
    }
}

// utils
impl FF01 {
    fn handle_mutation(&mut self, mutation: Mutation) {
        use Mutation::*;

        match mutation {
            Tare => self.scale.tare(),
            ClearTare => self.scale.tare_clear(),
            ClearLights => self.lights.lights_disable_all(),
        }

        self.state_changed = true;
    }
}

// ----------------------------------------------------------------------------------------
// -------------------------------- repeating machine junk --------------------------------
// ----------------------------------------------------------------------------------------
impl FF01 {
    pub const MACHINE_IDENTIFICATION: MachineIdentification = MachineIdentification {
        vendor: VENDOR_QITECH,
        machine: FF01,
    };

    pub fn emit_live_values(&mut self) {
        if let Some(event) = self.get_live_values() {
            self.channel.emit(event);
        }
    }

    pub fn emit_state(&mut self) {
        let event = self.get_state();
        self.channel.emit(event);
    }
}

impl MachineWithChannel for FF01 {
    type State = State;
    type LiveValues = LiveValues;

    fn get_machine_channel(&self) -> &MachineChannel {
        &self.channel
    }

    fn get_machine_channel_mut(&mut self) -> &mut MachineChannel {
        &mut self.channel
    }

    fn mutate(&mut self, value: serde_json::Value) -> anyhow::Result<()> {
        let mutation: Mutation = serde_json::from_value(value)?;
        self.handle_mutation(mutation);
        Ok(())
    }

    fn on_namespace(&mut self) {
        self.emit_state();
    }

    fn update(&mut self, now: Instant) -> anyhow::Result<()> {
        self.update_impl(now)?;

        if self.state_changed {
            //self.emit_state();
            self.state_changed = false;
        }

        if now.duration_since(self.last_measurement_emit) > Duration::from_secs_f64(1.0 / 30.0) {
            self.emit_live_values();
            self.emit_state();
            self.last_measurement_emit = now;
        }

        Ok(())
    }

    fn get_state(&self) -> State {
        State {
            is_default_state: self.emitted_default_state,
            plates_counted: self.plates_counted,
            current_entry: self.service.current_entry().clone(),
        }
    }

    fn get_live_values(&self) -> Option<LiveValues> {
        let live_values = LiveValues {
            weight_peak: self.scale.weight_peak(),
            weight_prev: self.scale.weight(),
        };

        Some(live_values)
    }
}
