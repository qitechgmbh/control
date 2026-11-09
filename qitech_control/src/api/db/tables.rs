use chrono::DateTime;
use chrono::Utc;
use serde::Deserialize;
use serde::Serialize;

#[allow(dead_code)]
pub mod machine_config_property_records {
    use qitech_framework::ConfigPropertyEvent;
    use qitech_framework::ConfigPropertyEventRecord;

    use super::*;

    pub const TABLE_NAME: &str = "machine_config_property_records";
    pub const COLUMNS: &str = "timestamp, identity, name, event_type, event_data";

    pub const TIMESTAMP: &str = "timestamp";
    pub const IDENTITY: &str = "identity";
    pub const NAME: &str = "name";
    pub const EVENT_TYPE: &str = "event_type";
    pub const EVENT_DATA: &str = "event_data";

    pub type Timestamp = DateTime<Utc>;
    pub type Identity = u64;
    pub type Name = String;

    #[derive(Debug, Serialize, Deserialize, clickhouse::Row)]
    pub struct Row {
        #[serde(with = "clickhouse::serde::chrono::datetime64::millis")]
        pub timestamp: Timestamp,

        pub identity: Identity,
        pub name: Name,

        pub event_type: i8,
        pub event_data: String,
    }

    impl From<ConfigPropertyEventRecord> for Row {
        fn from(record: ConfigPropertyEventRecord) -> Self {
            Self {
                timestamp: record.timestamp,
                identity: record.machine.to_u64(),
                name: record.path,
                event_type: match &record.event {
                    ConfigPropertyEvent::Registered { .. } => 1,
                    ConfigPropertyEvent::DefaultChanged(_) => 2,
                    ConfigPropertyEvent::CapabilityChanged(_) => 3,
                    ConfigPropertyEvent::ConstraintsChanged(_) => 4,
                    ConfigPropertyEvent::Written { .. } => 5,
                },
                event_data: serde_json::to_string(&record.event).unwrap(),
            }
        }
    }
}

#[allow(dead_code)]
pub mod machine_state_property_records {
    use qitech_framework::StatePropertyEvent;
    use qitech_framework::StatePropertyEventRecord;

    use super::*;

    pub const TABLE_NAME: &str = "machine_state_property_records";
    pub const COLUMNS: &str = "timestamp, identity, name, event_type, event_data";

    pub const TIMESTAMP: &str = "timestamp";
    pub const IDENTITY: &str = "identity";
    pub const NAME: &str = "name";
    pub const EVENT_TYPE: &str = "event_type";
    pub const EVENT_DATA: &str = "event_data";

    pub type Timestamp = DateTime<Utc>;
    pub type Identity = u64;
    pub type Name = String;

    #[derive(Debug, Serialize, Deserialize, clickhouse::Row)]
    pub struct Row {
        #[serde(with = "clickhouse::serde::chrono::datetime64::millis")]
        pub timestamp: Timestamp,

        pub identity: Identity,
        pub name: Name,

        pub event_type: i8,
        pub event_data: String,
    }

    impl From<StatePropertyEventRecord> for Row {
        fn from(record: StatePropertyEventRecord) -> Self {
            let value = match &record.event {
                StatePropertyEvent::Registered { value } => value,
                StatePropertyEvent::ValueChanged { value } => value,
            };

            Self {
                timestamp: record.timestamp,
                identity: record.machine.to_u64(),
                name: record.path,
                event_type: match &record.event {
                    StatePropertyEvent::Registered { .. } => 1,
                    StatePropertyEvent::ValueChanged { .. } => 2,
                },
                event_data: serde_json::json!({ "value": value }).to_string(),
            }
        }
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub enum EventType {
        Registered,
        ValueChanged,
    }
}

#[allow(dead_code)]
pub mod machine_measurements {
    use super::*;

    pub const TABLE_NAME: &str = "machine_measurements";
    pub const COLUMNS: &str = "timestamp, identity, name, value";

    pub const TIMESTAMP: &str = "timestamp";
    pub const IDENTITY: &str = "identity";
    pub const NAME: &str = "name";
    pub const VALUE: &str = "value";

    pub type Timestamp = DateTime<Utc>;
    pub type Identity = u64;
    pub type Name = String;
    pub type Value = Option<f64>;

    #[derive(Debug, Serialize, Deserialize, clickhouse::Row)]
    pub struct Row {
        #[serde(with = "clickhouse::serde::chrono::datetime64::millis")]
        pub timestamp: Timestamp,

        pub identity: Identity,
        pub name: Name,
        pub value: Value,
    }
}

#[allow(dead_code)]
pub mod machine_events {
    use super::*;

    pub const TABLE_NAME: &str = "machine_events";
    pub const COLUMNS: &str = "timestamp, identity, name, data";

    pub const TIMESTAMP: &str = "timestamp";
    pub const IDENTITY: &str = "identity";
    pub const NAME: &str = "name";
    pub const DATA: &str = "data";

    pub type Timestamp = DateTime<Utc>;
    pub type Identity = u64;
    pub type Name = String;
    pub type Data = String;

    #[derive(Debug, Serialize, Deserialize, clickhouse::Row)]
    pub struct Row {
        #[serde(with = "clickhouse::serde::chrono::datetime64::millis")]
        pub timestamp: Timestamp,

        pub identity: Identity,
        pub name: Name,
        pub data: Data,
    }
}
