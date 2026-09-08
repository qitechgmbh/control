//! Persistence for "which machine sits on which USB serial port".
//!
//! The framework only *scans* for serial ports ([`qitech_framework::runtime::modbus_rtu`]); which
//! machine is reachable on which port is control's policy, so it is stored and reloaded here and
//! handed to the runtime as `modbus_rtu_device` entries at startup (see `main.rs`).

use std::fs;

use qitech_framework::MachineInstanceIdentification;
use serde::Deserialize;
use serde::Serialize;

/// A user-configured binding of a USB serial port to a machine instance, persisted to disk so it
/// survives restarts. Keyed by `port` (the `/dev/serial/by-path` basename), which stays stable
/// across replug unlike `/dev/ttyUSBn`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ModbusRtuAssignment {
    pub port: String,
    pub machine: MachineInstanceIdentification,
    pub slave_id: u8,
}

/// A missing or unreadable file means "nothing assigned yet" - never an error, so a fresh install
/// and a corrupted file both leave the Setup page usable.
pub fn read() -> Vec<ModbusRtuAssignment> {
    let path = config_path();

    let Ok(json) = fs::read_to_string(&path) else {
        return Vec::new();
    };

    serde_json::from_str(&json).unwrap_or_else(|e| {
        tracing::error!("failed to parse modbus rtu assignments at {path}: {e}");
        Vec::new()
    })
}

/// Assign a port, replacing any assignment already stored under the same port.
pub fn write(assignment: ModbusRtuAssignment) -> std::io::Result<()> {
    let mut assignments = read();

    if let Some(existing) = assignments.iter_mut().find(|a| a.port == assignment.port) {
        *existing = assignment;
    } else {
        assignments.push(assignment);
    }

    save(&assignments)
}

pub fn remove(port: &str) -> std::io::Result<()> {
    let mut assignments = read();
    assignments.retain(|a| a.port != port);
    save(&assignments)
}

fn save(assignments: &[ModbusRtuAssignment]) -> std::io::Result<()> {
    let path = config_path();
    let tmp_path = format!("{path}.tmp");

    let json = serde_json::to_string_pretty(assignments)
        .expect("ModbusRtuAssignment is always serializable");

    // --- write-then-rename, so a crash mid-write cannot leave a truncated file behind ---
    fs::write(&tmp_path, json)?;
    fs::rename(&tmp_path, &path)?;

    Ok(())
}

/// `STATE_DIRECTORY` is what systemd sets from `StateDirectory = "qitech"` in the NixOS unit, so
/// this is `/var/lib/qitech/modbus_assignments.json` in production. The rest are development
/// fallbacks.
fn config_path() -> String {
    let dir = std::env::var("STATE_DIRECTORY")
        .or(std::env::var("XDG_STATE_HOME"))
        .or(std::env::var("HOME"))
        .unwrap_or(".".to_string());

    dir + "/modbus_assignments.json"
}
