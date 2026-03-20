use std::time::{Duration, Instant};

use control_core::socketio::namespace::NamespaceCacheingLogic;

use crate::{
    MachineChannel, MachineWithChannel, VENDOR_QITECH,
    machine_identification::MachineIdentification,
    stahlwerk::ff01::{
        FF01,
        api::{LiveValues, Mutation, State},
    },
};

#[derive(Debug)]
pub struct Base {
    pub channel: MachineChannel,
    pub state_changed: bool,
    pub emitted_default_state: bool,
    pub last_measurement_emit: Instant,
}

impl Base {
    pub fn new(channel: MachineChannel) -> Self {
        Self { 
            channel, 
            state_changed: false, 
            emitted_default_state: false, 
            last_measurement_emit: Instant::now() 
        }
    }
}

impl FF01 {
    pub const MACHINE_IDENTIFICATION: MachineIdentification = MachineIdentification {
        vendor: VENDOR_QITECH,
        machine: crate::stahlwerk::machine_registry::FF01,
    };

    pub fn emit_live_values(&mut self) {
        if let Some(event) = self.get_live_values() {
            self.base.channel.emit(event);
        }
    }

    pub fn emit_state(&mut self) {
        let event = self.get_state();
        self.base.channel.emit(event);
    }
}

impl MachineWithChannel for FF01 {
    type State = State;
    type LiveValues = LiveValues;

    fn get_machine_channel(&self) -> &MachineChannel {
        &self.base.channel
    }

    fn get_machine_channel_mut(&mut self) -> &mut MachineChannel {
        &mut self.base.channel
    }

    fn mutate(&mut self, value: serde_json::Value) -> anyhow::Result<()> {
        let mutation: Mutation = serde_json::from_value(value)?;
        self.apply_mutation(mutation);
        Ok(())
    }

    fn on_namespace(&mut self) {
        self.emit_state();
    }

    fn update(&mut self, now: Instant) -> anyhow::Result<()> {
        self.update(now)?;

        if self.base.state_changed {
            self.emit_state();
            self.base.state_changed = false;
        }

        if now.duration_since(self.base.last_measurement_emit) > Duration::from_secs_f64(1.0 / 30.0) {
            self.emit_live_values();
            self.base.last_measurement_emit = now;
        }

        Ok(())
    }

    fn get_state(&self) -> State {
        self.state()
    }

    fn get_live_values(&self) -> Option<LiveValues> {
        Some(self.live_values())
    }
}
