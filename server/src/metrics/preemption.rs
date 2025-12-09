use std::fs;
use std::sync::OnceLock;

// Global storage for the RT loop thread ID.
static RT_LOOP_TID: OnceLock<libc::pid_t> = OnceLock::new();

/// Record the realtime loop thread ID (TID) for preemption stats.
pub fn set_rt_loop_tid(tid: libc::pid_t) {
    let _ = RT_LOOP_TID.set(tid);
}

/// Get the realtime loop thread ID (TID), if it has been recorded.
pub fn get_rt_loop_tid() -> Option<libc::pid_t> {
    RT_LOOP_TID.get().copied()
}

/// Per-thread scheduling / preemption-related stats from /proc/<pid>/task/<tid>/sched.
#[derive(Debug, Clone)]
pub struct ThreadSchedStats {
    pub tid: libc::pid_t,
    pub nr_switches: u64,
    pub nr_voluntary_switches: u64,
    pub nr_involuntary_switches: u64,
}

/// Read basic scheduling stats for a thread (by TID) from /proc/<tid>/sched.
///
/// Returns None if the file can't be read or parsed.
pub fn read_thread_sched_stats(tid: libc::pid_t) -> Option<ThreadSchedStats> {
    // /proc/<tid>/sched is a shortcut for /proc/self/task/<tid>/sched
    let path = format!("/proc/self/task/{tid}/sched");
    let contents = fs::read_to_string(path).ok()?;

    let mut nr_switches = None;
    let mut nr_voluntary = None;
    let mut nr_involuntary = None;

    for line in contents.lines() {
        if let Some(rest) = line.strip_prefix("nr_switches") {
            nr_switches = rest.split_whitespace().last()?.parse().ok();
        } else if let Some(rest) = line.strip_prefix("nr_voluntary_switches") {
            nr_voluntary = rest.split_whitespace().last()?.parse().ok();
        } else if let Some(rest) = line.strip_prefix("nr_involuntary_switches") {
            nr_involuntary = rest.split_whitespace().last()?.parse().ok();
        }
    }

    Some(ThreadSchedStats {
        tid,
        nr_switches: nr_switches?,
        nr_voluntary_switches: nr_voluntary?,
        nr_involuntary_switches: nr_involuntary?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_read_current_thread_sched_stats() {
        // Safety: gettid is safe to call and returns the current thread ID.
        let tid = unsafe { libc::syscall(libc::SYS_gettid) as libc::pid_t };
        let stats = read_thread_sched_stats(tid)
            .expect("should be able to read /proc/<tid>/sched for current thread");
        assert!(stats.nr_switches >= stats.nr_voluntary_switches);
        assert!(stats.nr_switches >= stats.nr_involuntary_switches);
    }
}
