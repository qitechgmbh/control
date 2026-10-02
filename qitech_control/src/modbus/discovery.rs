//! Discovery of the USB serial ports a Modbus RTU device can be reached on.
//!
//! The framework only resolves a single stored `/dev/serial/by-path` name when it builds a device;
//! enumerating what is currently plugged in - and the USB metadata that lets a user tell two
//! identical adapters apart on the Setup page - is control's job.

use std::collections::HashMap;
use std::collections::HashSet;
use std::fs;

const BY_PATH: &str = "/dev/serial/by-path";
const BY_ID: &str = "/dev/serial/by-id";

/// USB identity of a serial adapter, as reported by the `serialport` crate.
struct UsbMeta {
    vid: u16,
    pid: u16,
    serial: Option<String>,
    description: Option<String>,
}

/// One USB serial port as physically discovered, before joining against any assignment.
pub struct SerialPortInfo {
    /// The canonical name for this port, and the key a new assignment is stored under.
    pub port: String,
    pub aliases: Vec<String>,
    pub device_node: String,
    pub by_id: Option<String>,
    pub description: Option<String>,
    pub usb_vid: Option<u16>,
    pub usb_pid: Option<u16>,
    pub usb_serial: Option<String>,
}

/// Enumerate the USB serial ports currently plugged in, one entry per physical port, enriched with
/// USB metadata (vendor/product id, serial number, description) and the `/dev/serial/by-id` alias
/// where available.
pub fn list_serial_ports() -> Vec<SerialPortInfo> {
    let mut usb_by_node = usb_metadata_by_node();

    let by_id_by_node: HashMap<String, String> = read_links(BY_ID)
        .into_iter()
        .map(|(name, node)| (node, name))
        .collect();

    let mut aliases_by_node = group_by_node(read_links(BY_PATH));

    if aliases_by_node.is_empty() {
        aliases_by_node = device_node_fallback(usb_by_node.keys().cloned().collect());
    }

    let mut ports: Vec<SerialPortInfo> = aliases_by_node
        .into_iter()
        .map(|(device_node, aliases)| {
            let usb = usb_by_node.remove(&device_node);

            SerialPortInfo {
                port: aliases[0].clone(),
                by_id: by_id_by_node.get(&device_node).cloned(),
                description: usb.as_ref().and_then(|u| u.description.clone()),
                usb_vid: usb.as_ref().map(|u| u.vid),
                usb_pid: usb.as_ref().map(|u| u.pid),
                usb_serial: usb.and_then(|u| u.serial),
                aliases,
                device_node,
            }
        })
        .collect();

    // --- map iteration order is arbitrary; keep the list stable between rescans ---
    ports.sort_by(|a, b| a.aliases.cmp(&b.aliases));
    ports
}

fn usb_metadata_by_node() -> HashMap<String, UsbMeta> {
    serialport::available_ports()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|p| {
            let serialport::SerialPortType::UsbPort(usb) = p.port_type else {
                return None;
            };

            Some((
                p.port_name,
                UsbMeta {
                    vid: usb.vid,
                    pid: usb.pid,
                    serial: usb.serial_number,
                    description: usb.product.or(usb.manufacturer),
                },
            ))
        })
        .collect()
}

/// Read a `/dev/serial/*` directory as `(link name, resolved device node)` pairs, skipping links
/// that no longer resolve. A missing directory yields no pairs.
fn read_links(dir: &str) -> Vec<(String, String)> {
    fs::read_dir(dir)
        .into_iter()
        .flatten()
        .filter_map(|entry| entry.ok())
        .filter_map(|entry| {
            let node = fs::canonicalize(entry.path()).ok()?;
            Some((
                entry.file_name().to_string_lossy().into_owned(),
                node.to_string_lossy().into_owned(),
            ))
        })
        .collect()
}

/// Group link names by the device node they resolve to, so a port with several `by-path` links is
/// listed once rather than once per link. The canonical name (per [`by_path_rank`]) comes first;
/// the rest are kept as aliases so assignments stored under them keep resolving.
fn group_by_node(links: Vec<(String, String)>) -> HashMap<String, Vec<String>> {
    let mut by_node: HashMap<String, Vec<String>> = HashMap::new();

    for (name, node) in links {
        by_node.entry(node).or_default().push(name);
    }

    for names in by_node.values_mut() {
        names.sort_by(|a, b| by_path_rank(a).cmp(&by_path_rank(b)).then_with(|| a.cmp(b)));
    }

    by_node
}

/// Sort key picking the canonical name among several `by-path` links to the same device node. The
/// legacy `...-usb-...` spelling wins over systemd's newer `...-usbv2-...` one because it is
/// emitted by both old and new systemd, so an assignment keyed under it survives an OS upgrade in
/// either direction. It is also the name the framework can resolve when it builds the device.
fn by_path_rank(name: &str) -> u8 {
    u8::from(name.contains("-usbv"))
}

/// Without `/dev/serial/by-path` (macOS, or a Linux box with no USB serial adapter plugged in) the
/// device node is the only name available. It does not survive replugging into a different USB
/// port, but it keeps the Setup page populated during development.
///
/// macOS exposes every adapter twice, as `/dev/tty.*` and `/dev/cu.*`; only the callout (`cu.`)
/// node is listed.
fn device_node_fallback(nodes: Vec<String>) -> HashMap<String, Vec<String>> {
    let all: HashSet<String> = nodes.iter().cloned().collect();

    nodes
        .into_iter()
        .filter(|node| {
            let callout = node.replace("/tty.", "/cu.");
            &callout == node || !all.contains(&callout)
        })
        .map(|node| (node.clone(), vec![node]))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::device_node_fallback;
    use super::group_by_node;

    fn link(name: &str, node: &str) -> (String, String) {
        (name.to_string(), node.to_string())
    }

    #[test]
    fn one_entry_per_device_node_with_the_legacy_name_first() {
        let by_node = group_by_node(vec![
            link("pci-0000:00:14.0-usbv2-0:2:1.0-port0", "/dev/ttyUSB0"),
            link("pci-0000:00:14.0-usb-0:2:1.0-port0", "/dev/ttyUSB0"),
            link("pci-0000:00:14.0-usb-0:3:1.0-port0", "/dev/ttyUSB1"),
        ]);

        assert_eq!(by_node.len(), 2);
        assert_eq!(
            by_node["/dev/ttyUSB0"],
            vec![
                "pci-0000:00:14.0-usb-0:2:1.0-port0",
                "pci-0000:00:14.0-usbv2-0:2:1.0-port0",
            ]
        );
        assert_eq!(
            by_node["/dev/ttyUSB1"],
            vec!["pci-0000:00:14.0-usb-0:3:1.0-port0"]
        );
    }

    #[test]
    fn fallback_keeps_the_callout_node_of_a_macos_pair() {
        let by_node = device_node_fallback(vec![
            "/dev/tty.usbserial-1420".to_string(),
            "/dev/cu.usbserial-1420".to_string(),
            "/dev/tty.usbmodem101".to_string(),
        ]);

        let mut nodes: Vec<&String> = by_node.keys().collect();
        nodes.sort();
        assert_eq!(
            nodes,
            vec!["/dev/cu.usbserial-1420", "/dev/tty.usbmodem101"]
        );
    }

    #[test]
    fn fallback_leaves_linux_device_nodes_alone() {
        let by_node =
            device_node_fallback(vec!["/dev/ttyUSB0".to_string(), "/dev/ttyUSB1".to_string()]);

        assert_eq!(by_node.len(), 2);
        assert_eq!(by_node["/dev/ttyUSB0"], vec!["/dev/ttyUSB0"]);
    }
}
