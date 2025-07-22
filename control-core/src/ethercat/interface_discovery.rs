use std::{sync::Arc, time::Duration};

use ethercrab::{
    MainDevice, MainDeviceConfig, PduStorage, Timeouts,
    std::{ethercat_now, tx_rx_task},
};
use interfaces::Interface;

// Constants for EtherCAT configuration
const IFACE_DISCOVERY_MAX_SUBDEVICES: usize = 16; // Must be power of 2 > 1
const IFACE_DISCOVERY_MAX_PDU_DATA: usize = PduStorage::element_size(1100);
const IFACE_DISCOVERY_MAX_FRAMES: usize = 64;
const IFACE_DISCOVERY_MAX_PDI_LEN: usize = 128;

/// Finds an ethernet interface that is suitable for EtherCAT communication.
///
/// ```ignore
/// match discover_ethercat_interface().await {
///     Ok(interface) => println!("Found working interface: {}", interface),
///     Err(_) => println!("No working interface found"),
/// }
/// ```
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
        // Don't call the default hook, which would print the backtrace
    }));

    // Get eligible interfaces
    let mut interfaces = Interface::get_all()
        .or_else(|e| Err(anyhow::anyhow!("Failed to get network interfaces: {}", e)))?
        .into_iter()
        .filter(|iface| {
            iface.is_up()
                && iface.is_running()
                && !iface.is_loopback()
                && !iface.name.starts_with("bridge")
                && !iface.name.starts_with("utun")   // Exclude tunnel interfaces
                && !iface.name.starts_with("awdl")   // Exclude Apple Wireless Direct Link
                && !iface.name.starts_with("anpi")   // Exclude Apple Network Privacy Interface
                && !iface.name.starts_with("llw") // Exclude Apple low-latency WLAN
        })
        .collect::<Vec<_>>();

    // Sort interfaces by name
    interfaces.sort_by(|a, b| a.name.cmp(&b.name));

    // Try all interfaces concurrently
    let tasks = interfaces
        .iter()
        .map(|iface| {
            let name = iface.name.clone();
            std::thread::Builder::new()
                .name(format!("ethercat-test-{}", name))
                .spawn(move || {
                    std::panic::catch_unwind(|| {
                        smol::block_on(async {
                            match test_interface(&name) {
                                Ok(_) => {
                                    tracing::info!("Found working EtherCAT interface: {}", name);
                                    Some(name.clone())
                                }
                                Err(_) => {
                                    tracing::debug!("Interface {} failed", name);
                                    None
                                }
                            }
                        })
                    })
                    .unwrap_or(None)
                })
                .expect("Failed to spawn thread")
        })
        .collect::<Vec<_>>();

    // Wait for all tasks to complete and collect successful results
    let successful_interfaces: Vec<String> = tasks
        .into_iter()
        .filter_map(|task| match task.join().expect("Should join thread") {
            Some(name) => Some(name),
            None => None,
        })
        .collect();

    // Return the first successful interface (they're sorted by name)
    let result = successful_interfaces.into_iter().next();

    // Restore the default panic hook
    std::panic::set_hook(default_hook);

    match result {
        Some(name) => Ok(name),
        None => Err(anyhow::anyhow!("No suitable EtherCAT interface found")),
    }
}

fn test_interface(interface: &str) -> Result<(), anyhow::Error> {
    tracing::trace!("Testing interface: {}", interface);

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
                retry_behaviour: ethercrab::RetryBehaviour::Count(10),
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
