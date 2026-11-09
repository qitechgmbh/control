CREATE TABLE IF NOT EXISTS control_hub.machine_config_property_records 
(
    timestamp       DateTime64(3, 'UTC'),
    identity        UInt64,
    name            LowCardinality(String),

    -- value --
    event_type Enum8(
        'Registered', 
        'DefaultChanged', 
        'CapabilityChanged', 
        'ConstraintsChanged', 
        'Written'
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
