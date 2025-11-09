use crate::AsyncThreadMessage;
use crate::{
    MACHINE_MOCK, Machine, MachineMessage, VENDOR_QITECH,
    machine_identification::{MachineIdentification, MachineIdentificationUnique},
};
use api::{LiveValuesEvent, MockEvents, MockMachineNamespace, Mode, ModeState, StateEvent};
use control_core::socketio::event::BuildEvent;
use control_core::socketio::namespace::NamespaceCacheingLogic;
use smol::channel::{Receiver, Sender};
use std::time::Instant;
use tracing::info;
use units::f64::*;
use units::frequency::{hertz, millihertz};

pub mod act;
pub mod api;
pub mod new;

#[derive(Debug)]
pub struct MockMachine {
    machine_identification_unique: MachineIdentificationUnique,
    main_sender: Option<Sender<AsyncThreadMessage>>,

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
    api_sender: Sender<MachineMessage>,
    api_receiver: Receiver<MachineMessage>,
}

impl Machine for MockMachine {
    fn get_machine_identification_unique(&self) -> MachineIdentificationUnique {
        self.machine_identification_unique.clone()
    }

    fn get_main_sender(&self) -> Option<Sender<AsyncThreadMessage>> {
        self.main_sender.clone()
    }
}

impl MockMachine {
    pub const MACHINE_IDENTIFICATION: MachineIdentification = MachineIdentification {
        machine: MACHINE_MOCK,
        vendor: VENDOR_QITECH,
    };

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

    /// Set the frequencies of the sine waves
    pub fn set_frequency1(&mut self, frequency_mhz: f64) {
        self.frequency1 = Frequency::new::<millihertz>(frequency_mhz);
        self.emit_state();
    }

    pub fn set_frequency2(&mut self, frequency_mhz: f64) {
        self.frequency2 = Frequency::new::<millihertz>(frequency_mhz);
        self.emit_state();
    }

    pub fn set_frequency3(&mut self, frequency_mhz: f64) {
        self.frequency3 = Frequency::new::<millihertz>(frequency_mhz);
        self.emit_state();
    }

    /// Set the mode of the mock machine
    pub fn set_mode(&mut self, mode: Mode) {
        self.mode = mode;
        self.emit_state();
    }
}
