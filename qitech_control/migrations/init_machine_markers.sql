CREATE TABLE IF NOT EXISTS control_hub.machine_markers(
    id         UUID,
    identity   UInt64,
    timestamp  DateTime64(3, 'UTC'),
    name       String,
    color      String,
    created_at DateTime64(3, 'UTC') DEFAULT now64(3)
)

ENGINE = MergeTree

ORDER BY (identity, timestamp, id)

-- markers are only useful while the measurements under them exist (30 days)
TTL timestamp + INTERVAL 30 DAY
