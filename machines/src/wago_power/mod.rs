use std::{net::SocketAddr, time::Instant};

use anyhow::Result;
use control_core::modbus::tcp::ModbusTcpDevice;

use crate::{HasMachineChannel, MachineAct, MachineChannel, MachineMessage, Mutatable};

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

impl HasMachineChannel for WagoPowerSupply {
    fn get_machine_channel(&self) -> &MachineChannel {
        &self.channel
    }

    fn get_machine_channel_mut(&mut self) -> &mut MachineChannel {
        &mut self.channel
    }
}

impl Mutatable for WagoPowerSupply {
    fn mutate(&mut self, _mutation: serde_json::Value) -> Result<()> {
        Ok(())
    }
}

impl MachineAct for WagoPowerSupply {
    fn act(&mut self, _now: Instant) {}

    fn act_machine_message(&mut self, _msg: MachineMessage) {}
}
