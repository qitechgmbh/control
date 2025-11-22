use anyhow::{Context, Result, bail};
use ethercrab::{
    MainDevice, MainDeviceConfig, PduStorage, RetryBehaviour, Timeouts,
    std::{ethercat_now, tx_rx_task},
};
use interfaces::Interface;
use std::cmp::min;
use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    process::Command,
};

use crate::{futures::FutureIteratorExt, modbus::tcp::ModbusTcpDevice};

// Constants for EtherCAT configuration
const IFACE_DISCOVERY_MAX_SUBDEVICES: usize = 16; // Must be power of 2 > 1
const IFACE_DISCOVERY_MAX_PDU_DATA: usize = PduStorage::element_size(1100);
const IFACE_DISCOVERY_MAX_FRAMES: usize = 16;
const IFACE_DISCOVERY_MAX_PDI_LEN: usize = 128;

#[derive(Default, Debug)]
pub struct EthernetProbe {
    pub ethercat: Option<String>,
    pub modbus_tcp: Vec<SocketAddr>,
}

/// Returns true if the given network interface is Ethernet.
/// Prevents testing wlan,loopback and other non ethernet devices
fn is_ethernet(interface: &str) -> std::io::Result<bool> {
    #[cfg(target_os = "linux")]
    {
        let base_path = format!("/sys/class/net/{}", interface);
        let type_path = std::path::Path::new(&base_path).join("type");
        let iface_type = std::fs::read_to_string(&type_path)?.trim().to_string();
        // 1 means Ethernet
        let reports_as_ethernet = iface_type == "1";
        // Double-check that it's not a wireless interface
        let uevent_path = std::path::Path::new(&base_path).join("uevent");
        let uevent = std::fs::read_to_string(&uevent_path).unwrap_or_default();
        // If "DEVTYPE=wlan" appears, it's lying
        let actually_wifi = uevent.contains("DEVTYPE=wlan");
        Ok(reports_as_ethernet && !actually_wifi)
    }
    #[cfg(not(target_os = "linux"))]
    return Ok(true);
}

pub fn get_interfaces() -> Result<Vec<Interface>> {
    let vec = Interface::get_all()
        .map_err(|e| anyhow::anyhow!("Failed to get network interfaces: {}", e))?
        .into_iter()
        .filter(|iface| {
            iface.is_up()
                && iface.is_running()
                && !iface.is_loopback()
                && is_ethernet(&iface.name).unwrap_or(false)
                && !iface.name.starts_with("bridge")
                && !iface.name.starts_with("utun")   // Exclude tunnel interfaces
                && !iface.name.starts_with("awdl")   // Exclude Apple Wireless Direct Link
                && !iface.name.starts_with("anpi")   // Exclude Apple Network Privacy Interface
                && !iface.name.starts_with("llw") // Exclude Apple low-latency WLAN
        })
        .collect::<Vec<_>>();
    Ok(vec)
}

pub async fn probe_ethernet() -> EthernetProbe {
    tracing::info!("Probing network for devices...");

    let interfaces = match get_interfaces() {
        Ok(x) => x,
        Err(_) => return EthernetProbe::default(),
    };

    let ethercat: Option<String> = interfaces
        .iter()
        .map(|iface| iface.name.to_owned())
        .map(probe_ethercat)
        .join_all()
        .await
        .into_iter()
        .find_map(|x| x.ok());

    let modbus_tcp = interfaces
        .into_iter()
        .map(probe_modbus_tcp)
        .join_all()
        .await
        .into_iter()
        .flatten()
        .collect();

    EthernetProbe {
        ethercat,
        modbus_tcp,
    }
}

async fn probe_modbus_tcp(interface: Interface) -> Vec<SocketAddr> {
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

async fn probe_ethercat(interface: String) -> Result<String> {
    tracing::info!("Testing ethercat on interface: {}", interface);

    let pdu_storage = Box::leak(Box::new(PduStorage::<
        IFACE_DISCOVERY_MAX_FRAMES,
        IFACE_DISCOVERY_MAX_PDU_DATA,
    >::new()));

    let (tx, rx, pdu_loop) = pdu_storage
        .try_split()
        .expect("Failed to split PDU storage");

    let tx_rx_handle =
        smol::spawn(tx_rx_task(&interface, tx, rx).context("Failed to spawn TX/RX task")?);

    let maindevice = MainDevice::new(
        pdu_loop,
        Timeouts::default(),
        MainDeviceConfig {
            // Default 10000
            dc_static_sync_iterations: 10000,
            // Default None
            retry_behaviour: RetryBehaviour::Count(3),
        },
    );

    let res = maindevice
        .init_single_group::<IFACE_DISCOVERY_MAX_SUBDEVICES, IFACE_DISCOVERY_MAX_PDI_LEN>(
            ethercat_now,
        )
        .await
        .context(format!(
            "Failed to initialize EtherCAT on interface {}",
            interface
        ));

    tx_rx_handle.cancel().await;

    res?;
    Ok(interface)
}

pub async fn discover_ethercat_interface() -> Result<String, anyhow::Error> {
    tracing::info!("Discovering EtherCAT interface...");

    // Set up a custom panic hook that suppresses panic output
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        // Optional: log the panic in a controlled way
        if let Some(location) = panic_info.location() {
            tracing::debug!(
                "Suppressed panic in interface test: {} (at {}:{})",
                panic_info,
                location.file(),
                location.line()
            );
        } else {
            tracing::debug!("Suppressed panic in interface test: {}", panic_info);
        }
    }));

    let mut interfaces = get_interfaces()?;

    interfaces.sort_by(|a, b| a.name.cmp(&b.name));
    let mut interface: Option<&str> = None;

    for i in 0..interfaces.len() {
        match probe_ethercat(interfaces[i].name.to_string()).await {
            Ok(_) => {
                // if interface found with ethercat Exit early, we expect only one interface with ethercat
                interface = Some(&interfaces[i].name);
                break;
            }
            Err(_) => (),
        }
    }

    for i in 0..interfaces.len() {
        if interface.is_some() && interface.unwrap() == &interfaces[i].name {
            set_interface_managed(&interfaces[i].name, false);
        } else {
            set_interface_managed(&interfaces[i].name, true);
        }
    }

    // Restore the default panic hook
    std::panic::set_hook(default_hook);

    match interface {
        Some(interface) => Ok(interface.to_string()),
        None => Err(anyhow::anyhow!("No suitable EtherCAT interface found")),
    }
}

/// Sets a network interface to unmanaged by NetworkManager.
/// Returns true if the command succeeded.
pub fn set_interface_managed(interface: &str, managed: bool) -> bool {
    let managed_str = match managed {
        true => "yes",
        false => "no",
    };
    tracing::info!(
        "set_interface_managed for {} managed was set to: {}",
        interface,
        managed_str
    );
    let status = Command::new("nmcli")
        .args(["dev", "set", interface, "managed", managed_str])
        .status();

    matches!(status, Ok(s) if s.success())
}
