use api::{
    MockEvents, MockMachineNamespace, Mode, ModeStateEvent, SineWaveEvent, SineWaveStateEvent,
};
use control_core::{machines::Machine, socketio::namespace::NamespaceCacheingLogic};
use tracing_subscriber::fmt::FormatFields;
use std::time::Instant;
use uom::si::{
    f64::Frequency,
    frequency::{hertz, millihertz},
};

pub mod act;
pub mod api;
pub mod new;

#[derive(Debug)]
struct SineWaveFrequencies {
    frequency1: Frequency,
    frequency2: Frequency,
    frequency3: Frequency,
}

impl SineWaveFrequencies {
    pub fn new() -> Self {
        SineWaveFrequencies {
            frequency1: Frequency::new::<hertz>(0.1),
            frequency2: Frequency::new::<hertz>(0.2),
            frequency3: Frequency::new::<hertz>(0.5),
        }
    }

    pub fn get_as_mhz(&self) -> [f64; 3] {
        [
            self.frequency1.get::<millihertz>(),
            self.frequency2.get::<millihertz>(),
            self.frequency3.get::<millihertz>(),
        ]
    }

    pub fn get_amplitudes(&self, t: f64) -> [f64; 3] {
        [
            (self.frequency1.get::<hertz>() * t).sin(),
            (self.frequency2.get::<hertz>() * t).sin(),
            (self.frequency3.get::<hertz>() * t).sin(),
        ]
    }
}

#[derive(Debug)]
pub struct MockMachine {
    // socketio
    namespace: MockMachineNamespace,
    last_measurement_emit: Instant,

    // mock machine specific fields
    t_0: Instant,
    frequencies: SineWaveFrequencies,
    mode: Mode,

    // State tracking to only emit when values change
    last_emitted_frequencies: Option<[f64; 3]>,
    last_emitted_mode: Option<Mode>,
}

impl Machine for MockMachine {}

impl MockMachine {
    /// Emit a sine wave data event with the current time and frequency
    pub fn emit_sine_wave(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.t_0).as_secs_f64();

        // Calculate sine wave: sin(2π * frequency * time)
        let t = match self.mode {
            Mode::Standby => 0.0,
            Mode::Running => 2.0 * std::f64::consts::PI * elapsed,
        };

        let amplitudes = self.frequencies.get_amplitudes(t);

        let sine_wave_event = SineWaveEvent {
            amplitude_sum: amplitudes[0] + amplitudes[1] + amplitudes[2],
            amplitude1: amplitudes[0],
            amplitude2: amplitudes[1],
            amplitude3: amplitudes[2],
        };

        self.namespace
            .emit(MockEvents::SineWave(sine_wave_event.build()));
    }

    /// Emit the current state of the mock machine only if values have changed
    pub fn emit_sine_wave_state(&mut self) {
        let current_frequencies = self.frequencies.get_as_mhz();

        // Only emit if values have changed or this is the first emission
        let should_emit = self.last_emitted_frequencies != Some(current_frequencies);

        if should_emit {
            let frequencies_event = SineWaveStateEvent {
                frequency1: current_frequencies[0],
                frequency2: current_frequencies[1],
                frequency3: current_frequencies[2],
            };

            self.namespace
                .emit(MockEvents::SineWaveState(frequencies_event.build()));

            // Update last emitted values
            self.last_emitted_frequencies = Some(current_frequencies);
        }
    }

    /// Emit the current mode state only if values have changed
    pub fn emit_mode_state(&mut self) {
        let current_mode = self.mode.clone();

        // Only emit if values have changed or this is the first emission
        let should_emit = self.last_emitted_mode != Some(current_mode.clone());

        if should_emit {
            let mode_state_event = ModeStateEvent {
                mode: current_mode.clone(),
            };

            self.namespace
                .emit(MockEvents::ModeState(mode_state_event.build()));

            // Update last emitted values
            self.last_emitted_mode = Some(current_mode);
        }
    }

    /// Set the frequency of the first sine wave
    pub fn set_frequency1(&mut self, frequency_mhz: f64) {
        self.frequencies.frequency1 = Frequency::new::<millihertz>(frequency_mhz);
        // Emit state change immediately
        self.emit_sine_wave_state();
    }

    /// Set the frequency of the second sine wave
    pub fn set_frequency2(&mut self, frequency_mhz: f64) {
        self.frequencies.frequency2 = Frequency::new::<millihertz>(frequency_mhz);
        // Emit state change immediately
        self.emit_sine_wave_state();
    }

    /// Set the frequency of the third sine wave
    pub fn set_frequency3(&mut self, frequency_mhz: f64) {
        self.frequencies.frequency3 = Frequency::new::<millihertz>(frequency_mhz);
        // Emit state change immediately
        self.emit_sine_wave_state();
    }

    /// Set the mode of the mock machine
    pub fn set_mode(&mut self, mode: Mode) {
        self.mode = mode;
        // Emit state change immediately
        self.emit_mode_state();
    }
}
