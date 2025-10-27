use std::process::{Command, exit};

/// Load an XDP program onto the given interface.
/// Equivalent to:
///     sudo ip link set <iface> xdp obj <xdp_obj_path> sec xdp
pub fn load_xdp(iface: &str, xdp_obj_path: &str) {
    println!("Loading XDP program '{}' on interface '{}'", xdp_obj_path, iface);

    let status = Command::new("ip")
        .args([
            "link", "set", "dev", iface,
            "xdp", "obj", xdp_obj_path,
            "sec", "xdp",
        ])
        .status()
        .expect("failed to execute ip command");

    if !status.success() {
        eprintln!("Failed to load XDP program (exit code {:?})", status.code());
        exit(1);
    }
}

/// Unload any XDP program from the given interface.
/// Equivalent to:
///     sudo ip link set <iface> xdp off
pub fn unload_xdp(iface: &str) {
    println!("Unloading XDP program from interface '{}'", iface);

    let status = Command::new("ip")
        .args(["link", "set", "dev", iface, "xdp", "off"])
        .status()
        .expect("failed to execute ip command");

    if !status.success() {
        eprintln!("Failed to unload XDP program (exit code {:?})", status.code());
        exit(1);
    }
}