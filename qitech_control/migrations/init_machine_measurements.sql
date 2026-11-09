CREATE TABLE IF NOT EXISTS  control_hub.machine_measurements(
    timestamp DateTime64(3, 'UTC'),
    identity  UInt64,
    name      LowCardinality(String),
    value     Nullable(Float64),

    PROJECTION by_time(
        SELECT *
        ORDER BY timestamp
    )
)

ENGINE = MergeTree

PARTITION BY toYYYYMMDD(timestamp)
ORDER BY (identity, name, timestamp)

-- expiry time for data (30 Days)
TTL timestamp + INTERVAL 30 DAY
