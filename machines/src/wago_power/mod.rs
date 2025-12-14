use std::time::{Duration, Instant};

use anyhow::Result;
use control_core::socketio::{event::{BuildEvent, GenericEvent}, namespace::{CacheFn, CacheableEvents, NamespaceCacheingLogic, cache_n_events}};
use control_core_derive::BuildEvent;
use serde::Serialize;

use crate::{MachineChannel, MachineWithChannel};

#[derive(Serialize, Debug, Clone, BuildEvent)]
pub struct StateEvent {
  voltage_milli_volt: f32,
  current_milli_ampere: f32
}

impl CacheableEvents<Self> for StateEvent {

    fn event_value(&self) -> GenericEvent {
        self.build().into()
    }

    fn event_cache_fn(&self) -> CacheFn {
        cache_n_events(10)
    }
}

#[derive(Debug)]
pub struct WagoPower {
    channel: MachineChannel,
    last_emit: Instant,
}

impl WagoPower {
    pub async fn new(channel: MachineChannel, /* addr: SocketAddr */) -> Result<Self> {
        // let device = smol::block_on(ModbusTcpDevice::new(addr))?;
        Ok(Self {
            channel,
            last_emit: Instant::now(),
        })
    }
}

impl MachineWithChannel for WagoPower {
    fn get_machine_channel(&self) -> &MachineChannel {
        &self.channel
    }

    fn get_machine_channel_mut(&mut self) -> &mut MachineChannel {
        &mut self.channel
    }

    fn mutate(&mut self, _mutation: serde_json::Value) -> Result<()> {
        Ok(())
    }

    fn update(&mut self, now: Instant) {
        if now - self.last_emit < Duration::from_millis(1000 / 30) {
            let event = StateEvent {
                voltage_milli_volt: 24000.0,
                current_milli_ampere: 5000.0,
            };

            self.channel.emit(event);
            self.last_emit = now;
        }
    }
}
