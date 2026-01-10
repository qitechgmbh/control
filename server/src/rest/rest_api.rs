use std::sync::Arc;

use axum::{Router, debug_handler};
use axum::extract::{Path, State};
use axum::routing::get;
use machines::MachineMessage;
use machines::machine_identification::MachineIdentificationUnique;
use machines::winder2::Winder2;
use machines::winder2::api::{LiveValuesEvent, StateEvent, Winder2Events};
use serde::{Deserialize, Serialize};

use crate::app_state::SharedState;
use crate::rest::response::*;

#[derive(Deserialize, Serialize, Debug, PartialEq)]
struct MachineResponce {
    legacy_id: MachineIdentificationUnique,
    serial: u16,
    vendor: String,
    slug: String,
    error: Option<String>,
}

impl From<MachineIdentificationUnique> for MachineResponce {

    fn from(machine_identification_unique: MachineIdentificationUnique) -> Self {
        let vendor = machine_identification_unique.machine_identification.vendor_str();
        let slug = machine_identification_unique.machine_identification.slug();
        let serial = machine_identification_unique.serial;

        MachineResponce {
            legacy_id: machine_identification_unique,
            serial,
            vendor,
            slug,
            error: None,
        }
    }
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
struct GetMachinesResponce {
    machines: Vec<MachineResponce>
}

#[debug_handler]
async fn get_machines_handler(
    State(shared_state): State<Arc<SharedState>>,
) -> Result<GetMachinesResponce> {

    let machines = shared_state.get_machines_meta().await.into_iter().map(|m| {
        let vendor = m.machine_identification_unique.machine_identification.vendor_str();
        let slug = m.machine_identification_unique.machine_identification.slug();
        let serial = m.machine_identification_unique.serial;

        MachineResponce {
            legacy_id: m.machine_identification_unique,
            serial,
            vendor,
            slug,
            error: m.error,
        }
    })
    .collect();

    json(GetMachinesResponce {
        machines
    })
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
struct GetMachineResponce {
    machine: MachineResponce,
    state: serde_json::Value,
    live_values: serde_json::Value,
}

#[debug_handler]
async fn get_winder_v1_handler(
    Path(serial): Path<u16>,
    State(shared_state): State<Arc<SharedState>>,
) -> Result<GetMachineResponce> {

    let id = MachineIdentificationUnique {
        machine_identification: Winder2::MACHINE_IDENTIFICATION,
        serial,
    };

    let (sender, receiver) = smol::channel::unbounded();
    shared_state.message_machine(&id, MachineMessage::RequestValues(sender)).await.map_err(not_found)?;

    let values = receiver.recv().await.map_err(internal_error)?;

    json(GetMachineResponce {
        machine: MachineResponce::from(id),
        state: values.state,
        live_values: values.live_values,
    })
}

pub fn rest_api_router() -> Router<Arc<SharedState>> {
    Router::new()
        .route("/machine", get(get_machines_handler))
        .route("/machine/winder_v1/{serial}", get(get_winder_v1_handler))
}
