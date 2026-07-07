use std::{fs, os::unix::io::AsRawFd};
use qitech_lib::common::get_async_runtime;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio_serial::{SerialPortInfo, available_ports};

const SIOCSIFFLAGS: libc::c_ulong = 0x8914;
const IFF_UP: libc::c_short = 0x1;
// Matching the Linux kernel 'ifreq' structure for ioctl
#[repr(C)]
struct IfReq {
    ifr_name: [u8; libc::IFNAMSIZ],
    ifr_flags: libc::c_short,
}

pub fn bring_up_interface(iface: &str) -> std::io::Result<()> {
    let socket = std::net::UdpSocket::bind("0.0.0.0:0")?;
    let fd = socket.as_raw_fd();
    // Prepare the ifr_name buffer (16 bytes, null-padded)
    let mut ifr_name = [0u8; libc::IFNAMSIZ];
    let bytes = iface.as_bytes();
    if bytes.len() >= libc::IFNAMSIZ {
        return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, "Interface name too long"));
    }
    ifr_name[..bytes.len()].copy_from_slice(bytes);
    let ifr = IfReq {
        ifr_name,
        ifr_flags: IFF_UP,
    };
    unsafe {
        let res = libc::ioctl(fd, SIOCSIFFLAGS, &ifr);
        if res < 0 {
            return Err(std::io::Error::last_os_error());
        }
    }
    println!("Successfully activated: {}", iface);
    Ok(())
}

/// Fails if run by non root user
pub fn set_all_ethernet_up() -> bool {
    // Scan sysfs directory for ethernet devices
    if let Ok(entries) = fs::read_dir("/sys/class/net/") {
        for entry in entries.flatten() {
            if let Ok(name) = entry.file_name().into_string() {
                if name.starts_with("en") || name.starts_with("eth") {
                    if let Err(e) = bring_up_interface(&name) {
                        eprintln!("Error bringing up interface {}: {}", name, e);
                    }
                }
            }
        }
    }
    true
}



pub fn detect_serial(rx: Receiver<()>, tx_ports: Sender<Vec<SerialPortInfo>>) {
    get_async_runtime().spawn(async move {
        let mut rx = rx;
        loop {
            let res = rx.recv().await;
            match res {
                Some(_) => (),
                None => break, // In this case channel is closed, so stop
            }
            let ports = available_ports();
            let ports = match ports {
                Ok(p) => p,
                Err(_e) => vec![],
            };
            let _res = tx_ports.send(ports).await;
        }
    });
}