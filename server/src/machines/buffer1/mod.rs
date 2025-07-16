use api::{
    BufferV1Events, Buffer1Namespace, StateEvent, ModeState, SineWaveState, LiveValuesEvent,
};
use control_core::{machines::{downcast_machine, identification::{MachineIdentification, MachineIdentificationUnique}, manager::MachineManager, Machine}, socketio::namespace::NamespaceCacheingLogic, uom_extensions::velocity::meter_per_minute};
use futures::executor::block_on;
use serde::{Deserialize, Serialize};
use smol::lock::{Mutex, RwLock};
use tracing::info;
use uom::si::{f64::Frequency, frequency::hertz, length::millimeter};
use std::{any::Any, fmt::Debug, sync::{Arc, Weak}, time::Instant};
use puller_speed_controller::{PullerSpeedController};
use buffer_speed_controller::BufferSpeedController;
use ethercat_hal::io::{
    stepper_velocity_el70x1::StepperVelocityEL70x1,
};

use crate::machines::{buffer1::api::{BufferState, PullerState}, winder2::Winder2, MACHINE_BUFFER_V1, VENDOR_QITECH};

pub mod act;
pub mod api;
pub mod new;
pub mod puller_speed_controller;
pub mod buffer_speed_controller;

#[derive(Debug)]
pub struct BufferV1 {
    // drivers
    pub buffer: StepperVelocityEL70x1,
    pub puller: StepperVelocityEL70x1,

    // controllers
    pub buffer_speed_controller: BufferSpeedController,
    pub puller_speed_controller: PullerSpeedController,

    pub machine_manager: Weak<RwLock<MachineManager>>,

    // socketio
    namespace: Buffer1Namespace,
    last_measurement_emit: Instant,

    // TESTING LIVE EVENTS
    t_0: Instant,
    frequency: Frequency,

    // connected machines
    pub connected_winder: Option<ConnectedMachine<Weak<Mutex<Winder2>>>>,

    // mode
    pub mode: BufferV1Mode,
    pub buffer_mode: BufferMode,
    pub puller_mode: PullerMode,
}

trait GetStrongCount {
    fn get_strong_count(&self) -> usize;
}

impl<T> GetStrongCount for Weak<Mutex<T>> {
    fn get_strong_count(&self) -> usize {
        self.strong_count()
    }
}

#[derive(Debug)]
struct ConnectedMachine<T: GetStrongCount> {
    machine_identification_unique: MachineIdentificationUnique,
    machine: T,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ConnectedMachineData {
    machine_identification_unique: MachineIdentificationUnique,
    is_available: bool,
}

impl<T> From<&ConnectedMachine<T>> for ConnectedMachineData
where T: GetStrongCount
{
    fn from(value: &ConnectedMachine<T>) -> Self {
        Self {
            machine_identification_unique: value.machine_identification_unique.clone(),
            is_available: value.machine.get_strong_count() != 0
        }
    }
}

impl Machine for BufferV1 {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl BufferV1 {
    pub fn emit_live_values(&mut self) {
        let live_values = LiveValuesEvent {
            sine_wave: self.generate_sine_wave(),
        };

        let event = live_values.build();
        self.namespace.emit(BufferV1Events::LiveValues(event));
    }

    pub fn emit_state(&mut self) {
        let state = StateEvent {
            buffer_state: BufferState {

            },
            puller_state: PullerState {
                regulation: self.puller_speed_controller.regulation_mode.clone(),
                target_speed: self
                    .puller_speed_controller
                    .target_speed
                    .get::<meter_per_minute>(),
                target_diameter: self
                    .puller_speed_controller
                    .target_diameter
                    .get::<millimeter>(),
                forward: self.puller_speed_controller.forward,
            },
            mode_state: ModeState {
                mode: self.mode.clone(),
            },
            sinewave_state: SineWaveState {
                frequency: self.frequency.get::<hertz>(),
            },
            connected_machine_state: ConnectedMachineData {
                machine_identification_unique: match &self.connected_winder {
                    Some(connected_machine) => connected_machine.machine_identification_unique.clone(),
                    None => return,
                },
                is_available: match &self.connected_winder {
                    Some(connected_machine) => connected_machine.machine.get_strong_count() != 0,
                    None => false,
                },
            }

        };

        let event = state.build();
        self.namespace.emit(BufferV1Events::State(event));
    }
}

impl BufferV1 {

    pub const MACHINE_IDENTIFICATION: MachineIdentification = MachineIdentification{
        vendor: VENDOR_QITECH,
        machine: MACHINE_BUFFER_V1,
    };
    // Testing Live Value
    pub fn generate_sine_wave(&mut self) -> f64 {
        let now = Instant::now();
        let elapsed = now.duration_since(self.t_0).as_secs_f64();
        let freq_hz = self.frequency.get::<hertz>();

        // Calculate sine wave: sin(2π * frequency * time)
        let amplitude = match self.mode {
            BufferV1Mode::Standby => 0.0,
            _ => (2.0 * std::f64::consts::PI * freq_hz * elapsed).sin(),
        };

       amplitude
    }

    pub fn change_frequency(&mut self, frequency: f64) {
        self.frequency = Frequency::new::<hertz>(frequency);
    }

    // DEBUG MESSAGES
    fn fill_buffer(&mut self) {
        info!("Filling Buffer");
    }

    fn empty_buffer(&mut self) {
        info!("Emptying Buffer");
    }

    fn set_winder(&mut self, machine_identification_unique: MachineIdentificationUnique) {
        if !matches!(machine_identification_unique.machine_identification, Winder2::MACHINE_IDENTIFICATION) {
            return
        }
        let machine_manager_arc = match self.machine_manager.upgrade() {
            Some(machine_manager_arc) => machine_manager_arc,
            None => {
                return
            },
        };
        let machine_manager_guard = block_on(machine_manager_arc.read());
        let winder_weak
            = machine_manager_guard.get_machine_weak(&machine_identification_unique);
        let winder_weak = match winder_weak {
            Some(winder_weak) => winder_weak,
            None => return,
        };
        let winder_strong = match winder_weak.upgrade() {
            Some(winder_strong) => winder_strong,
            None => return,
        };

        let winder2: Arc<Mutex<Winder2>>
            = block_on(downcast_machine::<Winder2>(winder_strong)).expect("failed downcasting machine");

        let machine = Arc::downgrade(&winder2);

        self.connected_winder = Some(ConnectedMachine {
             machine_identification_unique,
             machine: machine.clone(),
        });
        
        self.emit_state();
    }

    // Stop Moving Buffer and do nothing
    fn switch_to_standby(&mut self) {
        match self.mode {
            BufferV1Mode::Standby => (),
            BufferV1Mode::FillingBuffer => {

            }
            BufferV1Mode::EmptyingBuffer => {

            }
        };
    self.mode = BufferV1Mode::Standby;
    }

    // Turn on motor and fill buffer
    fn switch_to_filling(&mut self) {
        match self.mode {
            BufferV1Mode::Standby => self.fill_buffer(),
            BufferV1Mode::FillingBuffer => (),
            BufferV1Mode::EmptyingBuffer => {

            }
        };
    self.mode = BufferV1Mode::FillingBuffer;
    }

    // Turn off motor and empty buffer
    fn switch_to_emptying(&mut self) {
        match self.mode {
            BufferV1Mode::Standby => self.empty_buffer(),
            BufferV1Mode::FillingBuffer => {

            }
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

    fn set_machine_state(&mut self, machine_identification_unique: MachineIdentificationUnique) {
        self.set_winder(machine_identification_unique);
        self.emit_state();
    }

    fn set_frequency_state(&mut self, frequency_mhz: f64) {
        self.change_frequency(frequency_mhz);
        self.emit_state();
    }
    fn set_rpm_state(&mut self, rpm: f64) {
        //TODO
        self.emit_state();
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub enum BufferV1Mode {
    Standby,
    FillingBuffer,
    EmptyingBuffer,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub enum BufferMode {
    Standby,
    Filling,
    Emptying,
}

impl From<BufferV1Mode> for BufferMode {
    fn from(mode: BufferV1Mode) -> Self {
        match mode {
            BufferV1Mode::Standby => BufferMode::Standby,
            BufferV1Mode::FillingBuffer => BufferMode::Filling,
            BufferV1Mode::EmptyingBuffer => BufferMode::Emptying,
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub enum PullerMode {
    Standby,
    Hold,
    Pull,
}

impl From<BufferV1Mode> for PullerMode {
    fn from(mode: BufferV1Mode) -> Self {
        match mode {
            BufferV1Mode::Standby => PullerMode::Standby,
            BufferV1Mode::FillingBuffer => PullerMode::Hold,
            BufferV1Mode::EmptyingBuffer => PullerMode::Pull,
        }
    }
}


impl std::fmt::Display for BufferV1 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "BufferV1")
    }
}
