CREATE TABLE IF NOT EXISTS  control_hub.machine_measurements(
    -- 32 Hz rows: delta-of-delta makes the timestamps nearly free
    timestamp DateTime64(3, 'UTC') CODEC(DoubleDelta, ZSTD(1)),
    identity  UInt64,
    name      LowCardinality(String),
    -- noisy sensor values compress better with plain ZSTD than with Gorilla
    value     Nullable(Float64) CODEC(ZSTD(3)),

    -- Pre-aggregated count/sum/min/max (and summed sample time) per series and 5 s /
    -- 1 min slot, for the history graphs: a 30 day range reads ~43k rows per series from
    -- p_1m instead of ~83M raw rows. ClickHouse keeps them in step with every insert,
    -- merge and TTL drop. The query in api/legacy/v1/history.rs must use the identical
    -- expressions, or ClickHouse falls back to the raw rows.
    PROJECTION p_5s (
        SELECT
            identity,
            name,
            intDiv(toUnixTimestamp64Milli(timestamp), 5000),
            countIf(ifNull(isFinite(value), 0)),
            sumIf(assumeNotNull(value), ifNull(isFinite(value), 0)),
            minIf(assumeNotNull(value), ifNull(isFinite(value), 0)),
            maxIf(assumeNotNull(value), ifNull(isFinite(value), 0)),
            sumIf(toUnixTimestamp64Milli(timestamp), ifNull(isFinite(value), 0))
        GROUP BY identity, name, intDiv(toUnixTimestamp64Milli(timestamp), 5000)
    ),
    PROJECTION p_1m (
        SELECT
            identity,
            name,
            intDiv(toUnixTimestamp64Milli(timestamp), 60000),
            countIf(ifNull(isFinite(value), 0)),
            sumIf(assumeNotNull(value), ifNull(isFinite(value), 0)),
            minIf(assumeNotNull(value), ifNull(isFinite(value), 0)),
            maxIf(assumeNotNull(value), ifNull(isFinite(value), 0)),
            sumIf(toUnixTimestamp64Milli(timestamp), ifNull(isFinite(value), 0))
        GROUP BY identity, name, intDiv(toUnixTimestamp64Milli(timestamp), 60000)
    )
)

ENGINE = MergeTree

PARTITION BY toYYYYMMDD(timestamp)
ORDER BY (identity, name, timestamp)

-- expiry time for data (30 Days)
TTL timestamp + INTERVAL 30 DAY
