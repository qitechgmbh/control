use crate::state::AppState;
use tauri::State;
use tokio::process::Command;

#[tauri::command]
pub async fn theme_current(state: State<'_, AppState>) -> Result<String, String> {
    let theme = state.theme.lock().await;
    Ok(theme.clone())
}

#[tauri::command]
pub async fn theme_toggle(state: State<'_, AppState>) -> Result<bool, String> {
    let mut theme = state.theme.lock().await;
    if *theme == "dark" {
        *theme = "light".to_string();
        Ok(false)
    } else {
        *theme = "dark".to_string();
        Ok(true)
    }
}

#[tauri::command]
pub async fn theme_dark(state: State<'_, AppState>) -> Result<(), String> {
    let mut theme = state.theme.lock().await;
    *theme = "dark".to_string();
    Ok(())
}

#[tauri::command]
pub async fn theme_light(state: State<'_, AppState>) -> Result<(), String> {
    let mut theme = state.theme.lock().await;
    *theme = "light".to_string();
    Ok(())
}

#[tauri::command]
pub async fn theme_system(state: State<'_, AppState>) -> Result<bool, String> {
    let mut theme = state.theme.lock().await;
    *theme = "system".to_string();
    // Return whether the system prefers dark colors
    // On Linux, check gsettings; default to false
    let prefers_dark = Command::new("gsettings")
        .args(["get", "org.gnome.desktop.interface", "color-scheme"])
        .output()
        .await
        .map(|output| {
            String::from_utf8_lossy(&output.stdout)
                .trim()
                .contains("dark")
        })
        .unwrap_or(false);
    Ok(prefers_dark)
}
