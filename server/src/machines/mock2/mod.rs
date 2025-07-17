use api::{
    LiveValuesEvent, Mock2Events, Mock2MachineNamespace, Mode, ModeState, SineWaveState, StateEvent,
};
use control_core::{
    machines::{
        Machine, downcast_machine,
        identification::{MachineIdentification, MachineIdentificationUnique},
        manager::MachineManager,
    },
    socketio::namespace::NamespaceCacheingLogic,
};
use futures::executor::block_on;
use serde::Serialize;
use smol::lock::{Mutex, RwLock};
use std::{
    any::Any,
    sync::{Arc, Weak},
    time::Instant,
};
use tracing::info;
use uom::si::{
    f64::Frequency,
    frequency::{hertz, millihertz},
};

use crate::machines::{
    MACHINE_MOCK2, VENDOR_QITECH, mock::MockMachine, mock2::api::ConnectedMachineState,
};

pub mod act;
pub mod api;
pub mod new;

#[derive(Debug)]
pub struct Mock2Machine {
    // socketio
    namespace: Mock2MachineNamespace,
    last_measurement_emit: Instant,

    // mock machine specific fields
    t_0: Instant,
    frequency: Frequency,
    mode: Mode,

    pub machine_manager: Weak<RwLock<MachineManager>>,
    pub machine_identification_unique: MachineIdentificationUnique,

    // connected machines
    pub connected_mock: Option<ConnectedMachine<Weak<Mutex<MockMachine>>>>,

    // State tracking to only emit when values change
    last_emitted_state: Option<StateEvent>,

    /// Will be initialized as false and set to true by emit_state
    /// This way we can signal to the client that the first state emission is a default state
    emitted_default_state: bool,
}

// this maybe could be moved into control core
pub trait GetStrongCount {
    fn get_strong_count(&self) -> usize;
}

impl<T> GetStrongCount for Weak<Mutex<T>> {
    fn get_strong_count(&self) -> usize {
        self.strong_count()
    }
}

#[derive(Debug)]
pub struct ConnectedMachine<T: GetStrongCount> {
    machine_identification_unique: MachineIdentificationUnique,
    machine: T,
}

#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct ConnectedMachineData {
    machine_identification_unique: MachineIdentificationUnique,
    is_available: bool,
}

impl<T> From<&ConnectedMachine<T>> for ConnectedMachineData
where
    T: GetStrongCount,
{
    fn from(value: &ConnectedMachine<T>) -> Self {
        Self {
            machine_identification_unique: value.machine_identification_unique.clone(),
            is_available: value.machine.get_strong_count() != 0,
        }
    }
}

impl Machine for Mock2Machine {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Mock2Machine {
    pub const MACHINE_IDENTIFICATION: MachineIdentification = MachineIdentification {
        vendor: VENDOR_QITECH,
        machine: MACHINE_MOCK2,
    };
}

impl Mock2Machine {
    /// Emit live values data event with the current sine wave amplitude
    pub fn emit_live_values(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.t_0).as_secs_f64();
        let freq_hz = self.frequency.get::<hertz>();

        // Calculate sine wave: sin(2π * frequency * time)
        let amplitude = match self.mode {
            Mode::Standby => 0.0,
            Mode::Running => (2.0 * std::f64::consts::PI * freq_hz * elapsed).sin(),
        };

        let live_values = LiveValuesEvent {
            sine_wave_amplitude: amplitude,
        };

        self.namespace
            .emit(Mock2Events::LiveValues(live_values.build()));
    }

    /// Emit the current state of the mock machine only if values have changed
    pub fn emit_state(&mut self) {
        info!(
            "Emitting state for Mock2Machine, is default state: {}",
            !self.emitted_default_state
        );

        let current_state = StateEvent {
            is_default_state: !std::mem::replace(&mut self.emitted_default_state, true),
            sine_wave_state: SineWaveState {
                frequency: self.frequency.get::<millihertz>(),
            },
            mode_state: ModeState {
                mode: self.mode.clone(),
            },
            connected_machine_state: ConnectedMachineState {
                machine_identification_unique: self.connected_mock.as_ref().map(
                    |connected_machine| {
                        ConnectedMachineData::from(connected_machine)
                            .machine_identification_unique
                            .clone()
                    },
                ),
                is_available: self
                    .connected_mock
                    .as_ref()
                    .map(|connected_machine| {
                        ConnectedMachineData::from(connected_machine).is_available
                    })
                    .unwrap_or(false),
            },
        };

        // Only emit if values have changed or this is the first emission
        let should_emit = self.last_emitted_state.as_ref() != Some(&current_state);

        if should_emit {
            self.namespace
                .emit(Mock2Events::State(current_state.build()));

            // Update last emitted state
            self.last_emitted_state = Some(current_state);
        }

        // Only emit connected machine state when machine connected
        if self.connected_mock.is_some() {
            if let Some(connected) = &self.connected_mock {
                if let Some(mock_arc) = connected.machine.upgrade() {
                    let mut mock = block_on(mock_arc.lock());
                    mock.emit_state();
                }
            }
        }
    }

    /// Set the frequency of the sine wave
    pub fn set_frequency(&mut self, frequency_mhz: f64) {
        self.frequency = Frequency::new::<millihertz>(frequency_mhz);
        // Emit state change immediately
        self.emit_state();
    }

    /// Set the mode of the mock machine
    pub fn set_mode(&mut self, mode: Mode) {
        self.mode = mode;
        // Emit state change immediately
        self.emit_state();
    }

    /// Testing Connected machine
    pub fn set_connected_mock_frequency(&mut self, f: f64) {
        if let Some(connected) = &self.connected_mock {
            if let Some(mock_arc) = connected.machine.upgrade() {
                let mut mock = block_on(mock_arc.lock());
                mock.set_frequency(f);
                mock.emit_state();
            }
        }
    }

    /// Set the connected mock2 machine
    pub fn set_connected_mock(
        &mut self,
        machine_identification_unique: MachineIdentificationUnique,
    ) {
        if !matches!(
            machine_identification_unique.machine_identification,
            MockMachine::MACHINE_IDENTIFICATION
        ) {
            return;
        }
        let machine_manager_arc = match self.machine_manager.upgrade() {
            Some(machine_manager_arc) => machine_manager_arc,
            None => return,
        };
        let machine_manager_guard = block_on(machine_manager_arc.read());
        let mock_weak = machine_manager_guard.get_serial_weak(&machine_identification_unique);
        let mock_weak = match mock_weak {
            Some(mock_weak) => mock_weak,
            None => return,
        };
        let mock_strong = match mock_weak.upgrade() {
            Some(mock_strong) => mock_strong,
            None => return,
        };

        let mock: Arc<Mutex<MockMachine>> = block_on(downcast_machine::<MockMachine>(mock_strong))
            .expect("failed downcasting machine");

        let machine = Arc::downgrade(&mock);

        self.connected_mock = Some(ConnectedMachine {
            machine_identification_unique,
            machine: machine.clone(),
        });

        self.emit_state();

        let self_id = self.machine_identification_unique.clone();
        let maybe_clone = machine.upgrade();

        if let Some(mock_arc) = maybe_clone {
            tokio::spawn(async move {
                let mut mock = match mock_arc.lock().await {
                    guard => guard,
                };
                if mock.connected_mock2.is_none() {
                    mock.set_connected_mock2(self_id);
                }
            });
        }
    }

    pub fn disconnect_mock(&mut self, machine_identification_unique: MachineIdentificationUnique) {
        if !matches!(
            machine_identification_unique.machine_identification,
            MockMachine::MACHINE_IDENTIFICATION
        ) {
            return;
        }
        if let Some(connected) = &self.connected_mock {
            if let Some(mock_arc) = connected.machine.upgrade() {
                let mut mock = block_on(mock_arc.lock());
                mock.connected_mock2 = None;
            }
        }
        self.connected_mock = None;
        self.emit_state();
    }
}
