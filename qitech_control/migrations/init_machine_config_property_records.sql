CREATE TABLE IF NOT EXISTS control_hub.machine_config_property_records 
(
    timestamp       DateTime64(3, 'UTC'),
    identity        UInt64,
    name            LowCardinality(String),

    -- values match `EventType` in src/api/db/tables.rs --
    event_type Enum8(
        'Registered' = 1,
        'DefaultChanged' = 2,
        'CapabilityChanged' = 3,
        'ConstraintsChanged' = 4,
        'Written' = 5
    ),
    
    event_data JSON,

    -- store duplicate ordered by timestamp only for history queries of all machines
    PROJECTION by_time(
        SELECT *
        ORDER BY timestamp
    )
)

ENGINE = MergeTree
PARTITION BY toYYYYMM(timestamp)
ORDER BY (identity, timestamp)
