use std::fs;
use std::path::Path;
use std::sync::OnceLock;

/// Raw byte counters for a network interface.
#[derive(Debug, Clone, Copy)]
pub struct NetDevCounters {
    pub rx_bytes: u64,
    pub tx_bytes: u64,
}

// Global storage for the discovered EtherCAT interface name.
static ETHERCAT_IFACE: OnceLock<String> = OnceLock::new();

/// Record the EtherCAT interface name for later I/O metrics.
pub fn set_ethercat_iface<S: Into<String>>(iface: S) {
    let _ = ETHERCAT_IFACE.set(iface.into());
}

/// Get the EtherCAT interface name if it has been discovered.
pub fn get_ethercat_iface() -> Option<&'static str> {
    ETHERCAT_IFACE.get().map(|s| s.as_str())
}

fn read_u64_from(path: &Path) -> Option<u64> {
    let s = fs::read_to_string(path).ok()?;
    s.trim().parse::<u64>().ok()
}

/// Read rx/tx byte counters for a network interface from /sys/class/net.
///
/// Returns None if the interface or files do not exist.
pub fn read_netdev_counters(iface: &str) -> Option<NetDevCounters> {
    let base = format!("/sys/class/net/{iface}/statistics");
    let rx_path = Path::new(&base).join("rx_bytes");
    let tx_path = Path::new(&base).join("tx_bytes");

    Some(NetDevCounters {
        rx_bytes: read_u64_from(&rx_path)?,
        tx_bytes: read_u64_from(&tx_path)?,
    })
}
