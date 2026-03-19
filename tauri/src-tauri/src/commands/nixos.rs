use serde::{Deserialize, Serialize};
use tokio::process::Command;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct NixOSGeneration {
    id: String,
    name: String,
    version: String,
    current: bool,
    date: String,
    path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    kernel_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
}

#[tauri::command]
pub async fn nixos_is_available() -> Result<bool, String> {
    Ok(Command::new("nix")
        .arg("--version")
        .output()
        .await
        .map(|output| output.status.success())
        .unwrap_or(false))
}

#[tauri::command]
pub async fn nixos_list_generations() -> Result<Vec<NixOSGeneration>, String> {
    let output = Command::new("sudo")
        .args(["nixos-rebuild", "list-generations"])
        .output()
        .await
        .map_err(|e| e.to_string())?;

    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).to_string());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    parse_nixos_generations(&stdout)
}

#[tauri::command]
pub async fn nixos_set_generation(generation_id: String) -> Result<(), String> {
    validate_generation_id(&generation_id)?;

    let script = format!(
        "nix-env --switch-generation {} -p /nix/var/nix/profiles/system && \
         /nix/var/nix/profiles/system/bin/switch-to-configuration boot && \
         reboot",
        generation_id
    );

    let output = Command::new("sudo")
        .args(["sh", "-c", &script])
        .output()
        .await
        .map_err(|e| e.to_string())?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        return Err(format!(
            "{}{}",
            stderr,
            if stderr.is_empty() { &stdout } else { "" }
        ));
    }

    Ok(())
}

#[tauri::command]
pub async fn nixos_delete_generation(generation_id: String) -> Result<(), String> {
    validate_generation_id(&generation_id)?;

    let script = format!(
        "nix-env --delete-generations {} -p /nix/var/nix/profiles/system && \
         nix store gc && \
         /nix/var/nix/profiles/system/bin/switch-to-configuration boot",
        generation_id
    );

    let output = Command::new("sudo")
        .args(["sh", "-c", &script])
        .output()
        .await
        .map_err(|e| e.to_string())?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        return Err(format!(
            "{}{}",
            stderr,
            if stderr.is_empty() { &stdout } else { "" }
        ));
    }

    Ok(())
}

#[tauri::command]
pub async fn nixos_delete_all_old_generations() -> Result<(), String> {
    let output = Command::new("sudo")
        .args(["sh", "-c", "nix-collect-garbage --delete-old"])
        .output()
        .await
        .map_err(|e| e.to_string())?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        return Err(format!(
            "{}{}",
            stderr,
            if stderr.is_empty() { &stdout } else { "" }
        ));
    }

    Ok(())
}

fn validate_generation_id(generation_id: &str) -> Result<(), String> {
    if generation_id.is_empty() || !generation_id.chars().all(|c| c.is_ascii_digit()) {
        return Err(format!(
            "Invalid generation ID '{}': must be a numeric value",
            generation_id
        ));
    }
    Ok(())
}

fn parse_nixos_generations(output: &str) -> Result<Vec<NixOSGeneration>, String> {
    let mut generations = Vec::new();

    for (i, line) in output.lines().enumerate() {
        // Skip the first line (header) and empty lines
        if i == 0 || line.trim().is_empty() {
            continue;
        }

        // Skip lines that look like headers
        if line.contains("Generation")
            && (line.contains("Build date")
                || line.contains("NixOS version")
                || line.contains("Configuration"))
        {
            continue;
        }

        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 4 {
            continue;
        }

        let id = parts[0];

        // Skip if first part is not a number
        if !id.chars().all(|c| c.is_ascii_digit()) {
            continue;
        }

        let mut current_index = 1;
        let is_current = if parts[1] == "current" {
            current_index = 2;
            true
        } else {
            false
        };

        // Extract date and time
        let date = parts[current_index];
        let time = parts[current_index + 1];
        let date_time = format!("{date} {time}");

        // The next part should be the NixOS version/name
        let nixos_version = parts.get(current_index + 2).unwrap_or(&"").to_string();
        let nixos_version = if nixos_version.is_empty() {
            format!("Generation {id}")
        } else {
            nixos_version
        };

        // The last part (if it looks like a kernel version) is the kernel
        let last_part = *parts.last().unwrap_or(&"");
        let kernel_version = if !last_part.is_empty()
            && last_part.chars().next().unwrap_or('a').is_ascii_digit()
            && last_part.contains('.')
            && last_part != date
            && last_part != time
        {
            Some(last_part.to_string())
        } else {
            None
        };

        generations.push(NixOSGeneration {
            id: id.to_string(),
            name: nixos_version.clone(),
            version: nixos_version,
            current: is_current,
            date: date_time,
            path: format!("/nix/var/nix/profiles/system-{id}-link"),
            kernel_version,
            description: None,
        });
    }

    // Sort by ID descending (newest first)
    generations.sort_by(|a, b| {
        b.id.parse::<u32>()
            .unwrap_or(0)
            .cmp(&a.id.parse::<u32>().unwrap_or(0))
    });

    Ok(generations)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_nixos_generations() {
        let output = r#"Generation Build date           NixOS version                                                       Configuration Revision  Specialisation Kernel
 62 current 2025-06-10 08:51:35  fix.33_c744e1481fdc0bf25821bd0ee0ae8278f155                                                           6.14.8
 61         2025-06-09 14:22:11  fix.32_a123456789                                                                                      6.14.7
"#;

        let generations = parse_nixos_generations(output).unwrap();
        assert_eq!(generations.len(), 2);
        assert_eq!(generations[0].id, "62");
        assert!(generations[0].current);
        assert_eq!(generations[0].kernel_version, Some("6.14.8".to_string()));
        assert_eq!(generations[1].id, "61");
        assert!(!generations[1].current);
    }
}
