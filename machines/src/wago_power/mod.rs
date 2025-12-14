use std::{net::SocketAddr, time::Instant};

use anyhow::Result;
use control_core::modbus::tcp::ModbusTcpDevice;

use crate::{MachineAct, MachineChannel, MachineMessage, MachineWithChannel};

#[derive(Debug)]
pub struct WagoPowerSupply {
    channel: MachineChannel,
    device: ModbusTcpDevice,
}

impl WagoPowerSupply {
    fn new(channel: MachineChannel, addr: SocketAddr) -> Result<Self> {
        let device = smol::block_on(ModbusTcpDevice::new(addr))?;
        Ok(Self { channel, device })
    }
}

impl MachineWithChannel for WagoPowerSupply {
    fn get_machine_channel(&self) -> &MachineChannel {
        &self.channel
    }

    fn get_machine_channel_mut(&mut self) -> &mut MachineChannel {
        &mut self.channel
    }

    fn mutate(&mut self, _mutation: serde_json::Value) -> Result<()> {
        Ok(())
    }

    fn update(&mut self, _now: Instant) {
    }
}
