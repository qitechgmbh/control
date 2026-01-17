use std::sync::{Mutex, OnceLock};

use crate::metrics::csv_writer::RuntimeSample;

/// Global storage for the latest runtime metrics sample.
///
/// Used by the REST API to serve fast responses without touching the CSV.
static RUNTIME_LATEST: OnceLock<Mutex<Option<RuntimeSample>>> = OnceLock::new();

fn latest_slot() -> &'static Mutex<Option<RuntimeSample>> {
    RUNTIME_LATEST.get_or_init(|| Mutex::new(None))
}

/// Update the latest runtime metrics sample.
///
/// Clones `sample` into the global slot.
pub fn set_latest_runtime_sample(sample: &RuntimeSample) {
    let mut guard = latest_slot().lock().unwrap();
    *guard = Some(sample.clone());
}

/// Get the latest runtime metrics sample, if any.
///
/// Returns a cloned `RuntimeSample`.
pub fn get_latest_runtime_sample() -> Option<RuntimeSample> {
    let guard = latest_slot().lock().unwrap();
    guard.clone()
}
