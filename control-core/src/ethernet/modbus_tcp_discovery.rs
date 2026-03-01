use crate::ethernet::get_interfaces;
use crate::{futures::FutureIteratorExt, modbus::tcp::ModbusTcpDevice};
use anyhow::{Result, bail};
use interfaces::Interface;
use std::cmp::min;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

pub struct ModbusTcpProbe {
    pub addr: SocketAddr,
    pub serial: u16,
}

pub async fn probe_modbus_tcp() -> Vec<ModbusTcpProbe> {
    let interfaces = match get_interfaces() {
        Ok(x) => x,
        Err(_) => return vec![],
    };

    interfaces
        .into_iter()
        .map(probe_modbus_tcp_addresses)
        .join_all()
        .await
        .into_iter()
        .flatten()
        .collect()
}

async fn probe_modbus_tcp_addresses(interface: Interface) -> Vec<ModbusTcpProbe> {
    let out: Vec<ModbusTcpProbe> = interface
        .addresses
        .iter()
        .filter_map(|addr| {
            let a = addr.addr.map(|a| a.ip());
            let m = addr.mask.map(|a| a.ip());

            match (a, m) {
                (Some(IpAddr::V4(addr)), Some(IpAddr::V4(mask))) => Some((addr, mask)),
                _ => None,
            }
        })
        .flat_map(|(addr, mask)| {
            let a = u32::from(addr);
            let m = u32::from(mask);
            let network = a & m;

            let prefix = min(24, m.count_ones()); // mask → prefix length
            let size = 1u32 << (32 - prefix); // number of addresses

            (0..size).map(move |i| SocketAddr::new(Ipv4Addr::from(network + i).into(), 502))
        })
        .map(|addr| smol::spawn(ping_modbus_device(addr)))
        .join_all()
        .await
        .into_iter()
        .filter_map(|x| x.ok())
        .collect();

    out
}

async fn ping_modbus_device(addr: SocketAddr) -> Result<ModbusTcpProbe> {
    let mut device = ModbusTcpDevice::new(addr).await?;

    let module_number1 = device.get_u32(0x2).await?;
    let module_number2 = device.get_u32(0x4).await?;

    if module_number1 != 0x2787_2144 || module_number2 != 0x0000_0000 {
        bail!("Unknown modbus tcp device!");
    }

    let serial = device.get_u32(0x000A).await?;

    Ok(ModbusTcpProbe {
        addr,
        serial: serial as u16,
    })
}
