-- Adds the history projections of init_machine_measurements.sql to a table created
-- before them. One statement per `;`, run only while the table lacks them (see main.rs).
-- New parts get the projections right away; existing parts are filled by the background
-- mutation that main.rs starts with MATERIALIZE PROJECTION.

ALTER TABLE control_hub.machine_measurements ADD PROJECTION IF NOT EXISTS p_5s (
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
    );

ALTER TABLE control_hub.machine_measurements ADD PROJECTION IF NOT EXISTS p_1m (
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
    );
