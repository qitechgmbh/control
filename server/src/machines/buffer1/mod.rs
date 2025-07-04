use api::{
    BufferV1Events, Buffer1Namespace, StateEvent, ModeState, SineWaveState, LiveValuesEvent,
};
use control_core::{machines::Machine, socketio::namespace::NamespaceCacheingLogic};
use serde::{Deserialize, Serialize};
use tracing::info;
use uom::si::{f64::Frequency, frequency::hertz};
use std::time::Instant;

pub mod act;
pub mod api;
pub mod new;

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub enum BufferV1Mode {
    Standby,
    FillingBuffer,
    EmptyingBuffer,
}

#[derive(Debug)]
pub struct BufferV1 {
    namespace: Buffer1Namespace,
    last_measurement_emit: Instant,

    // TESTING LIVE EVENTS
    t_0: Instant,
    frequency: Frequency,

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
        let live_values = LiveValuesEvent {
            sine_wave: self.generate_sine_wave(),
        };

        let event = live_values.build();
        self.namespace.emit(BufferV1Events::LiveValues(event));
    }

    pub fn emit_state(&mut self) {
        let state = StateEvent {
            mode_state: ModeState {
                mode: self.mode.clone(),
            },
            sinewave_state: SineWaveState {
                frequency: self.frequency.get::<hertz>(),
            },
        };

        let event = state.build();
        self.namespace.emit(BufferV1Events::State(event));
    }
}

impl BufferV1 {
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

    fn set_frequency_state(&mut self, frequency_mhz: f64) {
        self.change_frequency(frequency_mhz);
        self.emit_state();
    }
}
