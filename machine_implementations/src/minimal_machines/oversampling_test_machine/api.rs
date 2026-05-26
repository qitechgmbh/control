use std::sync::Arc;

use control_core::socketio::{
    event::{Event, GenericEvent},
    namespace::{
        CacheFn, CacheableEvents, Namespace, NamespaceCacheingLogic, cache_first_and_last_event,
    },
};
use serde::{Deserialize, Serialize};

use crate::{
    MachineApi, MachineMessage, MachineValues, minimal_machines::oversampling_test_machine::{AnalogOutOversamplingMachine, ChannelConfig, OVERSAMPLE_FACTOR, WaveformType},
};

#[derive(Serialize, Debug, Clone)]
pub struct StateEvent {
    pub channels: [ChannelConfig; 2],
    pub oversample_factor: usize,
    pub cycle_time_us: u64,
}

impl StateEvent {
    pub fn build(&self) -> Event<Self> {
        Event::new("StateEvent", self.clone())
    }
}

#[derive(Serialize, Debug, Clone)]
pub struct LiveValuesEvent {
    /// Most recent samples written to CH1 (one per oversampling slot)
    pub ch1_samples: [f32; OVERSAMPLE_FACTOR],
    /// Most recent samples written to CH2
    pub ch2_samples: [f32; OVERSAMPLE_FACTOR],
}

impl LiveValuesEvent {
    pub fn build(&self) -> Event<Self> {
        Event::new("LiveValuesEvent", self.clone())
    }
}

pub enum AnalogOutOversamplingEvents {
    State(Event<StateEvent>),
    LiveValues(Event<LiveValuesEvent>),
}

#[derive(Deserialize)]
#[serde(tag = "action", content = "value")]
pub enum Mutation {
    /// Replace the full config for one channel (0 or 1)
    SetChannelConfig { channel: usize, config: ChannelConfig },
    SetWaveform {
        channel: usize,
        waveform: WaveformType,
    },
    SetFrequency { channel: usize, frequency_hz: f64 },
    SetAmplitude { channel: usize, amplitude: f64 },
    SetOffset { channel: usize, offset: f64 },
}

#[derive(Debug, Clone)]
pub struct AnalogOutOversamplingNamespace {
    pub namespace: Option<Namespace>,
}

impl NamespaceCacheingLogic<AnalogOutOversamplingEvents> for AnalogOutOversamplingNamespace {
    fn emit(&mut self, events: AnalogOutOversamplingEvents) {
        let event = Arc::new(events.event_value());
        let buffer_fn = events.event_cache_fn();
        if let Some(ns) = &mut self.namespace {
            ns.emit(event, &buffer_fn);
        }
    }
}

impl CacheableEvents<Self> for AnalogOutOversamplingEvents {
    fn event_value(&self) -> GenericEvent {
        match self {
            AnalogOutOversamplingEvents::State(event) => event.into(),
            AnalogOutOversamplingEvents::LiveValues(event) => event.into(),
        }
    }

    fn event_cache_fn(&self) -> CacheFn {
        let cache_first_and_last = cache_first_and_last_event();
        match self {
            Self::State(_) => cache_first_and_last,
            Self::LiveValues(_) => cache_first_and_last,
        }
    }
}

impl MachineApi for AnalogOutOversamplingMachine {
    fn api_mutate(&mut self, value: serde_json::Value) -> Result<(), anyhow::Error> {
        let mutation: Mutation = serde_json::from_value(value)?;
        match mutation {
            Mutation::SetChannelConfig { channel, config } => {
                self.set_channel_config(channel, config);
            }
            Mutation::SetWaveform { channel, waveform } => {
                if channel < self.channels.len() {
                    self.channels[channel].waveform = waveform;
                    self.emit_state();
                }
            }
            Mutation::SetFrequency { channel, frequency_hz } => {
                if channel < self.channels.len() {
                    self.channels[channel].frequency_hz = frequency_hz.clamp(0.001, 10_000.0);
                    self.emit_state();
                }
            }
            Mutation::SetAmplitude { channel, amplitude } => {
                if channel < self.channels.len() {
                    self.channels[channel].amplitude = amplitude.clamp(0.0, 1.0);
                    self.emit_state();
                }
            }
            Mutation::SetOffset { channel, offset } => {
                if channel < self.channels.len() {
                    self.channels[channel].offset = offset.clamp(-1.0, 1.0);
                    self.emit_state();
                }
            }
        }
        Ok(())
    }

    fn api_event_namespace(&mut self) -> Option<Namespace> {
        self.namespace.namespace.clone()
    }

    fn get_api_sender(&self) -> tokio::sync::mpsc::Sender<MachineMessage> {
        self.api_sender.clone()
    }

    fn act_machine_message(&mut self, msg: MachineMessage) {
        match msg {
            MachineMessage::SubscribeNamespace(namespace) => {
                self.namespace.namespace = Some(namespace);
                self.emit_state();
            }
            MachineMessage::UnsubscribeNamespace => self.namespace.namespace = None,
            MachineMessage::HttpApiJsonRequest(value) => {
                let _res = self.api_mutate(value);
            }
            MachineMessage::RequestValues(sender) => {
                sender
                    .send(MachineValues {
                        state: serde_json::to_value(self.get_state())
                            .expect("Failed to serialize state"),
                        live_values: serde_json::to_value(self.get_live_values())
                            .expect("Failed to serialize live values"),
                    })
                    .expect("Failed to send values");
            }
        }
    }
}
