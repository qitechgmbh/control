pub mod act;
pub mod api;
pub mod buffer_lift_controller;
pub mod new;

use api::{Buffer1Namespace, BufferV1Events, LiveValuesEvent, ModeState, StateEvent};
use buffer_lift_controller::BufferLiftController;
use control_core::{
    machines::{
        downcast_machine, identification::{MachineIdentification, MachineIdentificationUnique}, manager::MachineManager, ConnectedMachine, ConnectedMachineData, Machine
    },
    socketio::namespace::NamespaceCacheingLogic, uom_extensions::velocity::meter_per_minute,
};
use control_core_derive::Machine;
use futures::executor::block_on;
use serde::{Deserialize, Serialize};
use smol::lock::{Mutex, RwLock};
use uom::si::{f64::Velocity, velocity::millimeter_per_second};
use std::{
    sync::{Arc, Weak},
    time::Instant,
};

use crate::machines::{
    buffer1::api::{ConnectedMachineState, CurrentInputSpeedState}, winder2::Winder2, MACHINE_BUFFER_V1, VENDOR_QITECH
};

#[derive(Debug, Machine)]
pub struct BufferV1 {
    // controllers
    pub buffer_lift_controller: BufferLiftController,

    // socketio
    namespace: Buffer1Namespace,
    last_measurement_emit: Instant,

    // machine connection
    pub machine_manager: Weak<RwLock<MachineManager>>,
    pub machine_identification_unique: MachineIdentificationUnique,

    // connected machines
    pub connected_winder: Option<ConnectedMachine<Weak<Mutex<Winder2>>>>,

    // mode
    mode: BufferV1Mode,
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
}

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
            connected_machine_state: ConnectedMachineState {
                machine_identification_unique: self.connected_winder.as_ref().map(
                    |connected_machine| {
                        ConnectedMachineData::from(connected_machine).machine_identification_unique
                    },
                ),
                is_available: self
                    .connected_winder
                    .as_ref()
                    .map(|connected_machine| {
                        ConnectedMachineData::from(connected_machine).is_available
                    })
                    .unwrap_or(false),
            },
            current_input_speed_state: CurrentInputSpeedState {
                current_input_speed: self.buffer_lift_controller.current_input_speed.get::<meter_per_minute>(),
            } 
        };

        let event = state.build();
        self.namespace.emit(BufferV1Events::State(event));
    }
}

impl BufferV1 {
    // To be implemented
    fn fill_buffer(&mut self) {
        let speed = self.buffer_lift_controller.calculate_buffer_lift_speed();
        self.buffer_lift_controller.update_speed(speed);
    }

    fn empty_buffer(&mut self) {
        self.buffer_lift_controller.update_speed(Velocity::new::<millimeter_per_second>(0.0));
    }

    // Turn off motor and do nothing
    fn switch_to_standby(&mut self) {
        match self.mode {
            BufferV1Mode::Standby => (),
            BufferV1Mode::FillingBuffer => {}
            BufferV1Mode::EmptyingBuffer => {}
        };
        self.mode = BufferV1Mode::Standby;
        self.buffer_lift_controller.set_enabled(false);
        let _ = self.buffer_lift_controller.stepper_driver.set_speed(0.0);
    }

    // Turn on motor and fill buffer
    fn switch_to_filling(&mut self) {
        match self.mode {
            BufferV1Mode::Standby => self.fill_buffer(),
            BufferV1Mode::FillingBuffer => (),
            BufferV1Mode::EmptyingBuffer => {}
        };
        self.mode = BufferV1Mode::FillingBuffer;
        self.buffer_lift_controller.set_enabled(true);
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

    fn set_current_input_speed(&mut self, speed: f64) {
        // speed comes as a f64 represents m/min
        self.buffer_lift_controller.set_current_input_speed(speed);
        self.emit_state();
    }
}

/// Connecting/Disconnecting machine
impl BufferV1 {
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
        let machine_manager_arc = match self.machine_manager.upgrade() {
            Some(machine_manager_arc) => machine_manager_arc,
            None => return,
        };
        let machine_manager_guard = block_on(machine_manager_arc.read());
        let winder2_weak = machine_manager_guard.get_machine_weak(&machine_identification_unique);
        let winder2_weak = match winder2_weak {
            Some(winder2_weak) => winder2_weak,
            None => return,
        };
        let winder2_strong = match winder2_weak.upgrade() {
            Some(winder2_strong) => winder2_strong,
            None => return,
        };

        let winder2: Arc<Mutex<Winder2>> = block_on(downcast_machine::<Winder2>(winder2_strong))
            .expect("failed downcasting machine");

        let machine = Arc::downgrade(&winder2);

        self.connected_winder = Some(ConnectedMachine {
            machine_identification_unique,
            machine,
        });

        self.emit_state();

        self.reverse_connect();
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
        if let Some(connected) = &self.connected_winder {
            if let Some(winder2_arc) = connected.machine.upgrade() {
                let future = async move {
                    let mut winder2 = winder2_arc.lock().await;
                    if winder2.connected_buffer.is_some() {
                        winder2.connected_buffer = None;
                        winder2.emit_state();
                    }
                };
                smol::spawn(future).detach();
            }
        }
        self.connected_winder = None;
        self.emit_state();
    }

    /// initiate connection from winder to buffer
    pub fn reverse_connect(&mut self) {
        let machine_identification_unique = self.machine_identification_unique.clone();
        if let Some(connected) = &self.connected_winder {
            if let Some(winder2_arc) = connected.machine.upgrade() {
                let future = async move {
                    let mut winder2 = winder2_arc.lock().await;
                    if winder2.connected_buffer.is_none() {
                        winder2.set_connected_buffer(machine_identification_unique);
                    }
                };
                smol::spawn(future).detach();
            }
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub enum BufferV1Mode {
    Standby,
    FillingBuffer,
    EmptyingBuffer,
}
