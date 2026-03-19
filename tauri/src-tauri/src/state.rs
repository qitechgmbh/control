use std::sync::Arc;
use tokio::process::Child;
use tokio::sync::Mutex;

/// Shared application state
#[derive(Clone)]
pub struct AppState {
    /// Handle to the currently running update process
    pub update_process: Arc<Mutex<Option<UpdateProcess>>>,
    /// Current theme mode
    pub theme: Arc<Mutex<String>>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            update_process: Arc::new(Mutex::new(None)),
            theme: Arc::new(Mutex::new("system".to_string())),
        }
    }
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
