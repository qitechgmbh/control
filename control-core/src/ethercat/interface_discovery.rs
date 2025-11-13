use ethercrab::{
    MainDevice, MainDeviceConfig, PduStorage, Timeouts,
    std::{ethercat_now, tx_rx_task},
};
use interfaces::Interface;
use std::{process::Command, sync::Arc, time::Duration};

// Constants for EtherCAT configuration
const IFACE_DISCOVERY_MAX_SUBDEVICES: usize = 16; // Must be power of 2 > 1
const IFACE_DISCOVERY_MAX_PDU_DATA: usize = PduStorage::element_size(1100);
const IFACE_DISCOVERY_MAX_FRAMES: usize = 16;
const IFACE_DISCOVERY_MAX_PDI_LEN: usize = 128;

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
        return Ok(reports_as_ethernet && !actually_wifi);
    }
    #[cfg(not(target_os = "linux"))]
    return Ok(true);
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

    // Get eligible interfaces
    let mut interfaces = Interface::get_all()
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

    interfaces.sort_by(|a, b| a.name.cmp(&b.name));
    let mut interface: Option<&str> = None;

    for i in 0..interfaces.len() {
        match test_interface(&interfaces[i].name) {
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
        Some(interface) => return Ok(interface.to_string()),
        None => return Err(anyhow::anyhow!("No suitable EtherCAT interface found")),
    }
}

fn test_interface(interface: &str) -> Result<(), anyhow::Error> {
    tracing::info!("Testing interface: {}", interface);

    let pdu_storage = Box::leak(Box::new(PduStorage::<
        IFACE_DISCOVERY_MAX_FRAMES,
        IFACE_DISCOVERY_MAX_PDU_DATA,
    >::new()));
    let (tx, rx, pdu_loop) = pdu_storage
        .try_split()
        .map_err(|e| anyhow::anyhow!("Failed to split PDU storage: {:?}", e))?;

    let rt = smol::LocalExecutor::new();

    let result = rt.run(async {
        let tx_rx_handle = rt.spawn(
            tx_rx_task(interface, tx, rx)
                .map_err(|e| anyhow::anyhow!("Failed to spawn TX/RX task: {}", e))?,
        );

        let maindevice = Arc::new(MainDevice::new(
            pdu_loop,
            Timeouts {
                // Default 5000ms
                state_transition: Duration::from_millis(5000),
                // Default 30_000us
                pdu: Duration::from_micros(30_000),
                // Default 10ms
                eeprom: Duration::from_millis(10),
                // Default 0ms
                wait_loop_delay: Duration::from_millis(0),
                // Default 100ms
                mailbox_echo: Duration::from_millis(100),
                // Default 1000ms
                mailbox_response: Duration::from_millis(1000),
            },
            MainDeviceConfig {
                // Default 10000
                dc_static_sync_iterations: 10000,
                // Default None
                retry_behaviour: ethercrab::RetryBehaviour::Count(3),
            },
        ));

        let result = maindevice
            .init_single_group::<IFACE_DISCOVERY_MAX_SUBDEVICES, IFACE_DISCOVERY_MAX_PDI_LEN>(
                ethercat_now,
            )
            .await
            .map(|_| ())
            .map_err(|e| anyhow::anyhow!("Failed to initialize group: {}", e));

        tx_rx_handle.cancel().await;

        result
    });

    // await the result of the async block
    let result = smol::block_on(result);

    if let Err(e) = result {
        return Err(anyhow::anyhow!(
            "Failed to initialize EtherCAT on interface {}: {:?}",
            interface,
            e
        ));
    }

    Ok(())
}
