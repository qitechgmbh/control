use std::process::Command;

use anyhow::Result;
use interfaces::Interface;

pub mod modbus_tcp_discovery;

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
