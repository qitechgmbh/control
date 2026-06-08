use api::{LiveValuesEvent, MockEvents, MockMachineNamespace, Mode, ModeState, StateEvent};
use control_core::socketio::event::BuildEvent;
use control_core::socketio::namespace::NamespaceCacheingLogic;
use tokio::sync::mpsc::{Receiver, Sender};
use std::time::Instant;
use tracing::info;
use units::f64::*;
use units::frequency::{hertz, millihertz};

use crate::{
    MACHINE_MOCK, MachineMessage, QiTechMachine, VENDOR_QITECH,
};
use qitech_lib::machines::{MachineIdentification, MachineIdentificationUnique};

pub mod act;
pub mod api;
pub mod new;

#[derive(Debug)]
pub struct MockMachine {
    pub receiver: Receiver<MachineMessage>,
    pub sender: Sender<MachineMessage>,
    pub machine_identification_unique: MachineIdentificationUnique,
    pub namespace: MockMachineNamespace,
    pub last_measurement_emit: Instant,
    pub t_0: Instant,
    pub frequency1: Frequency,
    pub frequency2: Frequency,
    pub frequency3: Frequency,
    pub mode: Mode,
    pub last_emitted_event: Option<StateEvent>,
    pub emitted_default_state: bool,
}

impl QiTechMachine for MockMachine {}

impl MockMachine {
    pub const MACHINE_IDENTIFICATION: MachineIdentification = MachineIdentification {
        machine: MACHINE_MOCK,
        vendor: VENDOR_QITECH,
    };

    pub fn get_live_values(&self) -> LiveValuesEvent {
        let now = Instant::now();
        let elapsed = now.duration_since(self.t_0).as_secs_f64();
        let freq1_hz = self.frequency1.get::<hertz>();
        let freq2_hz = self.frequency2.get::<hertz>();
        let freq3_hz = self.frequency3.get::<hertz>();

        let t = match self.mode {
            Mode::Standby => 0.0,
            Mode::Running => 2.0 * std::f64::consts::PI * elapsed,
        };

        let amplitude1 = (t * freq1_hz).sin();
        let amplitude2 = (t * freq2_hz).sin();
        let amplitude3 = (t * freq3_hz).sin();

        LiveValuesEvent {
            amplitude_sum: amplitude1 + amplitude2 + amplitude3,
            amplitude1,
            amplitude2,
            amplitude3,
        }
    }

    pub fn emit_live_values(&mut self) {
        let event = self.get_live_values().build();
        self.namespace.emit(MockEvents::LiveValues(event));
    }

    pub fn get_state(&self) -> StateEvent {
        info!(
            "Emitting state for MockMachine, is default state: {}",
            !self.emitted_default_state
        );

        StateEvent {
            is_default_state: !self.emitted_default_state,
            frequency1: self.frequency1.get::<millihertz>(),
            frequency2: self.frequency2.get::<millihertz>(),
            frequency3: self.frequency3.get::<millihertz>(),
            mode_state: ModeState {
                mode: self.mode.clone(),
            },
        }
    }

    pub fn emit_state(&mut self) {
        let state = self.get_state();
        let event = state.build();
        self.namespace.emit(MockEvents::State(event));
        self.emitted_default_state = true;
        self.last_emitted_event = Some(state);
    }

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

    pub fn set_mode(&mut self, mode: Mode) {
        self.mode = mode;
        self.emit_state();
    }
}
