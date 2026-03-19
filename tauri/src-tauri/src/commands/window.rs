use tauri::{AppHandle, Manager};

#[tauri::command]
pub async fn window_minimize(app: AppHandle) -> Result<(), String> {
    app.get_webview_window("main")
        .ok_or("Window not found")?
        .minimize()
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn window_maximize(app: AppHandle) -> Result<(), String> {
    let window = app.get_webview_window("main").ok_or("Window not found")?;
    if window.is_maximized().unwrap_or(false) {
        window.unmaximize()
    } else {
        window.maximize()
    }
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn window_fullscreen(app: AppHandle, value: bool) -> Result<(), String> {
    app.get_webview_window("main")
        .ok_or("Window not found")?
        .set_fullscreen(value)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn window_close(app: AppHandle) -> Result<(), String> {
    app.get_webview_window("main")
        .ok_or("Window not found")?
        .close()
        .map_err(|e| e.to_string())
}
