use std::time::{Duration, Instant};

use control_core::socketio::namespace::NamespaceCacheingLogic;

use crate::{ MachineChannel, MachineIdentification, MachineWithChannel, VENDOR_QITECH };
use super::machine_registry::FF01;

mod new;

mod api;
use api::{LiveValues, Mutation, State};

mod devices;
use devices::{Light, Scale, SignalLights};

mod services;
use services::WorkorderService;

#[derive(Debug)]
pub struct FF01 {
    // devices
    scale: Scale,
    lights: SignalLights,

    // services
    service: WorkorderService,

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
        scale: Scale, 
        lights: SignalLights, 
        service: WorkorderService,
        channel: MachineChannel,
    ) -> Self {
        Self { 
            scale, 
            lights, 
            service, 
            state_changed: false, 
            channel, 
            emitted_default_state: false, 
            last_measurement_emit: Instant::now(), 
        }
    }

    pub fn update(&mut self, now: Instant) -> anyhow::Result<()> {
        // check if done measuring a plate
        if let Some(weight) = self.scale.update() {
            use services::PlateSubmitResult::*;

            let expiry = now + Self::EXPIRY_DURATION;

            match self.service.submit_plate(weight) {
                InBounds => self.lights.enable_light(Light::Green, Some(expiry)),
                OutOufBOunds => self.lights.enable_light(Light::Red, Some(expiry)),
                NotCounting => self.lights.enable_light(Light::Yellow, Some(expiry)),
            }

            self.state_changed = true;
        }

        self.lights.update(now);
        self.service.update(now)?;
        Ok(())
    }
}

// utils
impl FF01 {
    fn handle_mutation(&mut self, mutation: Mutation) {
        use Mutation::*;

        match mutation {
            SetTare => self.scale.tare(),
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
        self.update(now)?;

        if self.state_changed {
            self.emit_state();
            self.state_changed = false;
        }

        if now.duration_since(self.last_measurement_emit) > Duration::from_secs_f64(1.0 / 30.0) {
            self.emit_live_values();
            self.last_measurement_emit = now;
        }

        Ok(())
    }

    fn get_state(&self) -> State {
        let current_workorder = self.service.current_entry().as_ref().map(|x| x.doc_entry);

        State {
            is_default_state: self.emitted_default_state,
            plates_counted: self.service.plates_counted(),
            current_workorder,
        }
    }

    fn get_live_values(&self) -> Option<LiveValues> {
        let live_values = LiveValues {
            weight_peak: self.scale.weight_peak(),
            weight_prev: self.scale.weight_prev(),
        };

        Some(live_values)
    }
}
