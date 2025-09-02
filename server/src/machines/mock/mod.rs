use crate::machines::{MACHINE_MOCK, VENDOR_QITECH};
use api::{LiveValuesEvent, MockEvents, MockMachineNamespace, Mode, ModeState, StateEvent};
use control_core::socketio::event::BuildEvent;
use control_core::{
    helpers::hasher_serializer::check_hash_different,
    machines::{Machine, identification::MachineIdentification},
    socketio::namespace::NamespaceCacheingLogic,
};

use std::time::Instant;
use tracing::info;
use uom::si::{
    f64::Frequency,
    frequency::{hertz, millihertz},
};

pub mod act;
pub mod api;
pub mod new;

#[derive(Debug)]
pub struct MockMachine {
    // socketio
    namespace: MockMachineNamespace,
    last_measurement_emit: Instant,

    // mock machine specific fields
    t_0: Instant,
    frequency1: Frequency,
    frequency2: Frequency,
    frequency3: Frequency,
    mode: Mode,

    // State tracking to only emit when values change
    last_emitted_event: Option<StateEvent>,

    /// Will be initialized as false and set to true by emit_state
    /// This way we can signal to the client that the first state emission is a default state
    emitted_default_state: bool,
}

impl Machine for MockMachine {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
impl MockMachine {
    pub const MACHINE_IDENTIFICATION: MachineIdentification = MachineIdentification {
        vendor: VENDOR_QITECH,
        machine: MACHINE_MOCK,
    };
}

impl MockMachine {
    /// Emit live values data event with the current sine wave amplitude
    pub fn emit_live_values(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.t_0).as_secs_f64();
        let freq1_hz = self.frequency1.get::<hertz>();
        let freq2_hz = self.frequency2.get::<hertz>();
        let freq3_hz = self.frequency3.get::<hertz>();

        // Calculate sine wave: sin(2π * frequency * time)
        let t = match self.mode {
            Mode::Standby => 0.0,
            Mode::Running => 2.0 * std::f64::consts::PI * elapsed,
        };

        let amplitude1 = (t * freq1_hz).sin();
        let amplitude2 = (t * freq2_hz).sin();
        let amplitude3 = (t * freq3_hz).sin();

        let live_values = LiveValuesEvent {
            amplitude_sum: amplitude1 + amplitude2 + amplitude3,
            amplitude1,
            amplitude2,
            amplitude3,
        };

        self.namespace
            .emit(MockEvents::LiveValues(live_values.build()));
    }

    /// Emit the current state of the mock machine only if values have changed
    pub fn emit_state(&mut self) {
        info!(
            "Emitting state for MockMachine, is default state: {}",
            !self.emitted_default_state
        );

        let current_state = StateEvent {
            is_default_state: !std::mem::replace(&mut self.emitted_default_state, true),
            frequency1: self.frequency1.get::<millihertz>(),
            frequency2: self.frequency2.get::<millihertz>(),
            frequency3: self.frequency3.get::<millihertz>(),
            mode_state: ModeState {
                mode: self.mode.clone(),
            },
        };

        self.namespace
            .emit(MockEvents::State(current_state.build()));
        self.last_emitted_event = Some(current_state);
    }

    pub fn maybe_emit_state_event(&mut self) {
        let new_state = StateEvent {
            is_default_state: !std::mem::replace(&mut self.emitted_default_state, true),
            frequency1: self.frequency1.get::<millihertz>(),
            frequency2: self.frequency2.get::<millihertz>(),
            frequency3: self.frequency3.get::<millihertz>(),
            mode_state: ModeState {
                mode: self.mode.clone(),
            },
        };

        let old_event = match &self.last_emitted_event {
            Some(old_event) => old_event,
            None => {
                self.emit_state();
                return;
            }
        };

        let should_emit = check_hash_different(&new_state, old_event);
        if should_emit {
            self.last_emitted_event = Some(new_state.clone());
            self.namespace.emit(MockEvents::State(new_state.build()));
        }
    }

    /// Set the frequencies of the sine waves
    pub fn set_frequency1(&mut self, frequency_mhz: f64) {
        self.frequency1 = Frequency::new::<millihertz>(frequency_mhz);
    }

    pub fn set_frequency2(&mut self, frequency_mhz: f64) {
        self.frequency2 = Frequency::new::<millihertz>(frequency_mhz);
    }

    pub fn set_frequency3(&mut self, frequency_mhz: f64) {
        self.frequency3 = Frequency::new::<millihertz>(frequency_mhz);
    }

    /// Set the mode of the mock machine
    pub fn set_mode(&mut self, mode: Mode) {
        self.mode = mode;
    }
}
