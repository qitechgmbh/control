use property::PropertySetView;

use crate::{MachineRegistry, PropertyType, SharedState};

pub mod local;
pub mod remote;

pub async fn update_registry(
    view: PropertySetView<'_>,
    registry: &mut MachineRegistry,
    state: &SharedState,
) {
    let mut did_registry_change = false;

    for entry in view.float {
        if registry.try_insert(entry.ident, &entry.name.into(), PropertyType::Float) {
            did_registry_change = true;
        }
    }

    for entry in view.integer {
        if registry.try_insert(entry.ident, &entry.name.into(), PropertyType::Integer) {
            did_registry_change = true;
        }
    }

    for entry in view.boolean {
        if registry.try_insert(entry.ident, &entry.name.into(), PropertyType::Boolean) {
            did_registry_change = true;
        }
    }

    for entry in view.string {
        if registry.try_insert(entry.ident, &entry.name.into(), PropertyType::String) {
            did_registry_change = true;
        }
    }

    if did_registry_change {
        // only invoke a registry sync when necessary
        let mut registry = state.registry.write().await;
        *registry = registry.clone();
    }
}