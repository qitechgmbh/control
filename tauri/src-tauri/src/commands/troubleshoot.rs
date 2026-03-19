use serde::{Deserialize, Serialize};
use tauri::AppHandle;
use tauri_plugin_dialog::DialogExt;
use tokio::process::Command;

#[derive(Debug, Serialize, Deserialize)]
pub struct TroubleshootResult {
    success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

#[tauri::command]
pub async fn troubleshoot_reboot_hmi() -> Result<TroubleshootResult, String> {
    match Command::new("sudo").arg("reboot").spawn() {
        Ok(_) => Ok(TroubleshootResult {
            success: true,
            error: None,
        }),
        Err(e) => Ok(TroubleshootResult {
            success: false,
            error: Some(e.to_string()),
        }),
    }
}

#[tauri::command]
pub async fn troubleshoot_restart_backend() -> Result<TroubleshootResult, String> {
    match Command::new("sudo")
        .args(["systemctl", "restart", "qitech-control-server"])
        .output()
        .await
    {
        Ok(output) if output.status.success() => Ok(TroubleshootResult {
            success: true,
            error: None,
        }),
        Ok(output) => Ok(TroubleshootResult {
            success: false,
            error: Some(String::from_utf8_lossy(&output.stderr).to_string()),
        }),
        Err(e) => Ok(TroubleshootResult {
            success: false,
            error: Some(e.to_string()),
        }),
    }
}

#[tauri::command]
pub async fn troubleshoot_export_logs(app: AppHandle) -> Result<TroubleshootResult, String> {
    let now = chrono::Local::now();
    let file_name = format!("journal_{}.log", now.format("%Y-%m-%d_%H-%M-%S"));

    // Use Tauri's dialog plugin for file save dialog
    let (tx, rx) = tokio::sync::oneshot::channel();
    app.dialog()
        .file()
        .set_title("Export System Logs")
        .set_file_name(&file_name)
        .add_filter("Log Files", &["log"])
        .save_file(|path| {
            let _ = tx.send(path);
        });
    let file_path = rx.await.unwrap_or(None);

    let Some(file_path) = file_path else {
        return Ok(TroubleshootResult {
            success: false,
            error: Some("Export cancelled by user".to_string()),
        });
    };

    let Some(path) = file_path.as_path() else {
        return Ok(TroubleshootResult {
            success: false,
            error: Some("Invalid file path selected".to_string()),
        });
    };

    // Run journalctl and save to file
    match Command::new("journalctl").args(["-xb"]).output().await {
        Ok(output) if output.status.success() => match std::fs::write(path, &output.stdout) {
            Ok(()) => Ok(TroubleshootResult {
                success: true,
                error: None,
            }),
            Err(e) => Ok(TroubleshootResult {
                success: false,
                error: Some(e.to_string()),
            }),
        },
        Ok(output) => Ok(TroubleshootResult {
            success: false,
            error: Some(String::from_utf8_lossy(&output.stderr).to_string()),
        }),
        Err(e) => Ok(TroubleshootResult {
            success: false,
            error: Some(e.to_string()),
        }),
    }
}
