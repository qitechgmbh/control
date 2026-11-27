//! Row types the [`ClickhouseWriter`](super::ClickhouseWriter) inserts, one module per
//! table (see `migrations/init_*.sql`).

use chrono::DateTime;
use chrono::Utc;
use serde::Serialize;

/// A runtime record that could not be turned into a row.
#[derive(Debug)]
pub struct RecordError {
    pub path: String,
    pub error: serde_json::Error,
}

impl std::fmt::Display for RecordError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "record `{}`: {}", self.path, self.error)
    }
}

pub mod machine_config_property_records {
    use qitech_framework::ConfigPropertyEvent;
    use qitech_framework::ConfigPropertyEventRecord;

    use super::*;

    pub const TABLE_NAME: &str = "machine_config_property_records";

    /// Values of the `event_type` `Enum8` column.
    #[repr(i8)]
    enum EventType {
        Registered = 1,
        DefaultChanged = 2,
        CapabilityChanged = 3,
        ConstraintsChanged = 4,
        Written = 5,
    }

    impl From<&ConfigPropertyEvent> for EventType {
        fn from(event: &ConfigPropertyEvent) -> Self {
            match event {
                ConfigPropertyEvent::Registered { .. } => Self::Registered,
                ConfigPropertyEvent::DefaultChanged(_) => Self::DefaultChanged,
                ConfigPropertyEvent::CapabilityChanged(_) => Self::CapabilityChanged,
                ConfigPropertyEvent::ConstraintsChanged(_) => Self::ConstraintsChanged,
                ConfigPropertyEvent::Written { .. } => Self::Written,
            }
        }
    }

    #[derive(Debug, Serialize, clickhouse::Row)]
    pub struct Row {
        #[serde(with = "clickhouse::serde::chrono::datetime64::millis")]
        pub timestamp: DateTime<Utc>,
        pub identity: u64,
        pub name: String,
        pub event_type: i8,
        pub event_data: String,
    }

    impl TryFrom<&ConfigPropertyEventRecord> for Row {
        type Error = RecordError;

        fn try_from(record: &ConfigPropertyEventRecord) -> Result<Self, Self::Error> {
            let event_data = serde_json::to_string(&record.event).map_err(|error| RecordError {
                path: record.path.clone(),
                error,
            })?;
            Ok(Self {
                timestamp: record.timestamp,
                identity: record.machine.to_u64(),
                name: record.path.clone(),
                event_type: EventType::from(&record.event) as i8,
                event_data,
            })
        }
    }
}

pub mod machine_state_property_records {
    use qitech_framework::StatePropertyEvent;
    use qitech_framework::StatePropertyEventRecord;

    use super::*;

    pub const TABLE_NAME: &str = "machine_state_property_records";

    /// Values of the `event_type` `Enum8` column.
    #[repr(i8)]
    enum EventType {
        Registered = 1,
        ValueChanged = 2,
    }

    #[derive(Debug, Serialize, clickhouse::Row)]
    pub struct Row {
        #[serde(with = "clickhouse::serde::chrono::datetime64::millis")]
        pub timestamp: DateTime<Utc>,
        pub identity: u64,
        pub name: String,
        pub event_type: i8,
        pub event_data: String,
    }

    impl TryFrom<&StatePropertyEventRecord> for Row {
        type Error = RecordError;

        fn try_from(record: &StatePropertyEventRecord) -> Result<Self, Self::Error> {
            let (event_type, value) = match &record.event {
                StatePropertyEvent::Registered { value } => (EventType::Registered, value),
                StatePropertyEvent::ValueChanged { value } => (EventType::ValueChanged, value),
            };
            // `json!` would panic on a value that fails to serialize, so convert it first.
            let value = serde_json::to_value(value).map_err(|error| RecordError {
                path: record.path.clone(),
                error,
            })?;
            Ok(Self {
                timestamp: record.timestamp,
                identity: record.machine.to_u64(),
                name: record.path.clone(),
                event_type: event_type as i8,
                event_data: serde_json::json!({ "value": value }).to_string(),
            })
        }
    }
}

pub mod machine_measurements {
    use super::*;

    pub const TABLE_NAME: &str = "machine_measurements";

    #[derive(Debug, Serialize, clickhouse::Row)]
    pub struct Row {
        #[serde(with = "clickhouse::serde::chrono::datetime64::millis")]
        pub timestamp: DateTime<Utc>,
        pub identity: u64,
        pub name: String,
        pub value: Option<f64>,
    }
}

pub mod machine_events {
    use super::*;

    pub const TABLE_NAME: &str = "machine_events";

    #[derive(Debug, Serialize, clickhouse::Row)]
    pub struct Row {
        #[serde(with = "clickhouse::serde::chrono::datetime64::millis")]
        pub timestamp: DateTime<Utc>,
        pub identity: u64,
        pub name: String,
        pub data: String,
    }
}
