use tracing::debug;
use tracing::error;

use libc;
use std::ffi::CString;
use std::io;
use std::str;

#[cfg(unix)]
/// Makes the current thread a real-time thread using the PREEMPT_RT capabilities of Linux.
///
/// This function configures the current thread with real-time scheduling properties by:
///
/// 1. Setting the scheduling policy to SCHED_FIFO (First-In-First-Out), which is a real-time
///    scheduling policy that allows the thread to run until it voluntarily yields, blocks on I/O,
///    or is preempted by a higher-priority real-time thread.
///
/// 2. Setting the real-time priority to 80 (on a scale of 1-99, where higher values indicate
///    higher priority). This priority level is chosen to be higher than most IRQ handlers
///    (typically priority 50) but lower than critical kernel tasks (priority 99).
///
/// When running on a PREEMPT_RT patched Linux kernel, this combination provides deterministic
/// scheduling with minimal latency, which is essential for real-time applications.
///
/// # Requirements
///
/// - The program must run with appropriate permissions to set real-time priorities.
///   This typically means either:
///   - Running as root
///   - Configuring `/etc/security/limits.conf` to allow the user to set real-time priorities
///
/// - The system should be running a Linux kernel with PREEMPT_RT patch applied for
///   optimal real-time performance.
///
/// # Example
///
/// ```ignore
/// fn main() {
///     // Lock memory to prevent page faults
///     lock_memory();
///     
///     // Create a thread and make it real-time
///     std::thread::spawn(|| {
///         set_realtime_priority();
///         
///         // Real-time code here
///     });
/// }
/// ```
///
/// # Returns
///
/// - `Ok(())` if the real-time priority was successfully set
/// - `Err` with the error message if setting the priority failed
#[cfg(unix)]
pub fn set_realtime_priority() -> Result<(), anyhow::Error> {
    use anyhow::anyhow;
    use libc::{SCHED_FIFO, pthread_setschedparam, sched_param};
    use std::mem;

    unsafe {
        // Get the pthread_t of the current thread using pthread_self()
        let pthread_self = libc::pthread_self();

        // Set up the scheduling parameters
        let mut param: sched_param = mem::zeroed();
        param.sched_priority = 95;

        // Set the thread to use SCHED_FIFO scheduling policy with our priority
        let result = pthread_setschedparam(pthread_self, SCHED_FIFO, &param);

        if result != 0 {
            error!(
                "Failed to set real-time scheduling policy for thread \"{:?}\": {}",
                std::thread::current().name().unwrap_or("unknown"),
                std::io::Error::last_os_error()
            );
            return Err(anyhow!(
                "Failed to set real-time priority: {}",
                std::io::Error::last_os_error()
            ));
        } else {
            debug!(
                "Successfully set real-time priority for the thread \"{:?}\"",
                std::thread::current().name().unwrap_or("unknown")
            );
        }
    }
    Ok(())
}

#[cfg(not(unix))]
pub fn set_realtime_priority() -> Result<(), anyhow::Error> {
    log::error!("This platform does not support realtime threads");
    Ok(())
}

/// Locks all current and future memory pages of the process into RAM to prevent page faults.
///
/// # When to call this function
///
/// This function should be called from the main thread, as early as possible in your program's
/// execution, ideally before creating any threads and before allocating large amounts of memory.
///
/// Important: You only need to call this function ONCE per process, not in each thread.
/// The memory locking applies to the entire process, including:
/// - The main thread
/// - All threads created after the call
///
/// This happens because all threads in a process share the same virtual memory space, and
/// `mlockall()` operates on the entire process's memory space, not just the calling thread.
///
/// # Example: Correct usage
///
/// ```ignore
/// fn main() -> Result<(), Box<dyn std::error::Error>> {
///     // Call early in main, before creating threads
///     lock_memory()?;
///     
///     // Now create real-time threads...
///     std::thread::spawn(|| {
///         set_realtime_priority();
///         // Real-time code here
///     });
///     
///     Ok(())
/// }
/// ```
///
/// # Example: Incorrect usage
///
/// ```ignore
/// fn main() {
///     // Don't do this - calling in each thread is unnecessary
///     std::thread::spawn(|| {
///         lock_memory().unwrap(); // Redundant if already called in main
///         set_realtime_priority();
///     });
/// }
/// ```
///
/// # How it works
///
/// [Rest of documentation as before...]
#[cfg(unix)]
pub fn lock_memory() -> Result<(), Box<dyn std::error::Error>> {
    use libc::{MCL_FUTURE, mlockall};

    unsafe {
        if mlockall(MCL_FUTURE) != 0 {
            error!("Failed to lock memory: {}", std::io::Error::last_os_error());
            return Err(
                format!("Failed to lock memory: {}", std::io::Error::last_os_error()).into(),
            );
        } else {
            debug!("Successfully locked memory for the process");
        }
    }
    Ok(())
}

#[cfg(not(unix))]
pub fn lock_memory() -> Result<(), Box<dyn std::error::Error>> {
    log::error!("This platform does not support memory locking");
    Ok(())
}

pub fn set_core_affinity(core_id: usize) -> Result<(), anyhow::Error> {
    let cores = core_affinity::get_core_ids().unwrap_or_default();
    let core = cores.get(core_id);
    if let Some(core) = core {
        core_affinity::set_for_current(*core);
        debug!(
            "Set core affinity of thread \"{:?}\" to core {:?}",
            std::thread::current().name().unwrap_or("unknown"),
            core
        );
        Ok(())
    } else {
        error!("No cores available to set affinity");
        Err(anyhow::anyhow!("No cores available to set affinity"))
    }
}

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

fn find_irq_for_interface(proc_content: &str, interface_name: &str) -> Option<u32> {
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

fn write_smp_affinity_list(irq: u32, cpu_list: &str) -> Result<(), io::Error> {
    let path: String = format!("/proc/irq/{}/smp_affinity_list", irq);
    let cpath = CString::new(path.clone()).unwrap();
    // We want to completely overwrite the smp affinities, so that we guarentee execution on the specified cores
    let fd = unsafe { libc::open(cpath.as_ptr(), libc::O_WRONLY) };

    if fd < 0 {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            format!("failed to open {}: errno {}", path, -fd),
        ));
    }
    // convert to "raw bytes"
    let bytes = cpu_list.as_bytes();
    let mut bytes_written_total = 0usize;

    while bytes_written_total < bytes.len() {
        let to_write = &bytes[bytes_written_total..];
        let bytes_written =
            unsafe { libc::write(fd, to_write.as_ptr() as *const _, to_write.len()) };
        if bytes_written < 0 {
            let e = io::Error::from_raw_os_error(-bytes_written as i32);
            unsafe { libc::close(fd) };
            return Err(e);
        }
        bytes_written_total += bytes_written as usize;
    }
    unsafe { libc::close(fd) };
    Ok(())
}

/// This is used to make sure that a given irq handler runs on the expected cpu
/// On Realtime systems this is the make or break fix for certain usecases like ethercrab
/// On our machines the 99.99th percentile of cycle times was very bad ~5-11 ms while ethercat is supposed to be in the microseconds
/// After pinning the ethernet irq to the core that the ethercat code ran on 99.99th percentile went down to ~200us (microseconds)
/// Example input: irq_name: "eno1" , cpu_list: "2"
/// Remember that cpu cores are counted from 0
pub fn set_irq_handler_affinity(irq_name: &str, cpu_list: &str) -> Result<(), anyhow::Error> {
    let proc_contents = read_proc_interrupts()?;
    let irq = find_irq_for_interface(&proc_contents, irq_name);
    if irq.is_none() {
        return Err(anyhow::anyhow!("Couldnt find irq number!"));
    }
    let res = write_smp_affinity_list(irq.unwrap(), cpu_list);
    if res.is_err() {
        return Err(anyhow::anyhow!("Couldnt write affinity list!"));
    } else {
        return Ok(());
    }
}
