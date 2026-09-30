use std::fs;
use std::os::unix::io::AsRawFd;

const SIOCSIFFLAGS: libc::c_ulong = 0x8914;
const IFF_UP: libc::c_short = 0x1;

/// Fails if run by non root user
pub fn bring_up_all_ethernet() {
    // Scan sysfs directory for ethernet devices
    let Ok(entries) = fs::read_dir("/sys/class/net/") else {
        return;
    };

    for entry in entries.flatten() {
        let Ok(name) = entry.file_name().into_string() else {
            continue;
        };

        if !(name.starts_with("en") || name.starts_with("eth")) {
            continue;
        }

        if let Err(e) = bring_up_interface(&name) {
            eprintln!("Error bringing up interface {}: {}", name, e);
        }
    }
}

fn bring_up_interface(iface: &str) -> std::io::Result<()> {
    let socket = std::net::UdpSocket::bind("0.0.0.0:0")?;
    let fd = socket.as_raw_fd();
    // Prepare the ifr_name buffer (16 bytes, null-padded)
    let mut ifr_name = [0u8; libc::IFNAMSIZ];
    let bytes = iface.as_bytes();
    if bytes.len() >= libc::IFNAMSIZ {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Interface name too long",
        ));
    }
    ifr_name[..bytes.len()].copy_from_slice(bytes);

    // Matching the Linux kernel 'ifreq' structure for ioctl
    #[repr(C)]
    struct IfReq {
        ifr_name: [u8; libc::IFNAMSIZ],
        ifr_flags: libc::c_short,
    }

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
