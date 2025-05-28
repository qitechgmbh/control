use api::{
    MockEvents, MockMachineNamespace, Mode, ModeStateEvent, SineWaveEvent, SineWaveStateEvent,
};
use control_core::{machines::Machine, socketio::namespace::NamespaceCacheingLogic};
use std::time::Instant;
use uom::si::{
    f64::Frequency,
    frequency::{hertz, millihertz},
};

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

    // State tracking to only emit when values change
    last_emitted_frequency: Option<f64>,
    last_emitted_mode: Option<Mode>,
}

impl Machine for MockMachine {}

impl MockMachine {
    /// Emit a sine wave data event with the current time and frequency
    pub fn emit_sine_wave(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.t_0).as_secs_f64();
        let freq_hz = self.frequency.get::<hertz>();

        // Calculate sine wave: sin(2π * frequency * time)
        let y = match self.mode {
            Mode::Standby => 0.0,
            Mode::Running => (2.0 * std::f64::consts::PI * freq_hz * elapsed).sin(),
        };

        let sine_wave_event = SineWaveEvent { amplitude: y };

        self.namespace
            .emit_cached(MockEvents::SineWave(sine_wave_event.build()));
    }

    /// Emit the current state of the mock machine only if values have changed
    pub fn emit_sine_wave_state(&mut self) {
        let current_frequency_mhz = self.frequency.get::<millihertz>();

        // Only emit if values have changed or this is the first emission
        let should_emit = self.last_emitted_frequency != Some(current_frequency_mhz);

        if should_emit {
            let mock_state_event = SineWaveStateEvent {
                frequency: current_frequency_mhz,
            };

            self.namespace
                .emit_cached(MockEvents::SineWaveState(mock_state_event.build()));

            // Update last emitted values
            self.last_emitted_frequency = Some(current_frequency_mhz);
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
                .emit_cached(MockEvents::ModeState(mode_state_event.build()));

            // Update last emitted values
            self.last_emitted_mode = Some(current_mode);
        }
    }

    /// Set the frequency of the sine wave
    pub fn set_frequency(&mut self, frequency_mhz: f64) {
        self.frequency = Frequency::new::<millihertz>(frequency_mhz);
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
