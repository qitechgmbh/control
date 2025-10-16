use std::sync::Arc;

use anyhow::{Context, anyhow};
use axum::{
    body::Body,
    extract::{Path, State},
    http::Response,
};
use control_core::machines::Machine;
use control_core::machines::connection::MachineConnection;
use control_core::machines::identification::{MachineIdentification, MachineIdentificationUnique};
use control_core::machines::manager::MachineManager;
use control_core::socketio::event::GenericEvent;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::app_state::AppState;
use crate::rest::util::{ResponseUtil, ResponseUtilError};

#[derive(Debug, Deserialize)]
pub struct MachineSimplePath {
    pub identifier: String,
}

#[derive(Debug, Deserialize)]
pub struct MachineSimpleWithSerialPath {
    pub identifier: String,
    pub serial: u16,
}

#[derive(Debug, Serialize, Clone)]
pub struct EventPayload {
    pub name: String,
    pub ts: u64,
    pub data: Value,
}

#[derive(Debug, Serialize, Clone)]
pub struct MachineSnapshot {
    pub machine_identification_unique: MachineIdentificationUnique,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<EventPayload>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub live: Option<EventPayload>,
}

#[axum::debug_handler]
pub async fn get_machine_state_simple(
    State(app_state): State<Arc<AppState>>,
    Path(params): Path<MachineSimplePath>,
) -> Response<Body> {
    handle_machine_request_simple(app_state, params.identifier, None, RequestedKind::State).await
}

#[axum::debug_handler]
pub async fn get_machine_state_simple_with_serial(
    State(app_state): State<Arc<AppState>>,
    Path(params): Path<MachineSimpleWithSerialPath>,
) -> Response<Body> {
    handle_machine_request_simple(
        app_state,
        params.identifier,
        Some(params.serial),
        RequestedKind::State,
    )
    .await
}

#[axum::debug_handler]
pub async fn get_machine_live_simple(
    State(app_state): State<Arc<AppState>>,
    Path(params): Path<MachineSimplePath>,
) -> Response<Body> {
    handle_machine_request_simple(app_state, params.identifier, None, RequestedKind::Live).await
}

#[axum::debug_handler]
pub async fn get_machine_live_simple_with_serial(
    State(app_state): State<Arc<AppState>>,
    Path(params): Path<MachineSimpleWithSerialPath>,
) -> Response<Body> {
    handle_machine_request_simple(
        app_state,
        params.identifier,
        Some(params.serial),
        RequestedKind::Live,
    )
    .await
}

#[axum::debug_handler]
pub async fn get_machine_snapshot_simple(
    State(app_state): State<Arc<AppState>>,
    Path(params): Path<MachineSimplePath>,
) -> Response<Body> {
    handle_machine_request_simple(app_state, params.identifier, None, RequestedKind::All).await
}

#[axum::debug_handler]
pub async fn get_machine_snapshot_simple_with_serial(
    State(app_state): State<Arc<AppState>>,
    Path(params): Path<MachineSimpleWithSerialPath>,
) -> Response<Body> {
    handle_machine_request_simple(
        app_state,
        params.identifier,
        Some(params.serial),
        RequestedKind::All,
    )
    .await
}

#[derive(Copy, Clone)]
enum RequestedKind {
    State,
    Live,
    All,
}

async fn handle_machine_request_simple(
    app_state: Arc<AppState>,
    identifier: String,
    serial_override: Option<u16>,
    kind: RequestedKind,
) -> Response<Body> {
    match resolve_identifier_to_unique(&app_state, &identifier, serial_override).await {
        Ok(unique) => handle_machine_state_request(app_state, unique, kind).await,
        Err(err) => err.into(),
    }
}

async fn handle_machine_state_request(
    app_state: Arc<AppState>,
    unique: MachineIdentificationUnique,
    kind: RequestedKind,
) -> Response<Body> {
    if !app_state.is_machine_api_enabled() {
        return ResponseUtilError::NotFound(anyhow!("Machine read API is disabled")).into();
    }

    match collect_machine_snapshot(&app_state, &unique).await {
        Ok(snapshot) => match kind {
            RequestedKind::State => match snapshot.state {
                Some(state) => ResponseUtil::ok(MachineSnapshot {
                    machine_identification_unique: unique,
                    state: Some(state),
                    live: None,
                }),
                None => ResponseUtilError::NotFound(anyhow!(
                    "No state event available for machine {}",
                    unique
                ))
                .into(),
            },
            RequestedKind::Live => match snapshot.live {
                Some(live) => ResponseUtil::ok(MachineSnapshot {
                    machine_identification_unique: unique,
                    state: None,
                    live: Some(live),
                }),
                None => ResponseUtilError::NotFound(anyhow!(
                    "No live event available for machine {}",
                    unique
                ))
                .into(),
            },
            RequestedKind::All => {
                if snapshot.state.is_none() && snapshot.live.is_none() {
                    ResponseUtilError::NotFound(anyhow!(
                        "No events available for machine {}",
                        unique
                    ))
                    .into()
                } else {
                    ResponseUtil::ok(MachineSnapshot {
                        machine_identification_unique: unique,
                        state: snapshot.state,
                        live: snapshot.live,
                    })
                }
            }
        },
        Err(err) => err.into(),
    }
}

async fn collect_machine_snapshot(
    app_state: &Arc<AppState>,
    unique: &MachineIdentificationUnique,
) -> Result<MachineSnapshot, ResponseUtilError> {
    let machine = resolve_machine(app_state, unique).await?;

    let namespace = {
        let mut guard = machine.lock().await;
        guard.api_event_namespace()
    };

    let namespace_guard = namespace.lock().await;
    let state = namespace_guard
        .events
        .get("StateEvent")
        .and_then(|events| events.last())
        .map(|event| convert_generic_event(event.as_ref()))
        .transpose()
        .map_err(ResponseUtilError::Error)?;
    let live = namespace_guard
        .events
        .get("LiveValuesEvent")
        .and_then(|events| events.last())
        .map(|event| convert_generic_event(event.as_ref()))
        .transpose()
        .map_err(ResponseUtilError::Error)?;

    Ok(MachineSnapshot {
        machine_identification_unique: unique.clone(),
        state,
        live,
    })
}

async fn resolve_machine(
    app_state: &Arc<AppState>,
    unique: &MachineIdentificationUnique,
) -> Result<Arc<smol::lock::Mutex<dyn Machine>>, ResponseUtilError> {
    let machines_guard = app_state.machines.read().await;
    let slot = machines_guard
        .get(unique)
        .ok_or_else(|| ResponseUtilError::NotFound(anyhow!("Machine {} not found", unique)))?;

    let slot_guard = slot.lock_blocking();
    match &slot_guard.machine_connection {
        MachineConnection::Connected(machine) => Ok(machine.clone()),
        MachineConnection::Error(error) => Err(ResponseUtilError::Error(anyhow!(
            "Machine {} has error: {}",
            unique,
            error
        ))),
        MachineConnection::Disconnected => Err(ResponseUtilError::Error(anyhow!(
            "Machine {} is disconnected",
            unique
        ))),
    }
}

fn convert_generic_event(event: &GenericEvent) -> Result<EventPayload, anyhow::Error> {
    let json = serde_json::to_value(event)?;
    let name = json
        .get("name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow!("Event is missing name field"))?
        .to_string();
    let ts = json
        .get("ts")
        .and_then(|v| v.as_u64())
        .ok_or_else(|| anyhow!("Event is missing timestamp"))?;
    let data = json.get("data").cloned().unwrap_or(Value::Null);

    Ok(EventPayload { name, ts, data })
}

async fn resolve_identifier_to_unique(
    app_state: &Arc<AppState>,
    identifier: &str,
    serial_override: Option<u16>,
) -> Result<MachineIdentificationUnique, ResponseUtilError> {
    let (machine_identification, serial_from_identifier) =
        parse_identifier(identifier).map_err(ResponseUtilError::NotFound)?;

    let machines_guard = app_state.machines.read().await;

    let serial = if let Some(serial) = serial_override.or(serial_from_identifier) {
        serial
    } else {
        infer_serial(&machines_guard, &machine_identification)?
    };

    let unique = MachineIdentificationUnique {
        machine_identification: machine_identification.clone(),
        serial,
    };

    if machines_guard.get(&unique).is_none() {
        return Err(ResponseUtilError::NotFound(anyhow!(
            "Machine {} not found. If multiple machines exist, specify the serial using '/api/{identifier}/{serial}'.",
            unique
        )));
    }

    drop(machines_guard);

    Ok(unique)
}

fn infer_serial(
    machines: &MachineManager,
    machine_identification: &MachineIdentification,
) -> Result<u16, ResponseUtilError> {
    let mut matching_serials: Vec<u16> = machines
        .iter()
        .filter_map(|(id, _)| {
            if &id.machine_identification == machine_identification {
                Some(id.serial)
            } else {
                None
            }
        })
        .collect();

    matching_serials.sort_unstable();
    matching_serials.dedup();

    match matching_serials.len() {
        0 => Err(ResponseUtilError::NotFound(anyhow!(
            "No machine registered for vendor {} machine {}.",
            machine_identification.vendor,
            machine_identification.machine
        ))),
        1 => Ok(matching_serials[0]),
        _ => Err(ResponseUtilError::NotFound(anyhow!(
            "Multiple machines found for vendor {} machine {}. Specify the serial using '/api/{{identifier}}/{{serial}}/...'.",
            machine_identification.vendor,
            machine_identification.machine
        ))),
    }
}

fn parse_identifier(
    identifier: &str,
) -> Result<(MachineIdentification, Option<u16>), anyhow::Error> {
    if let Ok(machine_identification) = resolve_machine_identifier(identifier) {
        return Ok((machine_identification, None));
    }

    for separator in ['-', ':', '.', '_'] {
        if let Some((base, serial_part)) = identifier.rsplit_once(separator) {
            if let Ok(serial) = parse_u16(serial_part) {
                if let Ok(machine_identification) = resolve_machine_identifier(base) {
                    return Ok((machine_identification, Some(serial)));
                }
            }
        }
    }

    Err(anyhow!(
        "Unable to parse machine identifier '{}'. Expected a known slug or 'vendor-machine' format, optionally followed by a serial (e.g. 'winder2-42').",
        identifier
    ))
}

fn resolve_machine_identifier(identifier: &str) -> Result<MachineIdentification, anyhow::Error> {
    match identifier {
        "winder2" => Ok(MachineIdentification {
            vendor: crate::machines::VENDOR_QITECH,
            machine: crate::machines::MACHINE_WINDER_V1,
        }),
        "extruder2" => Ok(MachineIdentification {
            vendor: crate::machines::VENDOR_QITECH,
            machine: crate::machines::MACHINE_EXTRUDER_V1,
        }),
        "laser1" => Ok(MachineIdentification {
            vendor: crate::machines::VENDOR_QITECH,
            machine: crate::machines::MACHINE_LASER_V1,
        }),
        "buffer1" => Ok(MachineIdentification {
            vendor: crate::machines::VENDOR_QITECH,
            machine: crate::machines::MACHINE_BUFFER_V1,
        }),
        "aquapath1" => Ok(MachineIdentification {
            vendor: crate::machines::VENDOR_QITECH,
            machine: crate::machines::MACHINE_AQUAPATH_V1,
        }),
        "mock1" => Ok(MachineIdentification {
            vendor: crate::machines::VENDOR_QITECH,
            machine: crate::machines::MACHINE_MOCK,
        }),
        other => parse_vendor_machine_identifier(other),
    }
}

fn parse_vendor_machine_identifier(
    identifier: &str,
) -> Result<MachineIdentification, anyhow::Error> {
    for separator in ['-', ':', '.', '_'] {
        if let Some((vendor_raw, machine_raw)) = identifier.split_once(separator) {
            return Ok(MachineIdentification {
                vendor: parse_u16(vendor_raw)
                    .with_context(|| format!("Invalid vendor '{}'", vendor_raw))?,
                machine: parse_u16(machine_raw)
                    .with_context(|| format!("Invalid machine '{}'", machine_raw))?,
            });
        }
    }

    Err(anyhow!(
        "Unable to parse machine identifier '{}'. Expected a known slug or 'vendor-machine' format.",
        identifier
    ))
}

fn parse_u16(value: &str) -> Result<u16, anyhow::Error> {
    if let Some(stripped) = value.strip_prefix("0x") {
        u16::from_str_radix(stripped, 16)
            .with_context(|| format!("Failed to parse hex '{}'", value))
    } else if let Some(stripped) = value.strip_prefix("0X") {
        u16::from_str_radix(stripped, 16)
            .with_context(|| format!("Failed to parse hex '{}'", value))
    } else {
        value
            .parse::<u16>()
            .with_context(|| format!("Failed to parse '{}'", value))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_identifier_known_slug() {
        let (id, serial) = parse_identifier("winder2").unwrap();
        assert_eq!(id.vendor, crate::machines::VENDOR_QITECH);
        assert_eq!(id.machine, crate::machines::MACHINE_WINDER_V1);
        assert!(serial.is_none());
    }

    #[test]
    fn parse_identifier_slug_with_serial_suffix() {
        let (id, serial) = parse_identifier("winder2-7").unwrap();
        assert_eq!(id.vendor, crate::machines::VENDOR_QITECH);
        assert_eq!(id.machine, crate::machines::MACHINE_WINDER_V1);
        assert_eq!(serial, Some(7));
    }

    #[test]
    fn parse_identifier_vendor_machine() {
        let (id, serial) = parse_identifier("1-2").unwrap();
        assert_eq!(id.vendor, 1);
        assert_eq!(id.machine, 2);
        assert!(serial.is_none());
    }

    #[test]
    fn parse_identifier_vendor_machine_serial() {
        let (id, serial) = parse_identifier("1-2-42").unwrap();
        assert_eq!(id.vendor, 1);
        assert_eq!(id.machine, 2);
        assert_eq!(serial, Some(42));
    }

    #[test]
    fn parses_known_slugs() {
        let id = resolve_machine_identifier("winder2").unwrap();
        assert_eq!(id.vendor, crate::machines::VENDOR_QITECH);
        assert_eq!(id.machine, crate::machines::MACHINE_WINDER_V1);
    }

    #[test]
    fn parses_vendor_machine_with_dash() {
        let id = resolve_machine_identifier("1-2").unwrap();
        assert_eq!(id.vendor, 1);
        assert_eq!(id.machine, 2);
    }

    #[test]
    fn parses_vendor_machine_hex() {
        let id = resolve_machine_identifier("0x01:0x02").unwrap();
        assert_eq!(id.vendor, 0x01);
        assert_eq!(id.machine, 0x02);
    }

    #[test]
    fn rejects_unknown_identifier() {
        let err = parse_identifier("unknown").unwrap_err();
        assert!(
            err.to_string()
                .contains("Unable to parse machine identifier")
        );
    }

    #[test]
    fn converts_generic_event() {
        let event = GenericEvent {
            name: "TestEvent".to_string(),
            data: Box::new(serde_json::json!({"value": 42})),
            ts: 123,
        };

        let payload = convert_generic_event(&event).unwrap();
        assert_eq!(payload.name, "TestEvent");
        assert_eq!(payload.ts, 123);
        assert_eq!(payload.data["value"], 42);
    }
}
