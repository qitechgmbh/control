use api::{
    LiveValuesEvent, MockEvents, MockMachineNamespace, Mode, ModeState, SineWaveState, StateEvent,
};
use control_core::{machines::{downcast_machine, identification::{MachineIdentification, MachineIdentificationUnique}, manager::MachineManager, Machine}, socketio::namespace::NamespaceCacheingLogic};
use futures::executor::block_on;
use serde::Serialize;
use smol::lock::{Mutex, RwLock};
use tracing::info;
use std::{any::Any, sync::{Arc, Weak}, time::Instant};
use uom::si::{
    f64::Frequency,
    frequency::{hertz, millihertz},
};

use crate::machines::{mock::api::ConnectedMachineState, mock2::Mock2Machine, MACHINE_MOCK, VENDOR_QITECH};

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
    frequency: Frequency,
    mode: Mode,

    pub machine_manager: Weak<RwLock<MachineManager>>,

    // connected machines
    pub connected_mock2: Option<ConnectedMachine<Weak<Mutex<Mock2Machine>>>>,

    // State tracking to only emit when values change
    last_emitted_state: Option<StateEvent>,

    /// Will be initialized as false and set to true by emit_state
    /// This way we can signal to the client that the first state emission is a default state
    emitted_default_state: bool,
}

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
where T: GetStrongCount
{
    fn from(value: &ConnectedMachine<T>) -> Self {
        Self{
            machine_identification_unique: value.machine_identification_unique.clone(),
            is_available: value.machine.get_strong_count() != 0

        }
    }
}

impl Machine for MockMachine {
    fn as_any(&self) -> &dyn Any {
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
            sine_wave_state: SineWaveState {
                frequency: self.frequency.get::<millihertz>(),
            },
            mode_state: ModeState {
                mode: self.mode.clone(),
            },
            connected_machine_state: ConnectedMachineState {
                machine_identification_unique: self
                    .connected_mock2
                    .as_ref()
                    .map(|connected_machine| 
                        ConnectedMachineData::from(connected_machine).machine_identification_unique.clone()),
                is_available: self
                    .connected_mock2
                    .as_ref()
                    .map(|connected_machine| 
                        ConnectedMachineData::from(connected_machine).is_available)
                    .unwrap_or(false),
            },
        };

        // Only emit if values have changed or this is the first emission
        let should_emit = self.last_emitted_state.as_ref() != Some(&current_state);

        if should_emit {
            self.namespace
                .emit(MockEvents::State(current_state.build()));

            // Update last emitted state
            self.last_emitted_state = Some(current_state);
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

    /// Set the connected mock2 machine
    pub fn set_connected_mock2(&mut self, machine_identification_unique: MachineIdentificationUnique) {
        if !matches!(machine_identification_unique.machine_identification, Mock2Machine::MACHINE_IDENTIFICATION) {
            return
        }
        let machine_manager_arc = match self.machine_manager.upgrade() {
            Some(machine_manager_arc) => machine_manager_arc,
            None => {
                return
            },
        };
        let machine_manager_guard = block_on(machine_manager_arc.read());
        let mock2_weak
            = machine_manager_guard.get_machine_weak(&machine_identification_unique);
        let mock2_weak = match mock2_weak {
            Some(mock2_weak) => mock2_weak,
            None => return,
        };
        let mock2_strong = match mock2_weak.upgrade() {
            Some(mock2_strong) => mock2_strong,
            None => return,
        };

        let mock2: Arc<Mutex<Mock2Machine>>
            = block_on(downcast_machine::<Mock2Machine>(mock2_strong)).expect("failed downcasting machine");

        let machine = Arc::downgrade(&mock2);

        self.connected_mock2 = Some(ConnectedMachine {
            machine_identification_unique,
            machine: machine.clone(),
        });

        info!("Hello there");

        self.emit_state();
        
    }
}
