-- Brings a machine_measurements table created before the codecs to the layout of
-- init_machine_measurements.sql. One statement per `;`, run only while the table still
-- has the old definition (see main.rs). Codec changes are metadata only: new parts use
-- them right away, existing parts are recompressed when merged or dropped by the TTL.

-- The by_time projection stored every row a second time; no query reads it.
ALTER TABLE control_hub.machine_measurements DROP PROJECTION IF EXISTS by_time;

ALTER TABLE control_hub.machine_measurements
    MODIFY COLUMN timestamp DateTime64(3, 'UTC') CODEC(DoubleDelta, ZSTD(1));

ALTER TABLE control_hub.machine_measurements
    MODIFY COLUMN value Nullable(Float64) CODEC(ZSTD(3));
