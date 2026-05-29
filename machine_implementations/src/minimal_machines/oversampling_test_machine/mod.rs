pub mod act;
pub mod api;
pub mod new;

use crate::{
    ANALOG_OUT_OVERSAMPLING_MACHINE, MachineMessage, QiTechMachine, VENDOR_QITECH,
    minimal_machines::oversampling_test_machine::api::{
        AnalogOutOversamplingEvents, AnalogOutOversamplingNamespace, LiveValuesEvent, StateEvent,
    },
};
use control_core::socketio::namespace::NamespaceCacheingLogic;
use qitech_lib::{
    ethercat_hal::{devices::el4732::EL4732, pdo::oversampling::OVERSAMPLE_FACTOR},
    machines::{MachineIdentification, MachineIdentificationUnique},
};
use std::{cell::RefCell, f64::consts::PI, rc::Rc, time::Instant};
use tokio::sync::mpsc::{Receiver, Sender};

/// How many samples the EL4732 outputs per EtherCAT cycle.
/// Must be one of: 1, 2, 3, 4, 5, 8, 10, 16, 20, 25, 32, 40, 50, 100
pub const CYCLE_TIME_US: u64 = 1000;
pub const SYNC0_PERIOD_US: u64 = CYCLE_TIME_US / OVERSAMPLE_FACTOR as u64; // 250µs
pub const SYNC1_PERIOD_US: u64 = SYNC0_PERIOD_US * (OVERSAMPLE_FACTOR as u64 - 1); // 750µs

#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum WaveformType {
    Sine,
    Sawtooth,
    Square,
    Constant,
}

impl Default for WaveformType {
    fn default() -> Self {
        Self::Sine
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ChannelConfig {
    pub waveform: WaveformType,
    /// Output frequency in Hz (ignored for Constant)
    pub frequency_hz: f64,
    /// Amplitude 0.0 ..= 1.0, maps to 0 V .. ±10 V full-scale
    pub amplitude: f64,
    /// DC offset -1.0 ..= 1.0
    pub offset: f64,
}

impl Default for ChannelConfig {
    fn default() -> Self {
        Self {
            waveform: WaveformType::Sine,
            frequency_hz: 10.0,
            amplitude: 1.0,
            offset: 0.0,
        }
    }
}

impl QiTechMachine for AnalogOutOversamplingMachine {}

#[derive(Debug)]
pub struct AnalogOutOversamplingMachine {
    pub api_receiver: Receiver<MachineMessage>,
    pub api_sender: Sender<MachineMessage>,

    pub el4732: Rc<RefCell<EL4732>>,

    pub namespace: AnalogOutOversamplingNamespace,
    pub machine_identification_unique: MachineIdentificationUnique,
    pub last_state_emit: Instant,
    pub last_live_values_emit: Instant,

    /// Per-channel waveform configuration
    pub channels: [ChannelConfig; 2],

    /// Phase accumulators in radians, advanced by one full cycle per act() call
    pub phase: [f64; 2],

    /// Last computed samples, kept for LiveValuesEvent
    pub last_samples: [[f32; OVERSAMPLE_FACTOR as usize]; 2],

    pub last_act: Option<Instant>,
}

impl AnalogOutOversamplingMachine {
    pub const MACHINE_IDENTIFICATION: MachineIdentification = MachineIdentification {
        vendor: VENDOR_QITECH,
        machine: ANALOG_OUT_OVERSAMPLING_MACHINE,
    };
}

impl AnalogOutOversamplingMachine {
    pub fn get_state(&self) -> StateEvent {
        StateEvent {
            channels: self.channels.clone(),
            oversample_factor: OVERSAMPLE_FACTOR as usize,
            cycle_time_us: CYCLE_TIME_US,
        }
    }

    pub fn emit_state(&mut self) {
        let event = self.get_state().build();
        self.namespace
            .emit(AnalogOutOversamplingEvents::State(event));
    }

    pub fn get_live_values(&self) -> LiveValuesEvent {
        LiveValuesEvent {
            ch1_samples: self.last_samples[0],
            ch2_samples: self.last_samples[1],
        }
    }

    pub fn emit_live_values(&mut self) {
        let event = self.get_live_values().build();
        self.namespace
            .emit(AnalogOutOversamplingEvents::LiveValues(event));
    }
}

impl AnalogOutOversamplingMachine {
    pub fn set_channel_config(&mut self, channel: usize, config: ChannelConfig) {
        if channel < self.channels.len() {
            self.channels[channel] = config;
            self.emit_state();
        }
    }

    /// Generate OVERSAMPLE_FACTOR samples for one channel and advance its phase.
    /// Sub-sample phases are linearly interpolated so the waveform is continuous
    /// across cycle boundaries regardless of frequency.
    pub fn generate_samples(&mut self, channel: usize) -> [f32; OVERSAMPLE_FACTOR as usize] {
        let now = Instant::now();
        let elapsed_secs = match self.last_act {
            Some(last) => now.duration_since(last).as_secs_f64(),
            None => CYCLE_TIME_US as f64 * 1e-6,
        };

        let config = &self.channels[channel];
        let cycle_secs = CYCLE_TIME_US as f64 * 1e-6;

        // Inter-slot spacing is always fixed to the EtherCAT cycle time —
        // each slot is one sub-period of the cycle apart from the next.
        let phase_step_per_slot =
            2.0 * PI * config.frequency_hz * cycle_secs / OVERSAMPLE_FACTOR as f64;

        let mut samples = [0.0f32; OVERSAMPLE_FACTOR as usize];
        for (i, slot) in samples.iter_mut().enumerate() {
            let p = self.phase[channel] + phase_step_per_slot * i as f64;
            let raw = match config.waveform {
                WaveformType::Sine => p.sin(),
                WaveformType::Sawtooth => {
                    let t = p / (2.0 * PI);
                    2.0 * (t - t.floor()) - 1.0
                }
                WaveformType::Square => {
                    if p.sin() >= 0.0 {
                        1.0
                    } else {
                        -1.0
                    }
                }
                WaveformType::Constant => 1.0,
            };
            *slot = ((raw * config.amplitude + config.offset).clamp(-1.0, 1.0)) as f32;
        }

        // Phase accumulator advances by actual elapsed time between act() calls
        self.phase[channel] =
            (self.phase[channel] + 2.0 * PI * config.frequency_hz * elapsed_secs) % (2.0 * PI);

        samples
    }
}
