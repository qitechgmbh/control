// Feature-specific modules for Gluetex machine

pub mod clamp_revolution;
pub mod filament_tension;

// Re-export feature types
pub use clamp_revolution::*;
pub use filament_tension::*;
