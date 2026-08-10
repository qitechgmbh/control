//! Where the server keeps state that must survive a restart.
//!
//! Only the directory resolution lives here. What goes in it is each subsystem's business — this
//! exists so there is one answer to "where is that", rather than the same environment-variable
//! chain copied into every caller that needs a file.

use std::path::PathBuf;

/// Directory for persistent state.
///
/// `STATE_DIRECTORY` is what systemd sets for a unit with `StateDirectory=`, and is the real
/// answer in production. The rest are fallbacks for running outside a unit, ending at the working
/// directory so a developer run never fails outright for want of a home directory.
pub fn state_dir() -> PathBuf {
    std::env::var_os("STATE_DIRECTORY")
        .or_else(|| std::env::var_os("XDG_DATA_HOME"))
        .or_else(|| std::env::var_os("HOME"))
        .map_or_else(|| PathBuf::from("."), PathBuf::from)
}

/// Full path to a file in the state directory.
pub fn state_path(file_name: &str) -> PathBuf {
    state_dir().join(file_name)
}
