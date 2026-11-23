CREATE TABLE IF NOT EXISTS  control_hub.machine_measurements(
    -- 32 Hz rows: delta-of-delta makes the timestamps nearly free
    timestamp DateTime64(3, 'UTC') CODEC(DoubleDelta, ZSTD(1)),
    identity  UInt64,
    name      LowCardinality(String),
    -- noisy sensor values compress better with plain ZSTD than with Gorilla
    value     Nullable(Float64) CODEC(ZSTD(3))
)

ENGINE = MergeTree

PARTITION BY toYYYYMMDD(timestamp)
ORDER BY (identity, name, timestamp)

-- expiry time for data (30 Days)
TTL timestamp + INTERVAL 30 DAY
