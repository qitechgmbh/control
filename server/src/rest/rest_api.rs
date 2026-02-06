use std::sync::Arc;

use axum::extract::{Path, State};
use axum::routing::{get, post};
use axum::{Extension, Json, Router, debug_handler};
use machines::MachineMessage;
use machines::analog_input_test_machine::AnalogInputTestMachine;
use machines::aquapath1::AquaPathV1;
use machines::extruder1::ExtruderV2;
use machines::gluetex::Gluetex;
use machines::ip20_test_machine::IP20TestMachine;
use machines::laser::LaserMachine;
use machines::machine_identification::{MachineIdentification, MachineIdentificationUnique};
use machines::mock::MockMachine;
use machines::test_machine::TestMachine;
use machines::wago_power::WagoPower;
use machines::winder2::Winder2;
use serde::Serialize;

use crate::app_state::SharedState;
use crate::rest::response::*;

#[derive(Serialize, Debug, PartialEq)]
struct MachineResponce {
    legacy_id: MachineIdentificationUnique,
    serial: u16,
    vendor: String,
    slug: String,
    error: Option<String>,
}

impl From<MachineIdentificationUnique> for MachineResponce {
    fn from(machine_identification_unique: MachineIdentificationUnique) -> Self {
        let vendor = machine_identification_unique
            .machine_identification
            .vendor_str();
        let slug = machine_identification_unique.machine_identification.slug();
        let serial = machine_identification_unique.serial;

        Self {
            legacy_id: machine_identification_unique,
            serial,
            vendor,
            slug,
            error: None,
        }
    }
}

#[derive(Serialize, Debug, PartialEq)]
struct GetMachinesResponce {
    machines: Vec<MachineResponce>,
}

#[debug_handler]
async fn get_machines_handler(
    State(shared_state): State<Arc<SharedState>>,
) -> Result<GetMachinesResponce> {
    let machines = shared_state
        .get_machines_meta()
        .await
        .into_iter()
        .map(|m| {
            let vendor = m
                .machine_identification_unique
                .machine_identification
                .vendor_str();
            let slug = m
                .machine_identification_unique
                .machine_identification
                .slug();
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

    json(GetMachinesResponce { machines })
}

#[derive(Serialize, Debug, PartialEq)]
struct GetMachineResponce {
    machine: MachineResponce,
    state: serde_json::Value,
    live_values: serde_json::Value,
}

#[debug_handler]
async fn get_machine_handler(
    Extension(id): Extension<MachineIdentification>,
    State(shared_state): State<Arc<SharedState>>,
    Path(serial): Path<u16>,
) -> Result<GetMachineResponce> {
    let id = MachineIdentificationUnique {
        machine_identification: id,
        serial,
    };

    let (sender, receiver) = smol::channel::bounded(1);
    shared_state
        .message_machine(&id, MachineMessage::RequestValues(sender))
        .await
        .map_err(not_found)?;

    let values = receiver.recv().await.map_err(internal_error)?;

    json(GetMachineResponce {
        machine: MachineResponce::from(id),
        state: values.state,
        live_values: values.live_values,
    })
}

type PostMachineRequest = Vec<serde_json::Value>;

#[debug_handler]
async fn post_machine_handler(
    Extension(id): Extension<MachineIdentification>,
    State(shared_state): State<Arc<SharedState>>,
    Path(serial): Path<u16>,
    Json(request): Json<PostMachineRequest>,
) -> Result<()> {
    let id = MachineIdentificationUnique {
        machine_identification: id,
        serial,
    };

    for value in request {
        shared_state
            .message_machine(&id, MachineMessage::HttpApiJsonRequest(value))
            .await
            .map_err(not_found)?;
    }

    json(())
}

fn make_machine_router(id: MachineIdentification) -> Router<Arc<SharedState>> {
    let slug = id.slug();
    let path = format!("/machine/{slug}/{{serial}}");
    Router::new()
        .route(&path, get(get_machine_handler))
        .route(&path, post(post_machine_handler))
        .layer(Extension(id))
}

pub fn rest_api_router() -> Router<Arc<SharedState>> {
    Router::new()
        .route("/machine", get(get_machines_handler))
        .merge(make_machine_router(LaserMachine::MACHINE_IDENTIFICATION))
        .merge(make_machine_router(Winder2::MACHINE_IDENTIFICATION))
        .merge(make_machine_router(Gluetex::MACHINE_IDENTIFICATION))
        .merge(make_machine_router(MockMachine::MACHINE_IDENTIFICATION))
        .merge(make_machine_router(ExtruderV2::MACHINE_IDENTIFICATION))
        .merge(make_machine_router(AquaPathV1::MACHINE_IDENTIFICATION))
        .merge(make_machine_router(TestMachine::MACHINE_IDENTIFICATION))
        .merge(make_machine_router(WagoPower::MACHINE_IDENTIFICATION))
        .merge(make_machine_router(IP20TestMachine::MACHINE_IDENTIFICATION))
        .merge(make_machine_router(
            AnalogInputTestMachine::MACHINE_IDENTIFICATION,
        ))
}
