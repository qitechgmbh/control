use std::fs;

use anyhow::{Context, Result};
use qitech_lib::ethercat_hal::machine_ident_read::MachineDeviceInfo;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

fn get_state_dir() -> String {
    std::env::var("STATE_DIRECTORY")
        .or(std::env::var("XDG_DATA_HOME"))
        .or(std::env::var("HOME"))
        .unwrap_or(".".to_string())
}

fn get_machine_device_info_path() -> String {
    get_state_dir() + "/qitech.json"
}

fn get_settings_path() -> String {
    get_state_dir() + "/qitech-settings.json"
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Settings {
    #[serde(default)]
    pub ethercat_enabled: Option<bool>,
}

pub fn read_settings() -> Settings {
    let path = get_settings_path();
    let Ok(json) = fs::read_to_string(&path) else {
        return Settings::default();
    };
    match serde_json::from_str(&json) {
        Ok(settings) => settings,
        Err(e) => {
            tracing::warn!("Ignoring unreadable settings file {}: {}", path, e);
            Settings::default()
        }
    }
}

pub fn write_settings(settings: &Settings) -> Result<()> {
    let json = serde_json::to_string_pretty(settings)?;
    fs::write(get_settings_path(), json).context("Failed to write settings file")?;
    Ok(())
}

pub fn resolve_ethercat_enabled() -> bool {
    if let Some(enabled) = read_settings().ethercat_enabled {
        return enabled;
    }
    match std::env::var("ETHERCAT_ENABLED").as_deref() {
        Ok("false" | "0" | "no") => false,
        Ok("true" | "1" | "yes") => true,
        _ => true,
    }
}

pub fn set_ethercat_enabled(enabled: bool) -> Result<()> {
    let mut settings = read_settings();
    settings.ethercat_enabled = Some(enabled);
    write_settings(&settings)
}

pub fn write_machine_device_info(infos: &[MachineDeviceInfo]) -> Result<()> {
    let json_vec = infos
        .iter()
        .map(|info| {
            json!({
                "role": info.role,
                "machine_id": info.machine_id,
                "machine_vendor": info.machine_vendor,
                "machine_serial": info.machine_serial,
                "device_address": info.device_address,
            })
        })
        .collect::<Vec<_>>();

    let json = serde_json::to_string(&json_vec)?;

    let path = get_machine_device_info_path();
    fs::write(path, json)?;

    Ok(())
}

pub fn read_machine_device_info() -> Result<Vec<MachineDeviceInfo>> {
    let path = get_machine_device_info_path();

    if !fs::exists(&path)? {
        return Ok(vec![]);
    }

    let json = fs::read_to_string(path)?;
    let value: Value = serde_json::from_str(&json)?;

    let infos = value
        .as_array()
        .context("Root value is not an array")?
        .iter()
        .map(|value| -> Result<MachineDeviceInfo> {
            Ok(MachineDeviceInfo {
                role: value["role"].as_u64().unwrap_or(0) as u16,
                machine_id: value["machine_id"].as_u64().unwrap_or(0) as u16,
                machine_vendor: value["machine_vendor"].as_u64().unwrap_or(0) as u16,
                machine_serial: value["machine_serial"].as_u64().unwrap_or(0) as u16,
                device_address: value["device_address"]
                    .as_u64()
                    .context("No device address given")? as u16,
            })
        })
        .collect::<Result<Vec<_>>>()?;

    Ok(infos)
}
