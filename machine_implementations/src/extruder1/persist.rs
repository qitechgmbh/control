//! On-disk store for identified coupling models and the gains synthesized from them.
//!
//! A campaign owns the machine for hours. Losing that to a restart is not acceptable, so the
//! result is written as soon as it exists and reloaded when the machine is constructed.
//!
//! Kept in its own file rather than added to `qitech.json`: the reader for that one requires the
//! JSON root to be an array of device-role records, so extending it in place would break the
//! device mapping — the one piece of state the server genuinely cannot start without.

use super::mimo::ZONE_ORDER;
use control_core::controllers::mimo::{MimoGains, MimoModel};
use control_core::persistence::state_path;
use qitech_lib::machines::MachineIdentificationUnique;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

const FILE_NAME: &str = "qitech_mimo.json";

/// Bumped only when a change cannot be expressed as an added optional field. A record written by a
/// different version is discarded rather than guessed at — re-running a campaign is a known cost,
/// whereas misreading a gain matrix onto live heaters is not.
const SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MimoRecord {
    pub version: u32,
    /// Physical zone order the matrices are indexed by, stored so a record can be recognised as
    /// stale if the ordering convention ever changes.
    pub zone_order: Vec<String>,
    pub model: MimoModel,
    pub gains: Option<MimoGains>,
    /// Name of the synthesis backend that produced `gains`.
    pub synthesis: String,
}

fn key(id: &MachineIdentificationUnique) -> String {
    format!(
        "{}:{}:{}",
        id.machine_ident.vendor, id.machine_ident.machine, id.serial
    )
}

fn expected_zone_order() -> Vec<String> {
    ZONE_ORDER.iter().map(|z| z.as_str().to_owned()).collect()
}

fn read_all() -> BTreeMap<String, MimoRecord> {
    let path = state_path(FILE_NAME);
    let Ok(text) = std::fs::read_to_string(&path) else {
        return BTreeMap::new();
    };
    match serde_json::from_str(&text) {
        Ok(map) => map,
        Err(e) => {
            // Start clean rather than propagate. The alternative is refusing to boot over a file
            // whose only content is a tuning convenience.
            tracing::warn!("ignoring unreadable {}: {e}", path.display());
            BTreeMap::new()
        }
    }
}

/// Store this machine's model and gains, replacing any previous record.
pub fn write(
    id: &MachineIdentificationUnique,
    model: &MimoModel,
    gains: Option<&MimoGains>,
    synthesis: &str,
) {
    let mut all = read_all();
    all.insert(
        key(id),
        MimoRecord {
            version: SCHEMA_VERSION,
            zone_order: expected_zone_order(),
            model: model.clone(),
            gains: gains.copied(),
            synthesis: synthesis.to_owned(),
        },
    );

    let path = state_path(FILE_NAME);
    match serde_json::to_string_pretty(&all).map(|json| std::fs::write(&path, json)) {
        Ok(Ok(())) => {}
        Ok(Err(e)) => tracing::error!("could not write {}: {e}", path.display()),
        Err(e) => tracing::error!("could not serialise MIMO records: {e}"),
    }
}

/// Load this machine's record, if one was written by a compatible version.
pub fn read(id: &MachineIdentificationUnique) -> Option<MimoRecord> {
    let record = read_all().remove(&key(id))?;

    if record.version != SCHEMA_VERSION {
        tracing::warn!(
            "discarding MIMO record written by schema version {} (expected {SCHEMA_VERSION})",
            record.version
        );
        return None;
    }
    if record.zone_order != expected_zone_order() {
        tracing::warn!(
            "discarding MIMO record indexed by {:?}, which is not the current zone order {:?}",
            record.zone_order,
            expected_zone_order()
        );
        return None;
    }
    Some(record)
}
