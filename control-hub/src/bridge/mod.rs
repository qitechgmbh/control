use std::collections::HashSet;

use machine_core::property::PropertyBatch;

// pub mod local;
pub mod remote;

pub fn update_registry(batch: &PropertyBatch, registry: &mut HashSet<u64>) -> bool {
    let mut changed = false;

    for prop in &batch.floats {
        changed |= registry.insert(prop.ident);
    }

    for prop in &batch.integers {
        changed |= registry.insert(prop.ident);
    }

    changed
}
