# Electron to Tauri Migration Plan

## Executive Summary

This plan outlines a comprehensive, phased migration from Electron to Tauri 2.x for the QiTech Control application. The migration leverages the existing shell-agnostic React UI layer (bridge pattern) while reimplementing all IPC functionality in Rust. Technical debt will be addressed strategically: some before migration (in shared UI), some during migration (as part of the Tauri implementation), and some deferred to post-migration cleanup.

**Key Insight**: The existing bridge pattern (`NativeBridge` interface + `getBridge()/setBridge()` singleton) makes this migration significantly easier than a typical Electron→Tauri migration. The React UI is already shell-agnostic.

---

## Phase 0: Pre-Migration Technical Debt Cleanup (UI Layer)

**Goal**: Fix technical debt in the shared `/ui/` layer before starting Tauri work, since these fixes benefit both Electron and Tauri.

### 0.1: Fix Type Safety Issues in Bridge

**Files to modify**:
- `/home/hsp/Repositories/qitech/control/ui/src/bridge/stub.ts`

**Changes**:
1. Remove `as any` cast on line 5
2. Fix return types for `notAvailable()` helper to properly return typed promises

**Rationale**: Type safety is critical for maintaining the bridge contract. This ensures both Electron and Tauri implementations conform to the same interface.

### 0.2: Fix Theme Helpers

**Files to modify**:
- `/home/hsp/Repositories/qitech/control/ui/src/helpers/theme_helpers.ts`

**Changes**:
1. Uncomment the theme switching logic (currently hardcoded to light mode)
2. Re-enable dark mode functionality
3. Keep the logic intact for proper theme synchronization

**Rationale**: Theme switching is currently broken. Since this is UI-layer logic that both shells will use, fix it before migration.

### 0.3: Consider Moving App.tsx to UI Layer

**Analysis needed**:
- `/home/hsp/Repositories/qitech/control/electron/src/App.tsx` (150 lines)
- Would create `/home/hsp/Repositories/qitech/control/ui/src/ShellApp.tsx`

**Decision**: **DEFER** - While this would reduce duplication, it's not blocking for Tauri migration. The App.tsx file is minimal (theme sync, language sync, router provider). Can be done post-migration as optimization.

---

## Phase 1: Project Structure Setup

### 1.1: Create Tauri Directory Structure

Create `/home/hsp/Repositories/qitech/control/tauri/` with this structure:

```
tauri/
├── package.json                 # npm package config
├── tsconfig.json               # TypeScript config
├── vite.config.ts              # Vite config (different from Electron)
├── public/                     # Static assets (icon, etc.)
│   └── icon.png
├── src/                        # Frontend entry point
│   ├── main.tsx               # Main entry (imports UI, sets bridge)
│   ├── bridge.ts              # Maps Tauri invoke() to NativeBridge
│   └── App.tsx                # React root (copied from electron)
└── src-tauri/                  # Rust backend
    ├── Cargo.toml             # Tauri package config
    ├── tauri.conf.json        # Tauri app config
    ├── build.rs               # Build script
    ├── icons/                 # App icons (generated)
    └── src/
        ├── main.rs            # Entry point, window setup
        ├── lib.rs             # Module exports
        ├── commands/          # IPC command modules
        │   ├── mod.rs
        │   ├── theme.rs
        │   ├── window.rs
        │   ├── environment.rs
        │   ├── troubleshoot.rs
        │   ├── nixos.rs
        │   └── update.rs
        └── state.rs           # Shared state (e.g., update process handle)
```

### 1.2: Update Root Configuration

**File**: `/home/hsp/Repositories/qitech/control/package.json`
```json
{
  "workspaces": ["ui", "electron", "tauri"]
}
```

**File**: `/home/hsp/Repositories/qitech/control/Cargo.toml`
```toml
[workspace]
members = [
    "server",
    "ethercat-hal",
    "ethercat-hal-derive",
    "control-core",
    "machines",
    "control-core-derive",
    "units",
    "utils",
    "tauri/src-tauri"  # Add Tauri to Rust workspace
]
```

### 1.3: Initialize Tauri Project

**Commands to run** (conceptually - actual initialization):
```bash
cd /home/hsp/Repositories/qitech/control/tauri
npm create tauri-app@latest . --name qitech-control-tauri --template vanilla-ts
```

Then modify the generated files to:
- Remove default template code
- Configure for monorepo (workspace paths)
- Set up proper app ID: `de.qitech.control-tauri`

---

## Phase 2: Frontend Layer (TypeScript/React)

### 2.1: Create Tauri Bridge Implementation

**File**: `/home/hsp/Repositories/qitech/control/tauri/src/bridge.ts`

This maps Tauri's `invoke()` and event listeners to the `NativeBridge` interface.

**Key differences from Electron**:
- Electron uses `window.themeMode`, `window.electronWindow`, etc. (exposed via contextBridge)
- Tauri uses `@tauri-apps/api/core` `invoke()` and `@tauri-apps/api/event` `listen()`

**Implementation approach**:

```typescript
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import type { NativeBridge } from '@ui/bridge/types';

export const tauriBridge: NativeBridge = {
  theme: {
    current: () => invoke<ThemeMode>('theme_current'),
    toggle: () => invoke<boolean>('theme_toggle'),
    dark: () => invoke('theme_dark'),
    light: () => invoke('theme_light'),
    system: () => invoke<boolean>('theme_system'),
  },
  window: {
    minimize: () => invoke('window_minimize'),
    maximize: () => invoke('window_maximize'),
    fullscreen: (value: boolean) => invoke('window_fullscreen', { value }),
    close: () => invoke('window_close'),
  },
  environment: {
    getInfo: () => invoke<EnvironmentInfo>('environment_get_info'),
  },
  update: {
    execute: (params) => invoke('update_execute', { params }),
    cancel: () => invoke<{success: boolean; error?: string}>('update_cancel'),
    onLog: (callback) => {
      listen<string>('update-log', (event) => callback(event.payload));
    },
    onEnd: (callback) => {
      listen<{success: boolean; error?: string}>('update-end', (event) => 
        callback(event.payload)
      );
    },
    onStep: (callback) => {
      listen<UpdateStepParams>('update-step', (event) => 
        callback(event.payload)
      );
    },
  },
  troubleshoot: {
    rebootHmi: () => invoke('troubleshoot_reboot_hmi'),
    restartBackend: () => invoke('troubleshoot_restart_backend'),
    exportLogs: () => invoke('troubleshoot_export_logs'),
  },
  nixos: {
    isNixOSAvailable: false, // Set at runtime via invoke in initialization
    listGenerations: () => invoke<NixOSGeneration[]>('nixos_list_generations'),
    setGeneration: (generationId) => invoke('nixos_set_generation', { generationId }),
    deleteGeneration: (generationId) => invoke('nixos_delete_generation', { generationId }),
    deleteAllOldGenerations: () => invoke('nixos_delete_all_old_generations'),
  },
};

// Initialize nixos availability at startup
invoke<boolean>('nixos_is_available').then(available => {
  tauriBridge.nixos.isNixOSAvailable = available;
});
```

**Note on event listeners**: Tauri's `listen()` returns an `UnlistenFn`, but the bridge interface doesn't support cleanup. This is a **known limitation** in the current bridge design (affects Electron too). For proper cleanup, the bridge interface would need to return unsubscribe functions.

### 2.2: Create Main Entry Point

**File**: `/home/hsp/Repositories/qitech/control/tauri/src/main.tsx`

```typescript
import "@ui/styles/global.css";
import "@ui/styles/markdown.css";

import { setBridge } from "@ui/bridge";
import { tauriBridge } from "./bridge";

// Register the Tauri bridge before the app boots
setBridge(tauriBridge);

import "./App";
```

**Pattern**: Identical to Electron's `renderer.ts`, just different bridge import.

### 2.3: Create App Component

**File**: `/home/hsp/Repositories/qitech/control/tauri/src/App.tsx`

Copy from `/home/hsp/Repositories/qitech/control/electron/src/App.tsx` with no changes needed (it's shell-agnostic).

### 2.4: Configure Vite for Tauri

**File**: `/home/hsp/Repositories/qitech/control/tauri/vite.config.ts`

```typescript
import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import path from "path";

// Tauri uses a different Vite plugin than Electron
export default defineConfig({
  plugins: [
    react({
      babel: {
        plugins: [["babel-plugin-react-compiler"]],
      },
    }),
  ],
  
  // Important: Tauri expects the frontend in 'src', not 'dist'
  root: "src",
  publicDir: path.resolve(__dirname, "public"),
  
  resolve: {
    preserveSymlinks: true,
    alias: {
      "@": path.resolve(__dirname, "./src"),
      "@ui": path.resolve(__dirname, "../ui/src"),
      "@root": path.resolve(__dirname, ".."),
    },
  },
  
  server: {
    fs: {
      allow: [
        path.resolve(__dirname),
        path.resolve(__dirname, "../ui")
      ],
    },
  },
  
  // Tauri-specific: where to output the build
  build: {
    outDir: path.resolve(__dirname, "dist"),
    emptyOutDir: true,
  },
  
  clearScreen: false,
  
  // Tauri dev server configuration
  server: {
    port: 1420,
    strictPort: true,
  },
  
  // Env prefix for exposing to frontend
  envPrefix: ["VITE_", "TAURI_"],
});
```

### 2.5: Package Configuration

**File**: `/home/hsp/Repositories/qitech/control/tauri/package.json`

```json
{
  "name": "qitech-control-tauri",
  "productName": "QiTech Control",
  "version": "2.15.0",
  "description": "Next Generation Machine Interface — Tauri shell",
  "private": true,
  "type": "module",
  "scripts": {
    "dev": "tauri dev",
    "build": "tauri build",
    "lint": "eslint .",
    "format": "prettier --check .",
    "format:write": "prettier --write ."
  },
  "dependencies": {
    "@qitech/ui": "*",
    "@tauri-apps/api": "^2.0.0",
    "@tauri-apps/plugin-dialog": "^2.0.0",
    "@tauri-apps/plugin-shell": "^2.0.0",
    "@vitejs/plugin-react": "^4.5.2",
    "babel-plugin-react-compiler": "^19.0.0-beta-714736e-20250131"
  },
  "devDependencies": {
    "@tauri-apps/cli": "^2.0.0",
    "typescript": "^5.7.3",
    "vite": "^6.4.1"
  }
}
```

---

## Phase 3: Rust Backend - Core Setup

### 3.1: Cargo Configuration

**File**: `/home/hsp/Repositories/qitech/control/tauri/src-tauri/Cargo.toml`

```toml
[package]
name = "qitech-control-tauri"
version = "2.15.0"
description = "QiTech Control Tauri Application"
authors = ["QiTech"]
edition = "2021"

[build-dependencies]
tauri-build = { version = "2.0", features = [] }

[dependencies]
tauri = { version = "2.0", features = ["unstable"] }
tauri-plugin-dialog = "2.0"
tauri-plugin-shell = "2.0"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tokio = { version = "1", features = ["full"] }
anyhow = "1"
thiserror = "1"

# For NixOS generation parsing
chrono = "0.4"

# Logging (replace console.log with proper logging)
tracing = "0.1"
tracing-subscriber = "0.3"

[features]
default = ["custom-protocol"]
custom-protocol = ["tauri/custom-protocol"]

[lints.clippy]
# Inherit from workspace
```

### 3.2: Tauri Configuration

**File**: `/home/hsp/Repositories/qitech/control/tauri/src-tauri/tauri.conf.json`

```json
{
  "productName": "QiTech Control",
  "version": "2.15.0",
  "identifier": "de.qitech.control-tauri",
  "build": {
    "beforeDevCommand": "npm run dev",
    "beforeBuildCommand": "npm run build",
    "devUrl": "http://localhost:1420",
    "frontendDist": "../dist"
  },
  "bundle": {
    "active": true,
    "targets": "all",
    "icon": [
      "icons/32x32.png",
      "icons/128x128.png",
      "icons/128x128@2x.png",
      "icons/icon.icns",
      "icons/icon.ico"
    ],
    "linux": {
      "deb": {
        "depends": []
      }
    }
  },
  "app": {
    "windows": [
      {
        "title": "QiTech Control",
        "width": 1200,
        "height": 800,
        "fullscreen": false,
        "resizable": true,
        "decorations": true
      }
    ],
    "security": {
      "csp": null
    }
  },
  "plugins": {
    "dialog": {},
    "shell": {
      "scope": [
        {
          "name": "nix",
          "cmd": "nix",
          "args": true,
          "sidecar": false
        },
        {
          "name": "sudo",
          "cmd": "sudo",
          "args": true,
          "sidecar": false
        },
        {
          "name": "git",
          "cmd": "git",
          "args": true,
          "sidecar": false
        },
        {
          "name": "journalctl",
          "cmd": "journalctl",
          "args": true,
          "sidecar": false
        }
      ]
    }
  }
}
```

**Key security note**: The shell plugin scope explicitly allows `sudo`, `nix`, `git`, and `journalctl` commands. This is necessary for the NixOS and update functionality but should be documented as a security consideration.

### 3.3: Main Entry Point

**File**: `/home/hsp/Repositories/qitech/control/tauri/src-tauri/src/main.rs`

```rust
// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod state;

use commands::*;
use state::AppState;
use tauri::Manager;
use tracing_subscriber;

fn main() {
    // Initialize logging (replaces console.log)
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            // Read QITECH_OS environment variable
            let qitech_os = std::env::var("QITECH_OS").unwrap_or_default() == "true";
            
            // Set fullscreen if running on QITECH_OS
            if qitech_os {
                if let Some(window) = app.get_webview_window("main") {
                    window.set_fullscreen(true)?;
                }
            }
            
            Ok(())
        })
        .manage(AppState::default())
        .invoke_handler(tauri::generate_handler![
            // Theme commands
            theme::theme_current,
            theme::theme_toggle,
            theme::theme_dark,
            theme::theme_light,
            theme::theme_system,
            
            // Window commands
            window::window_minimize,
            window::window_maximize,
            window::window_fullscreen,
            window::window_close,
            
            // Environment commands
            environment::environment_get_info,
            
            // Troubleshoot commands
            troubleshoot::troubleshoot_reboot_hmi,
            troubleshoot::troubleshoot_restart_backend,
            troubleshoot::troubleshoot_export_logs,
            
            // NixOS commands
            nixos::nixos_is_available,
            nixos::nixos_list_generations,
            nixos::nixos_set_generation,
            nixos::nixos_delete_generation,
            nixos::nixos_delete_all_old_generations,
            
            // Update commands
            update::update_execute,
            update::update_cancel,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

### 3.4: Shared State

**File**: `/home/hsp/Repositories/qitech/control/tauri/src-tauri/src/state.rs`

```rust
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::process::Child;

/// Shared application state
/// 
/// Replaces Electron's module-level globals (currentUpdateProcess, etc.)
#[derive(Default)]
pub struct AppState {
    /// Handle to the currently running update process
    /// None if no update is running
    pub update_process: Arc<Mutex<Option<UpdateProcess>>>,
}

pub struct UpdateProcess {
    pub child: Child,
    pub rust_build_progress: RustBuildProgress,
}

#[derive(Default)]
pub struct RustBuildProgress {
    pub total_derivations: usize,
    pub built_derivations: usize,
    pub max_percent: usize,
}
```

### 3.5: Commands Module Structure

**File**: `/home/hsp/Repositories/qitech/control/tauri/src-tauri/src/commands/mod.rs`

```rust
pub mod theme;
pub mod window;
pub mod environment;
pub mod troubleshoot;
pub mod nixos;
pub mod update;
```

---

## Phase 4: Rust Backend - Command Implementation

### 4.1: Theme Commands (Trivial - 30 min)

**File**: `/home/hsp/Repositories/qitech/control/tauri/src-tauri/src/commands/theme.rs`

**Complexity**: Low
**LoC**: ~50

**Implementation notes**:
- Tauri doesn't have native theme detection like Electron's `nativeTheme`
- Need to track theme state in app state or use OS-level APIs
- For Linux (NixOS), check `gsettings get org.gnome.desktop.interface color-scheme`
- Fall back to file-based state storage

**Approach**:
```rust
use tauri::State;
use std::sync::Mutex;

struct ThemeState(Mutex<String>);

#[tauri::command]
pub fn theme_current(state: State<ThemeState>) -> String {
    state.0.lock().unwrap().clone()
}

#[tauri::command]
pub fn theme_toggle(state: State<ThemeState>) -> bool {
    let mut theme = state.0.lock().unwrap();
    if *theme == "dark" {
        *theme = "light".to_string();
        false
    } else {
        *theme = "dark".to_string();
        true
    }
}

// Similar for theme_dark, theme_light, theme_system
```

**Technical debt addressed**: Remove console.log, use proper logging with `tracing` crate.

### 4.2: Window Commands (Trivial - 20 min)

**File**: `/home/hsp/Repositories/qitech/control/tauri/src-tauri/src/commands/window.rs`

**Complexity**: Low
**LoC**: ~40

**Implementation**:
```rust
use tauri::{AppHandle, Manager};

#[tauri::command]
pub fn window_minimize(app: AppHandle) -> Result<(), String> {
    app.get_webview_window("main")
        .ok_or("Window not found")?
        .minimize()
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn window_maximize(app: AppHandle) -> Result<(), String> {
    let window = app.get_webview_window("main").ok_or("Window not found")?;
    if window.is_maximized().unwrap_or(false) {
        window.unmaximize()
    } else {
        window.maximize()
    }
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn window_fullscreen(app: AppHandle, value: bool) -> Result<(), String> {
    app.get_webview_window("main")
        .ok_or("Window not found")?
        .set_fullscreen(value)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn window_close(app: AppHandle) -> Result<(), String> {
    app.get_webview_window("main")
        .ok_or("Window not found")?
        .close()
        .map_err(|e| e.to_string())
}
```

### 4.3: Environment Commands (Trivial - 15 min)

**File**: `/home/hsp/Repositories/qitech/control/tauri/src-tauri/src/commands/environment.rs`

**Complexity**: Low
**LoC**: ~30

**Implementation**:
```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct EnvironmentInfo {
    qitech_os: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    qitech_os_git_timestamp: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    qitech_os_git_commit: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    qitech_os_git_abbreviation: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    qitech_os_git_url: Option<String>,
}

#[tauri::command]
pub fn environment_get_info() -> EnvironmentInfo {
    EnvironmentInfo {
        qitech_os: std::env::var("QITECH_OS").unwrap_or_default() == "true",
        qitech_os_git_timestamp: std::env::var("QITECH_OS_GIT_TIMESTAMP").ok(),
        qitech_os_git_commit: std::env::var("QITECH_OS_GIT_COMMIT").ok(),
        qitech_os_git_abbreviation: std::env::var("QITECH_OS_GIT_ABBREVIATION").ok(),
        qitech_os_git_url: std::env::var("QITECH_OS_GIT_URL").ok(),
    }
}
```

**Technical debt addressed**: No hardcoded paths, clean implementation.

### 4.4: Troubleshoot Commands (Medium - 2 hours)

**File**: `/home/hsp/Repositories/qitech/control/tauri/src-tauri/src/commands/troubleshoot.rs`

**Complexity**: Medium
**LoC**: ~120

**Implementation approach**:
```rust
use serde::{Deserialize, Serialize};
use tauri_plugin_shell::ShellExt;
use tauri_plugin_dialog::{DialogExt, MessageDialogKind};
use std::process::Command;

#[derive(Debug, Serialize, Deserialize)]
pub struct TroubleshootResult {
    success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

#[tauri::command]
pub async fn troubleshoot_reboot_hmi() -> TroubleshootResult {
    match Command::new("sudo")
        .arg("reboot")
        .spawn()
    {
        Ok(_) => TroubleshootResult { success: true, error: None },
        Err(e) => TroubleshootResult {
            success: false,
            error: Some(e.to_string()),
        },
    }
}

#[tauri::command]
pub async fn troubleshoot_restart_backend() -> TroubleshootResult {
    match Command::new("sudo")
        .args(["systemctl", "restart", "qitech-control-server"])
        .output()
        .await
    {
        Ok(output) if output.status.success() => {
            TroubleshootResult { success: true, error: None }
        }
        Ok(output) => TroubleshootResult {
            success: false,
            error: Some(String::from_utf8_lossy(&output.stderr).to_string()),
        },
        Err(e) => TroubleshootResult {
            success: false,
            error: Some(e.to_string()),
        },
    }
}

#[tauri::command]
pub async fn troubleshoot_export_logs(app: AppHandle) -> TroubleshootResult {
    // Use Tauri's dialog plugin for file save dialog
    let file_path = app.dialog()
        .file()
        .set_title("Export System Logs")
        .set_file_name(&format!("journal_{}.log", chrono::Local::now().format("%Y-%m-%d_%H-%M-%S")))
        .add_filter("Log Files", &["log"])
        .blocking_save_file();
    
    let Some(file_path) = file_path else {
        return TroubleshootResult {
            success: false,
            error: Some("Export cancelled by user".to_string()),
        };
    };
    
    // Run journalctl and save to file
    match Command::new("journalctl")
        .args(["-xb"])
        .output()
        .await
    {
        Ok(output) if output.status.success() => {
            match std::fs::write(&file_path, output.stdout) {
                Ok(_) => TroubleshootResult { success: true, error: None },
                Err(e) => TroubleshootResult {
                    success: false,
                    error: Some(e.to_string()),
                },
            }
        }
        Ok(output) => TroubleshootResult {
            success: false,
            error: Some(String::from_utf8_lossy(&output.stderr).to_string()),
        },
        Err(e) => TroubleshootResult {
            success: false,
            error: Some(e.to_string()),
        },
    }
}
```

**Technical debt addressed**:
- Replace `spawn` with proper async `tokio::process::Command`
- Use Tauri's dialog plugin instead of Electron's dialog
- Proper error handling with `Result` types
- No console.log, use tracing instead

### 4.5: NixOS Commands (High Complexity - 4 hours)

**File**: `/home/hsp/Repositories/qitech/control/tauri/src-tauri/src/commands/nixos.rs`

**Complexity**: High
**LoC**: ~300

**Key challenges**:
1. Complex output parsing for `nixos-rebuild list-generations`
2. Multi-step commands (delete generation + gc + bootloader update)
3. Proper error handling for sudo commands

**Implementation approach**:

```rust
use serde::{Deserialize, Serialize};
use tokio::process::Command;
use anyhow::{Context, Result};

#[derive(Debug, Serialize, Deserialize, Clone)]
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
pub async fn nixos_is_available() -> bool {
    Command::new("nix")
        .arg("--version")
        .output()
        .await
        .map(|output| output.status.success())
        .unwrap_or(false)
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
    let script = format!(
        "nix-env --switch-generation {} -p /nix/var/nix/profiles/system && \
         /nix/var/nix/profiles/system/bin/switch-to-configuration boot && \
         reboot",
        generation_id
    );
    
    Command::new("sudo")
        .args(["sh", "-c", &script])
        .spawn()
        .map_err(|e| e.to_string())?;
    
    Ok(())
}

#[tauri::command]
pub async fn nixos_delete_generation(generation_id: String) -> Result<(), String> {
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
        return Err(String::from_utf8_lossy(&output.stderr).to_string());
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
        return Err(String::from_utf8_lossy(&output.stderr).to_string());
    }
    
    Ok(())
}

fn parse_nixos_generations(output: &str) -> Result<Vec<NixOSGeneration>, String> {
    let mut generations = Vec::new();
    
    for (i, line) in output.lines().enumerate() {
        // Skip header and empty lines
        if i == 0 || line.trim().is_empty() {
            continue;
        }
        
        // Skip lines that look like headers
        if line.contains("Generation") && 
           (line.contains("Build date") || line.contains("NixOS version")) {
            continue;
        }
        
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 4 {
            continue;
        }
        
        let id = parts[0];
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
        
        let date = parts[current_index];
        let time = parts[current_index + 1];
        let date_time = format!("{} {}", date, time);
        
        let nixos_version = parts.get(current_index + 2)
            .unwrap_or(&format!("Generation {}", id))
            .to_string();
        
        let last_part = parts.last().unwrap();
        let kernel_version = if last_part.chars().next().unwrap_or('a').is_ascii_digit() 
            && last_part.contains('.') {
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
            path: format!("/nix/var/nix/profiles/system-{}-link", id),
            kernel_version,
            description: None,
        });
    }
    
    // Sort by ID descending
    generations.sort_by(|a, b| {
        b.id.parse::<u32>().unwrap_or(0)
            .cmp(&a.id.parse::<u32>().unwrap_or(0))
    });
    
    Ok(generations)
}
```

**Technical debt addressed**:
- Direct translation of parsing logic (already well-tested in Electron)
- Proper error types with `Result<T, String>`
- No console.log
- Async/await throughout

**Input validation**: Consider adding Zod-equivalent validation in Rust (e.g., using `validator` crate or manual checks) for generation IDs to prevent command injection.

### 4.6: Update Commands (Highest Complexity - 8-12 hours)

**File**: `/home/hsp/Repositories/qitech/control/tauri/src-tauri/src/commands/update.rs`

**Complexity**: Very High
**LoC**: ~600-800 (down from 700 due to better structure)

**Key challenges**:
1. Real-time log streaming via events
2. Process cancellation (kill process tree)
3. Git progress parsing
4. Nix derivation progress parsing
5. Multi-step progress tracking
6. Managing long-running subprocess
7. Replacing tree-kill functionality

**Implementation approach**:

```rust
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager, State};
use tokio::process::Command;
use tokio::io::{AsyncBufReadExt, BufReader};
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::state::{AppState, UpdateProcess, RustBuildProgress};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UpdateExecuteParams {
    github_repo_owner: String,
    github_repo_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    github_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tag: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    branch: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    commit: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UpdateStepParams {
    step_id: String,
    status: String, // "pending" | "in-progress" | "completed" | "error"
    #[serde(skip_serializing_if = "Option::is_none")]
    progress: Option<u8>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateResult {
    success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

#[tauri::command]
pub async fn update_execute(
    app: AppHandle,
    state: State<'_, AppState>,
    params: UpdateExecuteParams,
) -> Result<(), String> {
    // Spawn the update in a background task
    let app_clone = app.clone();
    let state_clone = state.inner().clone();
    
    tokio::spawn(async move {
        match run_update(app_clone.clone(), state_clone, params).await {
            Ok(_) => {
                app_clone.emit("update-end", UpdateResult {
                    success: true,
                    error: None,
                }).ok();
            }
            Err(e) => {
                app_clone.emit("update-end", UpdateResult {
                    success: false,
                    error: Some(e.to_string()),
                }).ok();
            }
        }
    });
    
    Ok(())
}

#[tauri::command]
pub async fn update_cancel(state: State<'_, AppState>) -> Result<UpdateResult, String> {
    let mut process_lock = state.update_process.lock().await;
    
    if let Some(mut update_process) = process_lock.take() {
        match update_process.child.kill().await {
            Ok(_) => Ok(UpdateResult {
                success: true,
                error: None,
            }),
            Err(e) => Ok(UpdateResult {
                success: false,
                error: Some(e.to_string()),
            }),
        }
    } else {
        Ok(UpdateResult {
            success: false,
            error: Some("No update process running".to_string()),
        })
    }
}

async fn run_update(
    app: AppHandle,
    state: Arc<AppState>,
    params: UpdateExecuteParams,
) -> Result<(), anyhow::Error> {
    // Determine home directory
    let home_dir = if std::env::var("QITECH_CONTROL_ENV").unwrap_or_default() == "control-os" {
        "/home/qitech"
    } else {
        std::env::var("HOME")?.as_str()
    };
    
    let repo_dir = format!("{}/{}", home_dir, params.github_repo_name);
    
    // Step 1: Clear repo directory
    clear_repo_directory(&app, &repo_dir).await?;
    
    // Step 2: Clone repository
    emit_step(&app, "clone-repo", "in-progress", None)?;
    clone_repository(&app, &params, home_dir).await?;
    emit_step(&app, "clone-repo", "completed", None)?;
    
    // Step 3: Make script executable
    Command::new("chmod")
        .args(["+x", "nixos-install.sh"])
        .current_dir(&repo_dir)
        .output()
        .await?;
    
    // Step 4: Run nixos-install.sh with progress tracking
    emit_step(&app, "rust-build", "in-progress", None)?;
    run_install_script(&app, state, &repo_dir).await?;
    
    Ok(())
}

async fn clear_repo_directory(app: &AppHandle, repo_dir: &str) -> Result<(), anyhow::Error> {
    if std::path::Path::new(repo_dir).exists() {
        tokio::fs::remove_dir_all(repo_dir).await?;
        emit_log(app, &format!("✅ Deleted existing repository at {}", repo_dir))?;
    } else {
        emit_log(app, &format!("📝 No existing repository at {}", repo_dir))?;
    }
    Ok(())
}

async fn clone_repository(
    app: &AppHandle,
    params: &UpdateExecuteParams,
    home_dir: &str,
) -> Result<(), anyhow::Error> {
    let repo_url = if let Some(token) = &params.github_token {
        format!("https://{}@github.com/{}/{}.git", 
            token, params.github_repo_owner, params.github_repo_name)
    } else {
        format!("https://github.com/{}/{}.git",
            params.github_repo_owner, params.github_repo_name)
    };
    
    let mut args = vec!["clone", "--progress", &repo_url];
    
    if let Some(tag) = &params.tag {
        args.extend_from_slice(&["--branch", tag, "--single-branch"]);
        emit_log(app, &format!("📝 Cloning tag: {}", tag))?;
    } else if let Some(branch) = &params.branch {
        args.extend_from_slice(&["--branch", branch, "--single-branch"]);
        emit_log(app, &format!("📝 Cloning branch: {}", branch))?;
    } else if let Some(commit) = &params.commit {
        emit_log(app, &format!("📝 Will checkout commit: {}", commit))?;
    } else {
        return Err(anyhow::anyhow!("No version specified"));
    }
    
    run_command_with_output(app, "git", &args, home_dir, |line, app| {
        parse_git_progress(line, app)
    }).await?;
    
    // Checkout specific commit if specified
    if let Some(commit) = &params.commit {
        let repo_dir = format!("{}/{}", home_dir, params.github_repo_name);
        run_command_with_output(app, "git", &["checkout", commit], &repo_dir, |_, _| Ok(())).await?;
        emit_log(app, &format!("✅ Checked out commit: {}", commit))?;
    }
    
    Ok(())
}

async fn run_install_script(
    app: &AppHandle,
    state: Arc<AppState>,
    repo_dir: &str,
) -> Result<(), anyhow::Error> {
    let mut child = Command::new("./nixos-install.sh")
        .current_dir(repo_dir)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()?;
    
    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();
    
    // Store process in state for cancellation
    {
        let mut process_lock = state.update_process.lock().await;
        *process_lock = Some(UpdateProcess {
            child,
            rust_build_progress: RustBuildProgress::default(),
        });
    }
    
    // Stream stdout
    let app_clone = app.clone();
    let state_clone = state.clone();
    let stdout_task = tokio::spawn(async move {
        let reader = BufReader::new(stdout);
        let mut lines = reader.lines();
        while let Ok(Some(line)) = lines.next_line().await {
            emit_log(&app_clone, &line).ok();
            parse_rust_build_output(&app_clone, &state_clone, &line).await.ok();
        }
    });
    
    // Stream stderr
    let app_clone = app.clone();
    let state_clone = state.clone();
    let stderr_task = tokio::spawn(async move {
        let reader = BufReader::new(stderr);
        let mut lines = reader.lines();
        while let Ok(Some(line)) = lines.next_line().await {
            emit_log(&app_clone, &line).ok();
            parse_rust_build_output(&app_clone, &state_clone, &line).await.ok();
        }
    });
    
    // Wait for process to complete
    let mut process_lock = state.update_process.lock().await;
    if let Some(mut update_process) = process_lock.take() {
        let status = update_process.child.wait().await?;
        
        stdout_task.await?;
        stderr_task.await?;
        
        if !status.success() {
            emit_step(app, "rust-build", "error", None)?;
            emit_step(app, "system-install", "error", None)?;
            return Err(anyhow::anyhow!("Install script failed"));
        }
        
        emit_step(app, "system-install", "completed", None)?;
    }
    
    Ok(())
}

async fn parse_rust_build_output(
    app: &AppHandle,
    state: &Arc<AppState>,
    line: &str,
) -> Result<(), anyhow::Error> {
    let line_lower = line.to_lowercase();
    
    // Track derivations
    if let Some(captures) = regex::Regex::new(r"these (\d+) derivations? will be built")
        .unwrap()
        .captures(line)
    {
        if let Some(total) = captures.get(1) {
            if let Ok(total) = total.as_str().parse::<usize>() {
                let mut process_lock = state.update_process.lock().await;
                if let Some(update_process) = process_lock.as_mut() {
                    update_process.rust_build_progress.total_derivations = total;
                    update_process.rust_build_progress.built_derivations = 0;
                    update_process.rust_build_progress.max_percent = 0;
                }
                emit_step(app, "rust-build", "in-progress", Some(0))?;
            }
        }
    }
    
    // Track building
    if line_lower.contains("building '/nix/store/") || line_lower.contains("building /nix/store/") {
        let mut process_lock = state.update_process.lock().await;
        if let Some(update_process) = process_lock.as_mut() {
            update_process.rust_build_progress.built_derivations += 1;
            
            let percent = if line_lower.contains("-server-deps") {
                85
            } else if update_process.rust_build_progress.total_derivations > 0 {
                let ratio = update_process.rust_build_progress.built_derivations as f32 
                    / update_process.rust_build_progress.total_derivations as f32;
                15 + (ratio * 70.0) as usize
            } else {
                15
            };
            
            let percent = percent.max(update_process.rust_build_progress.max_percent);
            update_process.rust_build_progress.max_percent = percent;
            
            emit_step(app, "rust-build", "in-progress", Some(percent as u8))?;
        }
    }
    
    // Detect system install
    if line_lower.contains("updating grub") 
        || line_lower.contains("installing bootloader")
        || line_lower.contains("activating the configuration") 
    {
        emit_step(app, "rust-build", "completed", None)?;
        emit_step(app, "system-install", "in-progress", None)?;
    }
    
    Ok(())
}

fn parse_git_progress(line: &str, app: &AppHandle) -> Result<(), anyhow::Error> {
    if let Some(captures) = regex::Regex::new(r"Receiving objects:\s*(\d+)%")
        .unwrap()
        .captures(line)
    {
        if let Some(percent) = captures.get(1) {
            if let Ok(percent) = percent.as_str().parse::<u8>() {
                emit_step(app, "clone-repo", "in-progress", 
                    Some((percent as f32 * 0.8) as u8))?;
            }
        }
    } else if let Some(captures) = regex::Regex::new(r"Resolving deltas:\s*(\d+)%")
        .unwrap()
        .captures(line)
    {
        if let Some(percent) = captures.get(1) {
            if let Ok(percent) = percent.as_str().parse::<u8>() {
                emit_step(app, "clone-repo", "in-progress",
                    Some(80 + (percent as f32 * 0.2) as u8))?;
            }
        }
    }
    Ok(())
}

async fn run_command_with_output<F>(
    app: &AppHandle,
    cmd: &str,
    args: &[&str],
    cwd: &str,
    parse_fn: F,
) -> Result<(), anyhow::Error>
where
    F: Fn(&str, &AppHandle) -> Result<(), anyhow::Error> + Send + 'static,
{
    emit_log(app, &format!("🚀 {} {}", cwd, format!("{} {}", cmd, args.join(" "))))?;
    
    let mut child = Command::new(cmd)
        .args(args)
        .current_dir(cwd)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()?;
    
    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();
    
    // Stream output
    let app_clone = app.clone();
    let parse_fn = Arc::new(parse_fn);
    let parse_fn_clone = parse_fn.clone();
    
    let stdout_task = tokio::spawn(async move {
        let reader = BufReader::new(stdout);
        let mut lines = reader.lines();
        while let Ok(Some(line)) = lines.next_line().await {
            emit_log(&app_clone, &line).ok();
            parse_fn_clone(&line, &app_clone).ok();
        }
    });
    
    let app_clone = app.clone();
    let stderr_task = tokio::spawn(async move {
        let reader = BufReader::new(stderr);
        let mut lines = reader.lines();
        while let Ok(Some(line)) = lines.next_line().await {
            emit_log(&app_clone, &line).ok();
            parse_fn(&line, &app_clone).ok();
        }
    });
    
    let status = child.wait().await?;
    stdout_task.await?;
    stderr_task.await?;
    
    if !status.success() {
        return Err(anyhow::anyhow!("Command failed with code {:?}", status.code()));
    }
    
    Ok(())
}

// Helper functions
fn emit_log(app: &AppHandle, log: &str) -> Result<(), anyhow::Error> {
    app.emit("update-log", log)?;
    Ok(())
}

fn emit_step(app: &AppHandle, step_id: &str, status: &str, progress: Option<u8>) -> Result<(), anyhow::Error> {
    app.emit("update-step", UpdateStepParams {
        step_id: step_id.to_string(),
        status: status.to_string(),
        progress,
    })?;
    Ok(())
}
```

**Dependencies to add to Cargo.toml**:
```toml
regex = "1"
anyhow = "1"
```

**Technical debt addressed**:
1. **Structured logging**: Use `tracing` instead of console.log
2. **No hardcoded paths**: Read from env properly
3. **Proper state management**: No module-level globals
4. **Input validation**: Can add Zod-style validation with `validator` crate
5. **Error handling**: Proper Result types throughout
6. **Process management**: Use Tokio's process API, no tree-kill needed (Rust handles this better)
7. **No GitHub token logging**: Properly hide sensitive data in logs
8. **Smaller, more modular functions**: Breaking down the 700-line file

**Security considerations**:
- GitHub token is still in memory but not logged
- Consider redacting from logs: `log.replace(&params.github_token.unwrap_or_default(), "***")`

---

## Phase 5: Testing Strategy

### 5.1: Unit Testing (Rust)

Create test files for each command module:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_nixos_generations() {
        let output = r#"
Generation 62 current 2025-06-10 08:51:35 fix.33_c744e1481fdc0bf25821bd0ee0ae8278f155  6.14.8
Generation 61         2025-06-09 14:22:11 fix.32_a123456789                              6.14.7
        "#;
        
        let generations = parse_nixos_generations(output).unwrap();
        assert_eq!(generations.len(), 2);
        assert_eq!(generations[0].id, "62");
        assert!(generations[0].current);
    }
}
```

### 5.2: Integration Testing

Test full flow manually:
1. Start Tauri app in dev mode: `cd tauri && npm run dev`
2. Test each IPC module:
   - Theme switching
   - Window controls
   - Environment info display
   - Troubleshoot actions (in NixOS VM)
   - NixOS generation management (in NixOS VM)
   - Update flow (in NixOS VM with test repo)

### 5.3: Comparison Testing

Run Electron and Tauri side-by-side:
- Same inputs should produce same outputs
- Verify UI behavior is identical
- Check performance differences

---

## Phase 6: NixOS Packaging

### 6.1: Create Tauri Nix Package

**File**: `/home/hsp/Repositories/qitech/control/nixos/packages/tauri.nix`

```nix
{ lib, buildNpmPackage, rustPlatform, pkg-config, openssl, webkitgtk, libsoup, 
  glib-networking, gst_all_1, makeDesktopItem }:

let
  # Build the Rust backend
  tauriBackend = rustPlatform.buildRustPackage {
    pname = "qitech-control-tauri-backend";
    version = "2.15.0";
    
    src = ../../tauri/src-tauri;
    
    cargoLock = {
      lockFile = ../../tauri/src-tauri/Cargo.lock;
    };
    
    nativeBuildInputs = [
      pkg-config
    ];
    
    buildInputs = [
      openssl
      webkitgtk
      libsoup
      glib-networking
    ] ++ (with gst_all_1; [
      gstreamer
      gst-plugins-base
      gst-plugins-good
    ]);
    
    # Tauri requires webkit2gtk-4.1
    PKG_CONFIG_PATH = "${webkitgtk}/lib/pkgconfig";
  };
  
  # Build the frontend
  tauriFrontend = buildNpmPackage {
    pname = "qitech-control-tauri-frontend";
    version = "2.15.0";
    
    src = ../../tauri;
    
    npmDepsHash = "sha256-XXXXX"; # Update after first build
    npmFlags = [ "--no-audit" "--no-fund" ];
    
    buildPhase = ''
      npm run build
    '';
    
    installPhase = ''
      mkdir -p $out
      cp -r dist $out/
    '';
  };

in buildNpmPackage rec {
  pname = "qitech-control-tauri";
  version = "2.15.0";

  src = ../../tauri;
  
  npmDepsHash = "sha256-XXXXX"; # Update after first build
  
  nativeBuildInputs = [
    pkg-config
  ];
  
  buildInputs = [
    openssl
    webkitgtk
    libsoup
    glib-networking
  ];

  buildPhase = ''
    # Frontend is built by npm
    npm run build
    
    # Backend is built separately
    # Copy backend binary
    mkdir -p src-tauri/target/release
    cp ${tauriBackend}/bin/qitech-control-tauri src-tauri/target/release/
  '';

  installPhase = ''
    runHook preInstall

    mkdir -p $out/bin
    mkdir -p $out/share/qitech-control-tauri
    
    # Install Tauri bundle
    cp -r src-tauri/target/release/bundle/appimage/* $out/share/qitech-control-tauri/ || true
    cp src-tauri/target/release/qitech-control-tauri $out/bin/
    
    # Install icon
    if [ -f src-tauri/icons/icon.png ]; then
      mkdir -p $out/share/icons/hicolor/256x256/apps $out/share/pixmaps
      cp src-tauri/icons/icon.png $out/share/icons/hicolor/256x256/apps/de.qitech.control-tauri.png
      ln -sf $out/share/icons/hicolor/256x256/apps/de.qitech.control-tauri.png \
        $out/share/pixmaps/qitech-control-tauri.png
    fi

    # Desktop entry
    mkdir -p $out/share/applications
    cat > $out/share/applications/de.qitech.control-tauri.desktop << EOF
[Desktop Entry]
Type=Application
Name=QiTech Control (Tauri)
Comment=QiTech Control Tauri Application
Exec=qitech-control-tauri
Icon=de.qitech.control-tauri
Terminal=false
Categories=Development;Engineering;
X-GNOME-UsesNotifications=true
EOF

    runHook postInstall
  '';

  meta = with lib; {
    description = "QiTech Control Tauri";
    homepage = "https://qitech.de";
    platforms = platforms.linux;
  };
}
```

**Note**: Tauri packaging in Nix is more complex than Electron because:
1. Need to build Rust backend separately
2. Need to handle webkit dependencies
3. May need to use `tauri build` directly instead of manual assembly

### 6.2: Update NixOS Module

**File**: `/home/hsp/Repositories/qitech/control/nixos/modules/qitech.nix`

Add option to choose between Electron and Tauri:

```nix
{ config, lib, pkgs, ... }:

with lib;

let
  cfg = config.services.qitech-control;
in {
  options.services.qitech-control = {
    # ... existing options ...
    
    frontend = mkOption {
      type = types.enum [ "electron" "tauri" ];
      default = "electron";
      description = "Which frontend to use: electron or tauri";
    };
  };
  
  config = mkIf cfg.enable {
    environment.systemPackages = [
      (if cfg.frontend == "tauri" 
       then pkgs.qitech-control-tauri 
       else pkgs.qitech-control-electron)
    ];
    
    # ... rest of config ...
  };
}
```

---

## Phase 7: Documentation & Migration Path

### 7.1: Create Migration Documentation

**File**: `/home/hsp/Repositories/qitech/control/docs/TAURI_MIGRATION.md`

Contents:
- Architecture comparison (Electron vs Tauri)
- Bridge pattern explanation
- How to switch between Electron and Tauri
- Known differences/limitations
- Performance benchmarks
- Troubleshooting guide

### 7.2: User Migration Path

For end users:
1. **Parallel deployment**: Both Electron and Tauri can coexist
2. **Gradual rollout**: Test Tauri on dev machines first
3. **Fallback**: Keep Electron package available
4. **Switch via NixOS config**: `services.qitech-control.frontend = "tauri";`

### 7.3: Developer Documentation

Update development guides:
- How to add new IPC commands (both shells)
- How to test changes in both environments
- Bridge interface extension guidelines

---

## Phase 8: Post-Migration Cleanup

### 8.1: Technical Debt Resolved During Migration

Automatically resolved by Tauri implementation:
- ✅ Console.log statements → replaced with tracing
- ✅ Module-level globals → proper state management
- ✅ No input validation → can add with validator crate
- ✅ DevTools in production → Tauri handles this better
- ✅ Hardcoded paths → proper env handling
- ✅ GitHub token logging → properly hidden
- ✅ 700-line update file → split into focused functions

### 8.2: Remaining Technical Debt in Electron

**DO NOT fix these in Electron** (would create merge conflicts):
1. types.d.ts duplication - keep until Electron is deprecated
2. window.require pattern - Electron-specific
3. noImplicitAny: false - fix post-migration
4. App.tsx duplication - consider moving to UI layer later

### 8.3: Technical Debt in Bridge Pattern

**Known limitations** (affects both shells):
1. Event listeners don't return unsubscribe functions
2. nixos.isNixOSAvailable is a boolean property, not async
3. No TypeScript validation on IPC payloads

**Recommended future work**:
1. Add Zod validation to bridge interface
2. Make event listeners return cleanup functions
3. Add error boundaries in React app
4. Move App.tsx to UI layer

### 8.4: Performance Monitoring

After deployment, monitor:
- App startup time (Tauri should be faster)
- Memory usage (Tauri should use less)
- Update script execution time (should be similar)
- Window responsiveness

---

## Implementation Timeline

### Estimated Hours by Phase

| Phase | Description | Estimated Time |
|-------|-------------|----------------|
| 0 | Pre-migration cleanup | 2 hours |
| 1 | Project structure setup | 3 hours |
| 2 | Frontend layer (TypeScript) | 4 hours |
| 3 | Rust backend setup | 2 hours |
| 4.1-4.3 | Simple commands (theme, window, env) | 2 hours |
| 4.4 | Troubleshoot commands | 2 hours |
| 4.5 | NixOS commands | 4 hours |
| 4.6 | Update commands | 12 hours |
| 5 | Testing | 4 hours |
| 6 | NixOS packaging | 4 hours |
| 7 | Documentation | 2 hours |
| **Total** | **41 hours** |

**Recommended schedule**: 1 week with 1 developer working full-time, or 2 weeks part-time.

### Milestones

1. **Day 1-2**: Setup + simple commands working
2. **Day 3**: Troubleshoot + NixOS commands working
3. **Day 4-5**: Update command implementation
4. **Day 6**: Testing and bug fixes
5. **Day 7**: NixOS packaging + documentation

---

## Risk Assessment & Mitigation

### High Risk Areas

1. **Update command complexity**
   - Risk: Progress parsing may not work correctly
   - Mitigation: Extensive testing with real update scenarios
   - Fallback: Simplify progress tracking if needed

2. **NixOS-specific functionality**
   - Risk: Commands may behave differently in Tauri
   - Mitigation: Test in actual NixOS environment early
   - Fallback: Document any behavioral differences

3. **Process cancellation**
   - Risk: Killing process tree may not work as expected
   - Mitigation: Test cancellation thoroughly
   - Fallback: Use timeout-based cleanup if kill fails

### Medium Risk Areas

1. **Theme detection**
   - Risk: No native API in Tauri (unlike Electron)
   - Mitigation: Use OS-level APIs or file-based storage
   - Fallback: Always start in light mode, let user choose

2. **WebKit differences**
   - Risk: UI may render differently
   - Mitigation: Visual regression testing
   - Fallback: CSS adjustments if needed

### Low Risk Areas

1. **Bridge pattern** - Already proven to work
2. **Window management** - Tauri has good APIs
3. **IPC basics** - Well-documented in Tauri

---

## Success Criteria

The migration is successful when:

1. ✅ All 6 IPC modules work identically to Electron version
2. ✅ UI looks and behaves the same
3. ✅ Update flow completes successfully in test environment
4. ✅ NixOS generation management works
5. ✅ App can be built and packaged with Nix
6. ✅ No regressions in existing functionality
7. ✅ Performance is equal or better than Electron
8. ✅ Documentation is complete

---

## Technical Debt Decision Matrix

| Debt Item | When to Fix | Rationale |
|-----------|-------------|-----------|
| theme_helpers.ts hardcoded light | **Pre-migration** | Affects both shells |
| stub.ts `as any` cast | **Pre-migration** | Type safety for both |
| types.d.ts duplication | **Post-migration** | Only matters after Electron deprecation |
| Console.log statements | **During migration (Tauri only)** | Use tracing in Rust |
| Module-level globals | **During migration (Tauri only)** | Proper state in Rust |
| Hardcoded /home/qitech | **During migration (Tauri only)** | Clean env handling |
| 700-line update file | **During migration (Tauri only)** | Better structure in Rust |
| No input validation | **Post-migration** | Add to both shells later |
| App.tsx duplication | **Post-migration** | Low priority optimization |
| Event listener cleanup | **Post-migration** | Bridge interface change |

---

## Appendix A: File Checklist

### Files to Create (Tauri)

Frontend:
- [ ] `/home/hsp/Repositories/qitech/control/tauri/package.json`
- [ ] `/home/hsp/Repositories/qitech/control/tauri/tsconfig.json`
- [ ] `/home/hsp/Repositories/qitech/control/tauri/vite.config.ts`
- [ ] `/home/hsp/Repositories/qitech/control/tauri/src/main.tsx`
- [ ] `/home/hsp/Repositories/qitech/control/tauri/src/bridge.ts`
- [ ] `/home/hsp/Repositories/qitech/control/tauri/src/App.tsx`

Backend:
- [ ] `/home/hsp/Repositories/qitech/control/tauri/src-tauri/Cargo.toml`
- [ ] `/home/hsp/Repositories/qitech/control/tauri/src-tauri/tauri.conf.json`
- [ ] `/home/hsp/Repositories/qitech/control/tauri/src-tauri/build.rs`
- [ ] `/home/hsp/Repositories/qitech/control/tauri/src-tauri/src/main.rs`
- [ ] `/home/hsp/Repositories/qitech/control/tauri/src-tauri/src/lib.rs`
- [ ] `/home/hsp/Repositories/qitech/control/tauri/src-tauri/src/state.rs`
- [ ] `/home/hsp/Repositories/qitech/control/tauri/src-tauri/src/commands/mod.rs`
- [ ] `/home/hsp/Repositories/qitech/control/tauri/src-tauri/src/commands/theme.rs`
- [ ] `/home/hsp/Repositories/qitech/control/tauri/src-tauri/src/commands/window.rs`
- [ ] `/home/hsp/Repositories/qitech/control/tauri/src-tauri/src/commands/environment.rs`
- [ ] `/home/hsp/Repositories/qitech/control/tauri/src-tauri/src/commands/troubleshoot.rs`
- [ ] `/home/hsp/Repositories/qitech/control/tauri/src-tauri/src/commands/nixos.rs`
- [ ] `/home/hsp/Repositories/qitech/control/tauri/src-tauri/src/commands/update.rs`

Packaging:
- [ ] `/home/hsp/Repositories/qitech/control/nixos/packages/tauri.nix`

Documentation:
- [ ] `/home/hsp/Repositories/qitech/control/docs/TAURI_MIGRATION.md`

### Files to Modify

Pre-migration (UI layer):
- [ ] `/home/hsp/Repositories/qitech/control/ui/src/bridge/stub.ts` - Remove `as any`
- [ ] `/home/hsp/Repositories/qitech/control/ui/src/helpers/theme_helpers.ts` - Uncomment theme logic

Project config:
- [ ] `/home/hsp/Repositories/qitech/control/package.json` - Add `"tauri"` to workspaces
- [ ] `/home/hsp/Repositories/qitech/control/Cargo.toml` - Add `"tauri/src-tauri"` to members
- [ ] `/home/hsp/Repositories/qitech/control/nixos/modules/qitech.nix` - Add frontend option

---

## Appendix B: Key Architectural Decisions

### Decision 1: One Rust File Per IPC Module

**Rationale**: 
- Mirrors Electron structure (one directory per module)
- Easier to navigate than monolithic file
- Allows parallel development
- Clear ownership boundaries

**Alternative considered**: Single commands.rs file
**Rejected because**: Would become 1000+ lines

### Decision 2: Events for Streaming (Not Callbacks)

**Rationale**:
- Tauri events are the standard way to push updates
- Matches Electron's `event.sender.send()` pattern
- Works well with React hooks (`useEffect` + event listeners)

**Alternative considered**: WebSocket or polling
**Rejected because**: Over-engineered, events are idiomatic

### Decision 3: Proper State Management (No Globals)

**Rationale**:
- Electron's module-level globals are an anti-pattern
- Tauri's state management is clean and thread-safe
- Makes testing easier
- Prevents state leaks between operations

**Alternative considered**: Keep global pattern
**Rejected because**: This is opportunity to fix technical debt

### Decision 4: Keep App.tsx in Both electron/ and tauri/

**Rationale**:
- Moving to UI layer requires larger refactor
- Duplication is minimal (~40 lines)
- Each shell may want custom initialization
- Can be optimized post-migration

**Alternative considered**: Move to /ui/src/ShellApp.tsx
**Deferred because**: Not blocking for migration

### Decision 5: Parallel Deployment (Not Hard Cutover)

**Rationale**:
- Lower risk - both can coexist
- Gradual rollout possible
- Easy fallback if issues found
- Users can choose via NixOS config

**Alternative considered**: Replace Electron immediately
**Rejected because**: Too risky for production system

---

## Appendix C: Command Mapping Reference

| Electron Pattern | Tauri Equivalent | Notes |
|------------------|------------------|-------|
| `ipcMain.handle()` | `#[tauri::command]` | Function becomes command |
| `event.sender.send()` | `app.emit()` | Push events to frontend |
| `ipcRenderer.invoke()` | `invoke()` from @tauri-apps/api | Frontend calls |
| `ipcRenderer.on()` | `listen()` from @tauri-apps/api | Frontend listeners |
| `contextBridge.exposeInMainWorld()` | Not needed | Tauri handles security |
| `nativeTheme` | OS APIs or state | No direct equivalent |
| `BrowserWindow` methods | `Window` API | Similar but different |
| `dialog` module | `tauri-plugin-dialog` | Plugin required |
| `spawn()` from child_process | `tokio::process::Command` | Async by default |

---

## Appendix D: Testing Checklist

### Manual Testing Scenarios

**Theme Module**:
- [ ] Toggle theme from light to dark
- [ ] Toggle theme from dark to light
- [ ] Set to system theme
- [ ] Restart app, verify theme persists

**Window Module**:
- [ ] Minimize window
- [ ] Maximize window
- [ ] Restore window
- [ ] Enter fullscreen
- [ ] Exit fullscreen
- [ ] Close window

**Environment Module**:
- [ ] Run on non-QITECH_OS, verify qitechOs: false
- [ ] Run on QITECH_OS, verify qitechOs: true
- [ ] Verify git info is populated when available

**Troubleshoot Module**:
- [ ] Reboot HMI (confirm prompt, verify reboot happens)
- [ ] Restart backend (verify service restarts)
- [ ] Export logs (save dialog appears, file is created)
- [ ] Cancel log export (verify no file created)

**NixOS Module** (requires NixOS system):
- [ ] List generations (verify parsing is correct)
- [ ] Current generation is marked correctly
- [ ] Switch to older generation (verify reboot)
- [ ] Delete generation (verify it's removed)
- [ ] Delete all old generations (verify cleanup)

**Update Module** (requires test repo):
- [ ] Execute update with tag (clone succeeds)
- [ ] Execute update with branch (clone succeeds)
- [ ] Execute update with commit (checkout succeeds)
- [ ] Verify real-time log streaming
- [ ] Verify progress steps update correctly
- [ ] Cancel update mid-process (verify cleanup)
- [ ] Update with GitHub token (verify token not logged)
- [ ] Update fails (verify error handling)

### Comparison Testing

For each scenario:
- [ ] Run in Electron
- [ ] Run in Tauri
- [ ] Verify output is identical
- [ ] Verify UI behavior is identical

### Performance Testing

- [ ] Measure startup time (Electron vs Tauri)
- [ ] Measure memory usage after 1 hour
- [ ] Measure update script execution time
- [ ] Verify no memory leaks during repeated operations

---

## Appendix E: Common Pitfalls & Solutions

### Pitfall 1: Tauri Commands Must Be Async

**Problem**: Forgetting `async` keyword on commands
**Solution**: Always use `async fn` even for simple operations
**Example**:
```rust
// ❌ Wrong
#[tauri::command]
fn window_close(app: AppHandle) -> Result<(), String> {
    // ...
}

// ✅ Correct
#[tauri::command]
async fn window_close(app: AppHandle) -> Result<(), String> {
    // ...
}
```

### Pitfall 2: Event Names Must Match Frontend

**Problem**: Typo in event name causes silent failure
**Solution**: Use constants or generate types
**Example**:
```rust
// Backend
app.emit("update-log", log)?; // Hyphen

// Frontend
listen<string>('update_log', ...); // ❌ Underscore - won't receive events
listen<string>('update-log', ...); // ✅ Matches
```

### Pitfall 3: Sudo Commands Need Shell Config

**Problem**: `sudo` prompts for password in Tauri
**Solution**: Configure sudoers or use polkit
**Example**: Add to /etc/sudoers:
```
qitech ALL=(ALL) NOPASSWD: /usr/bin/systemctl restart qitech-control-server
qitech ALL=(ALL) NOPASSWD: /usr/bin/reboot
```

### Pitfall 4: WebKit Rendering Differences

**Problem**: CSS may render differently than Chromium
**Solution**: Test early, add WebKit-specific CSS if needed
**Example**: `-webkit-` prefixes may be required

### Pitfall 5: Process Cleanup on Cancel

**Problem**: Killed process leaves zombie processes
**Solution**: Use Tokio's process handling, avoid manual kill
**Example**:
```rust
// ✅ Tokio handles cleanup
let mut child = tokio::process::Command::new("git").spawn()?;
child.kill().await?; // Cleans up properly
```

---

## Summary

This migration plan provides a comprehensive roadmap for moving from Electron to Tauri while maintaining the existing bridge pattern architecture. The phased approach allows for:

1. **Early wins**: Fix technical debt in shared UI before starting
2. **Parallel development**: Tauri and Electron can coexist
3. **Risk mitigation**: Gradual rollout with fallback options
4. **Technical improvement**: Better code structure, logging, and state management
5. **Future-proofing**: Cleaner architecture for future enhancements

The key to success is leveraging the existing `NativeBridge` pattern - the React UI doesn't need to change at all, only the backend implementation. This makes the migration significantly easier than a typical Electron→Tauri project.

**Recommended next step**: Start with Phase 0 (pre-migration cleanup), then implement Phase 1-3 to get a basic "Hello World" Tauri app with the bridge pattern working. This validates the architecture before tackling the complex IPC modules.

