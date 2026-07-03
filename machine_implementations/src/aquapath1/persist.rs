use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Default)]
pub struct AquaPathV1Settings {
    #[serde(default)]
    pub swap_sides: bool,
}

fn settings_dir() -> PathBuf {
    let dir = std::env::var("STATE_DIRECTORY")
        .or_else(|_| std::env::var("XDG_DATA_HOME"))
        .or_else(|_| std::env::var("HOME"))
        .unwrap_or_else(|_| ".".to_string());

    PathBuf::from(dir).join("machine_settings")
}

fn settings_path(serial: u32) -> PathBuf {
    settings_dir().join(format!("aquapath1_{}.json", serial))
}

pub fn load_settings(serial: u32) -> AquaPathV1Settings {
    let path = settings_path(serial);

    if !path.exists() {
        return AquaPathV1Settings::default();
    }

    match fs::read_to_string(&path) {
        Ok(json) => serde_json::from_str(&json).unwrap_or_default(),
        Err(_) => AquaPathV1Settings::default(),
    }
}

pub fn save_settings(serial: u32, settings: &AquaPathV1Settings) -> std::io::Result<()> {
    let path = settings_path(serial);

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let json = serde_json::to_string_pretty(settings)?;
    fs::write(path, json)
}
