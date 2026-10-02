use axum::extract::State;
use qitech_framework::machine::MachineDescriptor;
use serde::Serialize;

use crate::api::legacy::LegacyApiState;
use crate::api::legacy::v2::MachineEntry;
use crate::api::legacy::v2::Result;
use crate::api::legacy::v2::json;
use crate::machines::ExtruderV1;
use crate::machines::ExtruderV2;
use crate::machines::LaserV1;
use crate::machines::WinderV1_7031_Spool;
use crate::machines::WinderV1_Regular;

#[derive(Serialize, Debug, PartialEq)]
pub struct Response {
    machines: Vec<MachineEntry>,
}

pub async fn get(State(LegacyApiState { ctx, .. }): State<LegacyApiState>) -> Result<Response> {
    let machines: Vec<_> = ctx
        .machines
        .load()
        .keys()
        .map(|ident| {
            let vendor = if ident.machine.vendor_id == 1 {
                "QiTech"
            } else {
                "N/A"
            }
            .to_string();

            let slug = match ident.machine {
                WinderV1_Regular::IDENTIFICATION => "winder_v1",
                WinderV1_7031_Spool::IDENTIFICATION => "winder_v1_7031",
                ExtruderV1::IDENTIFICATION => "extruder_v1",
                ExtruderV2::IDENTIFICATION => "extruder_v2",
                LaserV1::IDENTIFICATION => "laser_v1",
                _ => "N/A",
            }
            .to_string();

            MachineEntry {
                legacy_id: ident.clone().into(),
                serial: ident.serial,
                vendor,
                slug,
                error: None,
            }
        })
        .collect();

    json(Response { machines })
}
