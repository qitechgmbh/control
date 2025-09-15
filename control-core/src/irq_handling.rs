#[cfg(target_os = "linux")]
use libc;
use std::ffi::CString;
use std::io;
use std::str;

#[cfg(target_os = "linux")]
fn read_proc_interrupts() -> Result<String, io::Error> {
    let path = CString::new("/proc/interrupts").unwrap();
    // O_RDONLY
    let fd = unsafe { libc::open(path.as_ptr(), libc::O_RDONLY) };
    if fd < 0 {
        return Err(io::Error::from_raw_os_error(-fd));
    }

    let mut buf: Vec<u8> = Vec::new();
    let mut chunk = [0u8; 8192];

    loop {
        let n = unsafe { libc::read(fd, chunk.as_mut_ptr() as *mut _, chunk.len()) };
        if n < 0 {
            let e = io::Error::from_raw_os_error(-n as i32);
            unsafe { libc::close(fd) };
            return Err(e);
        } else if n == 0 {
            break;
        } else {
            buf.extend_from_slice(&chunk[..n as usize]);
        }
    }
    unsafe { libc::close(fd) };
    Ok(String::from_utf8_lossy(&buf).into_owned())
}

#[cfg(target_os = "linux")]
fn get_interface_irq(proc_content: &str, interface_name: &str) -> Option<u32> {
    for line in proc_content.lines() {
        if line.contains(interface_name) {
            if let Some(colon_pos) = line.find(':') {
                let head = &line[..colon_pos].trim();
                // try parse head as integer IRQ number
                if let Ok(irq) = head.parse::<u32>() {
                    return Some(irq);
                }
            }
        }
    }
    None
}

/// Since kernel 3.0 it’s possible to use the /proc/irq/<IRQ-NUMBER>/smp_affinity_list
/// With comma seperated values
/// This function takes the irq identifier and writes the cpu string
/// into /proc/irq/irq_number/smp_affinity_list
#[cfg(target_os = "linux")]
fn set_irq_affinity_raw(irq: u32, cpu: &str) -> Result<(), io::Error> {
    let path: String = format!("/proc/irq/{}/smp_affinity_list", irq);
    let cpath = CString::new(path.clone()).unwrap();
    // We want to completely overwrite the smp affinities, so that we guarentee execution on the specified cores
    let file_descriptor = unsafe { libc::open(cpath.as_ptr(), libc::O_WRONLY) };

    if file_descriptor < 0 {
        tracing::info!("Permission denied to write: {}", path);
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            format!("failed to open {}: errno {}", path, -file_descriptor),
        ));
    }
    // convert to "raw bytes"
    // cpu needs to be a string because we need the raw ascii code 48 = '0' and so on as bytes
    let bytes = cpu.as_bytes();
    let mut bytes_written_total = 0usize;

    while bytes_written_total < bytes.len() {
        let to_write = &bytes[bytes_written_total..];
        let bytes_written = unsafe {
            libc::write(
                file_descriptor,
                to_write.as_ptr() as *const _,
                to_write.len(),
            )
        };
        if bytes_written < 0 {
            let e = io::Error::from_raw_os_error(-bytes_written as i32);
            unsafe { libc::close(file_descriptor) };
            return Err(e);
        }
        bytes_written_total += bytes_written as usize;
    }
    unsafe { libc::close(file_descriptor) };
    Ok(())
}

/// This is used to make sure that a given irq handler runs on the expected cpu
/// On Realtime systems this is the make or break fix for certain usecases like ethercrab
/// On our machines the 99.99th percentile of cycle times was very bad ~5-11 ms while ethercat is supposed to be in the microseconds
/// After pinning the ethernet irq to the core that the ethercat code ran on 99.99th percentile went down to ~200us (microseconds)
/// Example input: irq_name: "eno1" , cpu: 2
/// Remember that cpu cores are counted from 0
#[cfg(target_os = "linux")]
pub fn set_irq_affinity(irq_name: &str, cpu: u32) -> Result<(), anyhow::Error> {
    let proc_contents = read_proc_interrupts()?;
    let irq = get_interface_irq(&proc_contents, irq_name);
    if irq.is_none() {
        return Err(anyhow::anyhow!("Couldnt find irq number!"));
    }
    let res = set_irq_affinity_raw(irq.unwrap(), &cpu.to_string());
    if res.is_err() {
        return Err(anyhow::anyhow!("Couldnt write affinity list!"));
    } else {
        return Ok(());
    }
}
