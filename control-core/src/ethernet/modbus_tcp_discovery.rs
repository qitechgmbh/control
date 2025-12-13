use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::cmp::min;
use anyhow::{Result, bail};
use interfaces::Interface;
use crate::ethernet::get_interfaces;
use crate::{futures::FutureIteratorExt, modbus::tcp::ModbusTcpDevice};

pub async fn probe_modbus_tcp() -> Vec<SocketAddr> {
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

async fn probe_modbus_tcp_addresses(interface: Interface) -> Vec<SocketAddr> {
    let out: Vec<SocketAddr> = interface
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

            (200..size).map(move |i| SocketAddr::new(Ipv4Addr::from(network + i).into(), 502))
        })
        .map(|addr| smol::spawn(ping_modbus_device(addr)))
        .join_all()
        .await
        .into_iter()
        .filter_map(|x| x.ok())
        .collect();

    println!("FOUOOOUUUUNNNDDD {:?}", out);

    out
}

async fn ping_modbus_device(addr: SocketAddr) -> Result<SocketAddr> {
    tracing::info!("Trying modbus tcp at {}", addr);
    let mut device = ModbusTcpDevice::new(addr).await?;

    let serial1 = device.get_u32(0x2).await?;
    let serial2 = device.get_u32(0x4).await?;

    if serial1 != 0x2787_2144 || serial2 != 0x0000_0000 {
        bail!("Unknown modbus tcp device!");
    }

    Ok(addr)
}
