pub mod act;
pub mod api;
pub mod buffer_tower_controller;
pub mod new;

use api::{Buffer1Namespace, BufferV1Events, LiveValuesEvent, ModeState, StateEvent};
use buffer_tower_controller::BufferTowerController;
use control_core::machines::connection::{CrossConnectableMachine, MachineCrossConnection};
use control_core::{
    machines::{
        identification::{MachineIdentification, MachineIdentificationUnique},
        manager::MachineManager,
    },
    socketio::namespace::NamespaceCacheingLogic,
};
use control_core_derive::Machine;
use serde::{Deserialize, Serialize};
use smol::lock::RwLock;
use std::{sync::Weak, time::Instant};

use crate::machines::{MACHINE_BUFFER_V1, VENDOR_QITECH, winder2::Winder2};

#[derive(Debug, Machine)]
pub struct BufferV1 {
    // controllers
    pub buffer_tower_controller: BufferTowerController,

    // socketio
    namespace: Buffer1Namespace,
    last_measurement_emit: Instant,

    // machine connection
    pub machine_manager: Weak<RwLock<MachineManager>>,
    pub machine_identification_unique: MachineIdentificationUnique,

    // connected machines
    pub connected_winder: MachineCrossConnection<BufferV1, Winder2>,

    // mode
    mode: BufferV1Mode,
}

impl CrossConnectableMachine<BufferV1, Winder2> for BufferV1 {
    fn get_cross_connection(&mut self) -> &mut MachineCrossConnection<BufferV1, Winder2> {
        &mut self.connected_winder
    }
}

impl std::fmt::Display for BufferV1 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "BufferV1")
    }
}

impl BufferV1 {
    pub const MACHINE_IDENTIFICATION: MachineIdentification = MachineIdentification {
        vendor: VENDOR_QITECH,
        machine: MACHINE_BUFFER_V1,
    };
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
            connected_machine_state: self.connected_winder.to_state(),
        };

        let event = state.build();
        self.namespace.emit(BufferV1Events::State(event));
    }

    // To be implemented
    fn fill_buffer(&mut self) {
        todo!();
    }

    fn empty_buffer(&mut self) {
        todo!();
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

    fn set_mode_state(&mut self, mode: BufferV1Mode) {
        self.switch_mode(mode);
        self.emit_state();
    }

    /// Connecting/Disconnecting machine
    /// set connected winder
    pub fn set_connected_winder(
        &mut self,
        machine_identification_unique: MachineIdentificationUnique,
    ) {
        if !matches!(
            machine_identification_unique.machine_identification,
            Winder2::MACHINE_IDENTIFICATION
        ) {
            return;
        }

        self.connected_winder
            .set_connected_machine(&machine_identification_unique);

        self.emit_state();

        self.connected_winder.reverse_connect();
    }

    /// disconnect winder
    pub fn disconnect_winder(
        &mut self,
        machine_identification_unique: MachineIdentificationUnique,
    ) {
        if !matches!(
            machine_identification_unique.machine_identification,
            Winder2::MACHINE_IDENTIFICATION
        ) {
            return;
        }

        self.connected_winder.disconnect();
        self.emit_state();
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub enum BufferV1Mode {
    Standby,
    FillingBuffer,
    EmptyingBuffer,
}
