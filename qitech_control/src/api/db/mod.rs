//! ClickHouse: connection and migrations, and the hub listener that records every
//! runtime report.

mod migrations;
pub(crate) mod tables;
mod writer;

pub use migrations::connect;
pub use writer::ClickhouseWriter;

/// The ClickHouse database that holds every table of control.
pub const DATABASE: &str = "control_hub";
