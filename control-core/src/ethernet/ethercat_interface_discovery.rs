use anyhow::{Context, Result};
use ethercrab::{
    MainDevice, MainDeviceConfig, PduStorage, RetryBehaviour, Timeouts,
    std::{ethercat_now, tx_rx_task},
};

use crate::{
    ethernet::{get_interfaces, set_interface_managed},
    futures::FutureIteratorExt,
};

// Constants for EtherCAT configuration
const IFACE_DISCOVERY_MAX_SUBDEVICES: usize = 16; // Must be power of 2 > 1
const IFACE_DISCOVERY_MAX_PDU_DATA: usize = PduStorage::element_size(1100);
const IFACE_DISCOVERY_MAX_FRAMES: usize = 16;
const IFACE_DISCOVERY_MAX_PDI_LEN: usize = 128;

pub async fn probe_ethercat() -> Option<String> {
    let interfaces = match get_interfaces() {
        Ok(x) => x,
        Err(_) => return None,
    };

    interfaces
        .iter()
        .map(|iface| iface.name.to_owned())
        .map(probe_ethercat_interface)
        .join_all()
        .await
        .into_iter()
        .find_map(|x| x.ok())
}

async fn probe_ethercat_interface(interface: String) -> Result<String> {
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
        match probe_ethercat_interface(interfaces[i].name.to_string()).await {
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
