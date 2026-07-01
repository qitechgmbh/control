use std::collections::HashSet;
use machine_core::property::PropertySetView;

// pub mod local;
pub mod remote;

pub fn update_registry(view: PropertySetView<'_>, registry: &mut HashSet<u64>) -> bool {
    let mut changed = false;

    for prop in view.float {
        changed |= registry.insert(prop.ident);
    }

    for prop in view.integer {
        changed |= registry.insert(prop.ident);
    }

    changed
}
