use std::sync::Arc;
use axum::{Json, extract::State};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use machine_core::MachineIdentificationUnique;

use crate::SharedState;

pub(crate) async fn handle(
    State(state): State<Arc<SharedState>>,
) -> Result<Json<Vec<Entry>>, String> {
    let mut items: Vec<Entry> = Vec::new();

    for ident in state.machine_registry.load().iter() {
        let uid = MachineIdentificationUnique::from_u64(*ident);
        let name = if let Some(v) = state.machine_specs.get(&uid.ident) {
            &v.name
        } else { "N/A" };

        items.push(Entry {
            name: name.into(),
            vendor: "QiTech GmbH".into(),
            serial: uid.serial,
            last_active: Utc::now(),
        });
    }

    Ok(axum::Json(items))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Entry {
    name: String,
    vendor: String,
    serial: u32,
    last_active: DateTime<Utc>,
}
