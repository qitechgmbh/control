use api::{
    MockEvents, MockMachineNamespace, Mode, StateEvent, LiveValuesEvent, SineWaveState, ModeState,
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
    last_emitted_state: Option<StateEvent>,
}

impl Machine for MockMachine {}

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
        let current_state = StateEvent {
            sine_wave_state: SineWaveState {
                frequency: self.frequency.get::<millihertz>(),
            },
            mode_state: ModeState {
                mode: self.mode.clone(),
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
}
