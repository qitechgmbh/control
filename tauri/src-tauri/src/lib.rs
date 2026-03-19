mod commands;
mod state;

use state::AppState;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            // Focus the main window when a second instance is launched
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.set_focus();
                if window.is_minimized().unwrap_or(false) {
                    let _ = window.unminimize();
                }
            }
        }))
        .setup(|app| {
            let qitech_os = std::env::var("QITECH_OS").unwrap_or_default() == "true";

            if qitech_os {
                if let Some(window) = app.get_webview_window("main") {
                    window.set_fullscreen(true)?;
                }
            }

            Ok(())
        })
        .manage(AppState::default())
        .invoke_handler(tauri::generate_handler![
            commands::theme::theme_current,
            commands::theme::theme_toggle,
            commands::theme::theme_dark,
            commands::theme::theme_light,
            commands::theme::theme_system,
            commands::window::window_minimize,
            commands::window::window_maximize,
            commands::window::window_fullscreen,
            commands::window::window_close,
            commands::environment::environment_get_info,
            commands::troubleshoot::troubleshoot_reboot_hmi,
            commands::troubleshoot::troubleshoot_restart_backend,
            commands::troubleshoot::troubleshoot_export_logs,
            commands::nixos::nixos_is_available,
            commands::nixos::nixos_list_generations,
            commands::nixos::nixos_set_generation,
            commands::nixos::nixos_delete_generation,
            commands::nixos::nixos_delete_all_old_generations,
            commands::update::update_execute,
            commands::update::update_cancel,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
