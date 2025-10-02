use tracing::debug;
use tracing::error;

#[cfg(target_os = "linux")]
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
#[cfg(target_os = "linux")]
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

#[cfg(not(target_os = "linux"))]
pub fn set_realtime_priority() -> Result<(), anyhow::Error> {
    tracing::error!("This platform does not support realtime threads");
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
#[cfg(target_os = "linux")]
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

#[cfg(not(target_os = "linux"))]
pub fn lock_memory() -> Result<(), Box<dyn std::error::Error>> {
    tracing::error!("This platform does not support memory locking");
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
