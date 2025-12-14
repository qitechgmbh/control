use std::{net::SocketAddr, time::{Duration, Instant}};
use smol::future;
use anyhow::Result;
use control_core::{modbus::tcp::ModbusTcpDevice, socketio::{event::{BuildEvent, GenericEvent}, namespace::{CacheFn, CacheableEvents, NamespaceCacheingLogic, cache_n_events}}};
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
    device: ModbusTcpDevice,
    last_emit: Instant,
}

impl WagoPower {

    pub async fn new(channel: MachineChannel, addr: SocketAddr) -> Result<Self> {
        Ok(Self {
            channel,
            device: ModbusTcpDevice::new(addr).await?,
            last_emit: Instant::now(),
        })
    }

    #[cfg(feature = "mock-machine")]
    pub fn get_state(&mut self) -> Result<StateEvent> {
        Ok(StateEvent {
            voltage_milli_volt: 24000.0,
            current_milli_ampere: 5000.0,
        })
    }

    #[cfg(not(feature = "mock-machine"))]
    pub fn get_state(&mut self) -> Result<StateEvent> {
        smol::block_on(async {
            let electric = self.device.get_holding_registers(0x0500, 2).await?;
            let voltage = electric[0];
            let current = electric[1];

            Ok(StateEvent { voltage_milli_volt: f32::from(voltage), current_milli_ampere: f32::from(current) })
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
            if let Ok(event) = self.get_state() {
                self.channel.emit(event);
                self.last_emit = now;
            }
        }
    }
}
