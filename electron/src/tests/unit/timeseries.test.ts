import { expect, test, describe } from "vitest";
import {
  createTimeSeries,
  trimSeries,
  MIN_RETENTION_MS,
  Series,
  TimeSeries,
} from "@/lib/timeseries";

function buildFullSeries(): Series {
  const { initialTimeSeries, insert } = createTimeSeries({
    sampleIntervalLong: 1000,
    retentionDurationLong: 60 * 60 * 1000, // 1h -> 3600 slots
  });

  let series = initialTimeSeries;
  const start = 1_000_000;
  // Fill the buffer well past capacity so it wraps at least once.
  for (let i = 0; i < 3700; i++) {
    series = insert(series, { value: i, timestamp: start + i * 1000 });
  }
  return series.long;
}

describe("trimSeries", () => {
  test("shrinks size and timeWindow by cutMs", () => {
    const long = buildFullSeries();
    const cutMs = 10 * 60 * 1000; // 10 min
    const trimmed = trimSeries(long, cutMs, 0);

    expect(trimmed.timeWindow).toBe(long.timeWindow - cutMs);
    expect(trimmed.size).toBe(Math.ceil(trimmed.timeWindow / trimmed.sampleInterval));
    expect(trimmed.values.length).toBe(trimmed.size);
  });

  test("preserves the most recent entries in chronological order", () => {
    const long = buildFullSeries();
    const cutMs = 10 * 60 * 1000;
    const trimmed = trimSeries(long, cutMs, 0);

    // Confirm every surviving entry is newer than the cutoff, and the series'
    // latest timestamp is unchanged.
    const cutoff = long.lastTimestamp - trimmed.timeWindow;
    for (const v of trimmed.values) {
      if (v && v.timestamp > 0) {
        expect(v.timestamp).toBeGreaterThanOrEqual(cutoff);
      }
    }
    expect(trimmed.lastTimestamp).toBe(long.lastTimestamp);
  });

  test("is idempotent once the floor is reached", () => {
    const long = buildFullSeries();
    const atFloor = trimSeries(long, long.timeWindow - MIN_RETENTION_MS);
    expect(atFloor.timeWindow).toBe(MIN_RETENTION_MS);

    const noop = trimSeries(atFloor, 10 * 60 * 1000);
    expect(noop).toBe(atFloor); // same reference, no-op
    expect(noop.timeWindow).toBe(MIN_RETENTION_MS);
  });

  test("respects the default MIN_RETENTION_MS floor", () => {
    const long = buildFullSeries();
    const trimmed = trimSeries(long, long.timeWindow); // try to cut everything
    expect(trimmed.timeWindow).toBe(MIN_RETENTION_MS);
  });

  test("cutting the oldest 10 min from a full 1h buffer keeps exactly 50 min", () => {
    const long = buildFullSeries(); // full 1h buffer, 3600 slots
    const cutMs = 10 * 60 * 1000; // matches memoryMonitor.ts's TRIM_CUT_MS
    const trimmed = trimSeries(long, cutMs); // uses the real default MIN_RETENTION_MS

    expect(long.timeWindow).toBe(60 * 60 * 1000);
    expect(trimmed.timeWindow).toBe(50 * 60 * 1000);
    expect(trimmed.size).toBe(50 * 60); // 50 min at 1 sample/sec
    expect(trimmed.validCount).toBe(trimmed.size); // buffer was overfull, so still full post-trim

    // Discarded: the oldest 10 min of data must not survive the trim.
    const cutoff = long.lastTimestamp - trimmed.timeWindow;
    const oldestSurviving = Math.min(
      ...trimmed.values.filter((v) => v && v.timestamp > 0).map((v) => v!.timestamp),
    );
    expect(oldestSurviving).toBeGreaterThanOrEqual(cutoff);

    // A second trim attempt at the floor is a no-op — 50 min stays 50 min.
    const secondTrim = trimSeries(trimmed, cutMs);
    expect(secondTrim.timeWindow).toBe(50 * 60 * 1000);
  });

  test("insert() on a trimmed series wraps correctly at the new smaller size", () => {
    const long = buildFullSeries();
    const trimmed = trimSeries(long, 10 * 60 * 1000, 0);
    const sizeBefore = trimmed.size;

    // Re-wrap the trimmed series into a full TimeSeries to use the shared insert().
    const { insert } = createTimeSeries();
    let ts: TimeSeries = { current: null, long: trimmed, short: trimmed };
    const nextTs = trimmed.lastTimestamp + trimmed.sampleInterval;
    ts = insert(ts, { value: 999, timestamp: nextTs });

    expect(ts.long.size).toBe(sizeBefore);
    expect(ts.long.lastTimestamp).toBe(nextTs);
    expect(ts.long.validCount).toBeLessThanOrEqual(sizeBefore);
  });
});
