use std::fs;

use qitech_lib::ethercat_hal::machine_ident_read::MachineDeviceInfo;
use serde_json::{Value, json};
use anyhow::{Result, Context};


fn get_machine_device_info_path() -> String {
    let dir =
        std::env::var("STATE_DIRECTORY")
        .or(std::env::var("XDG_DATA_HOME"))
        .or(std::env::var("HOME"))
        .unwrap_or(".".to_string());

    dir + "/qitech.json"
}

pub fn write_machine_device_info(infos: &[MachineDeviceInfo]) -> Result<()> {
    let json_vec = infos
        .iter()
        .map(|info| json!({
            "role": info.role,
            "machine_id": info.machine_id,
            "machine_vendor": info.machine_vendor,
            "machine_serial": info.machine_serial,
            "device_address": info.device_address,
        }))
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

    let infos = value.as_array().context("Root value is not an array")?.iter().map(|value| -> Result<MachineDeviceInfo> {
        Ok(MachineDeviceInfo {
            role: value["role"].as_u64().unwrap_or(0) as u16,
            machine_id: value["machine_id"].as_u64().unwrap_or(0) as u16,
            machine_vendor: value["machine_vendor"].as_u64().unwrap_or(0) as u16,
            machine_serial: value["machine_serial"].as_u64().unwrap_or(0) as u16,
            device_address: value["device_address"].as_u64().context("No device address given")? as u16,
        })
    })
    .collect::<Result<Vec<_>>>()?;

    Ok(infos)
}
