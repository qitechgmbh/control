use std::cell::UnsafeCell;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicUsize, Ordering};

const JITTER_RING_LEN: usize = 1024;
#[derive(Debug, Clone, Copy)]
pub struct JitterSample {
    /// Signed jitter in nanoseconds.
    /// Negative = early, 0 = on time, positive = late.
    pub jitter_ns: i128,
}

/// Fixed-size ring buffer for jitter samples.
///
/// We use `UnsafeCell` for interior mutability so we can write through `&self`
/// while only synchronizing the index with atomics. For metrics this is fine:
/// occasional races on reading partially-written samples are acceptable.
struct JitterRing {
    write_idx: AtomicUsize,
    buf: UnsafeCell<[JitterSample; JITTER_RING_LEN]>,
}

unsafe impl Sync for JitterRing {}

impl JitterRing {
    const fn new() -> Self {
        const ZERO: JitterSample = JitterSample { jitter_ns: 0 };
        Self {
            write_idx: AtomicUsize::new(0),
            buf: UnsafeCell::new([ZERO; JITTER_RING_LEN]),
        }
    }

    fn push(&self, sample: JitterSample) {
        let idx = self.write_idx.fetch_add(1, Ordering::Relaxed) % JITTER_RING_LEN;

        // SAFETY: single writer pattern; occasional torn reads are fine for diagnostics.
        unsafe {
            (*self.buf.get())[idx] = sample;
        }
    }

    fn snapshot(&self) -> Vec<JitterSample> {
        // SAFETY: best-effort copy; readers may race with a writer, which is acceptable.
        let buf = unsafe { &*self.buf.get() };
        buf.to_vec()
    }
}

static MACHINES_JITTER_RING: OnceLock<JitterRing> = OnceLock::new();

fn machines_ring() -> &'static JitterRing {
    MACHINES_JITTER_RING.get_or_init(JitterRing::new)
}

/// Record one jitter sample for the main RT loop (machines + EtherCAT).
pub fn record_machines_loop_jitter(jitter_ns: i128) {
    machines_ring().push(JitterSample { jitter_ns });
}

/// Get a snapshot of recent jitter samples for the machines loop.
pub fn snapshot_machines_jitter() -> Vec<JitterSample> {
    machines_ring().snapshot()
}
