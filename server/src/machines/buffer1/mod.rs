pub mod act;
pub mod api;
pub mod buffer_tower_controller;
pub mod new;

use api::{Buffer1Namespace, BufferV1Events, LiveValuesEvent, ModeState, StateEvent};
use buffer_tower_controller::BufferTowerController;
use control_core::{machines::Machine, socketio::namespace::NamespaceCacheingLogic};
use serde::{Deserialize, Serialize};
use std::time::Instant;
use tracing::info;

#[derive(Debug)]
pub struct BufferV1 {
    // controllers
    pub buffer_tower_controller: BufferTowerController,

    // socketio
    namespace: Buffer1Namespace,
    last_measurement_emit: Instant,

    // mode
    mode: BufferV1Mode,
}

impl std::fmt::Display for BufferV1 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "BufferV1")
    }
}

impl Machine for BufferV1 {}

impl BufferV1 {
    pub fn emit_live_values(&mut self) {
        let live_values = LiveValuesEvent {};

        let event = live_values.build();
        self.namespace.emit(BufferV1Events::LiveValues(event));
    }

    pub fn emit_state(&mut self) {
        let state = StateEvent {
            mode_state: ModeState {
                mode: self.mode.clone(),
            },
        };

        let event = state.build();
        self.namespace.emit(BufferV1Events::State(event));
    }
}

impl BufferV1 {
    // DEBUG MESSAGES
    fn fill_buffer(&mut self) {
        info!("Filling Buffer");
    }

    fn empty_buffer(&mut self) {
        info!("Emptying Buffer");
    }

    // Turn off motor and do nothing
    fn switch_to_standby(&mut self) {
        match self.mode {
            BufferV1Mode::Standby => (),
            BufferV1Mode::FillingBuffer => {}
            BufferV1Mode::EmptyingBuffer => {}
        };
        self.mode = BufferV1Mode::Standby;
        self.buffer_tower_controller.set_enabled(false);
    }

    // Turn on motor and fill buffer
    fn switch_to_filling(&mut self) {
        match self.mode {
            BufferV1Mode::Standby => self.fill_buffer(),
            BufferV1Mode::FillingBuffer => (),
            BufferV1Mode::EmptyingBuffer => {}
        };
        self.mode = BufferV1Mode::FillingBuffer;
        self.buffer_tower_controller.set_enabled(true);
    }

    // Turn on motor reverse and empty buffer
    fn switch_to_emptying(&mut self) {
        match self.mode {
            BufferV1Mode::Standby => self.empty_buffer(),
            BufferV1Mode::FillingBuffer => {}
            BufferV1Mode::EmptyingBuffer => (),
        };
        self.mode = BufferV1Mode::EmptyingBuffer;
    }

    fn switch_mode(&mut self, mode: BufferV1Mode) {
        if self.mode == mode {
            return;
        }

        match mode {
            BufferV1Mode::Standby => self.switch_to_standby(),
            BufferV1Mode::FillingBuffer => self.switch_to_filling(),
            BufferV1Mode::EmptyingBuffer => self.switch_to_emptying(),
        }
    }
}

impl BufferV1 {
    fn set_mode_state(&mut self, mode: BufferV1Mode) {
        self.switch_mode(mode);
        self.emit_state();
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub enum BufferV1Mode {
    Standby,
    FillingBuffer,
    EmptyingBuffer,
}
