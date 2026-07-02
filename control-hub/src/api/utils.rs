use std::sync::Arc;
use chrono::{DateTime, Utc};
use machine_core::{MachineIdentification, MachineIdentificationUnique, MachineSchema};
use crate::SharedState;

pub fn machine_info_from_slug<'a>(state: &'a Arc<SharedState>, slug: &String) -> Result<(MachineIdentification, &'a MachineSchema), String> {
    let ident = ident_from_slug(state, slug)?;
    let spec = state.machine_specs.get(&ident).expect("machine_slugs and machine_specs are sync'd");
    Ok((ident, spec))
}

pub fn ident_from_slug(state: &Arc<SharedState>, slug: &String) -> Result<MachineIdentification, String> {
    match state.machine_slugs.get(slug) {
        Some(v) => Ok(*v),
        None => Err(format!("No such machine: {slug}")),
    }
}

pub fn init_uid(ident: MachineIdentification, serial: u32) -> u64 {
    MachineIdentificationUnique { ident, serial }.as_u64()
}

pub fn dt_to_ch_datetime64_ms(dt: DateTime<Utc>) -> String {
    let secs = dt.timestamp();
    let nanos = dt.timestamp_subsec_nanos();

    // convert nanos → fractional seconds (up to 9 digits)
    let frac = nanos as f64 / 1_000_000_000.0;

    let value = secs as f64 + frac;

    // format with 3 decimals for DateTime64(3)
    format!("{:.3}", value)
}
