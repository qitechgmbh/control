import { produce } from "immer";

/**
 * Interface for a single data point
 */
export interface TimeSeriesValue {
  value: number;
  timestamp: number;
}

/**
 * Enhanced Series type without min/max tracking (we'll calculate dynamically)
 */
export type Series = {
  values: (TimeSeriesValue | null)[];
  index: number;
  size: number;
  lastTimestamp: number;
  timeWindow: number;
  sampleInterval: number;
  validCount: number; // Track how many valid entries we have
};

/**
 * Interface for the time series state
 */
export interface TimeSeries {
  current: TimeSeriesValue | null;
  long: Series;
  short: Series;
}

/**
 * Return type of createTimeSeries
 */
export interface TimeSeriesWithInsert {
  initialTimeSeries: TimeSeries;
  insert: (series: TimeSeries, valueObj: TimeSeriesValue) => TimeSeries;
}

/**
 * Configuration object for creating a time series
 */
export interface TimeSeriesConfig {
  sampleIntervalShort: number;
  sampleIntervalLong: number;
  retentionDurationShort: number;
  retentionDurationLong: number;
}

/**
 * Default configuration values
 * Short: 20ms sample, 5s retention
 * Long: 1s sample, 1h retention
 */
export const DEFAULT_TIMESERIES_CONFIG: TimeSeriesConfig = {
  sampleIntervalShort: 20, // 20ms
  sampleIntervalLong: 1000, // 1s
  retentionDurationShort: 5000, // 5s
  retentionDurationLong: 60 * 60 * 1000, // 1h
};

/**
 * Extract data with time window filtering in chronological order
 */
export function extractDataFromSeries(
  series: Series,
  timeWindow?: number,
): [number[], number[]] {
  const timestamps: number[] = [];
  const values: number[] = [];
  const cutoffTime = timeWindow ? series.lastTimestamp - timeWindow : 0;
  const { values: raw, index, size, validCount } = series;
  if (validCount === 0) {
    return [timestamps, values];
  }

  let startIdx: number;

  if (validCount < size) {
    startIdx = 0;
  } else {
    startIdx = index;
  }

  // Collect valid entries in chronological order
  for (let i = 0; i < validCount; i++) {
    const idx = (startIdx + i) % size;
    const val = raw[idx];

    if (val && val.timestamp > 0 && val.timestamp >= cutoffTime) {
      timestamps.push(val.timestamp);
      values.push(val.value);
    }
  }

  return [timestamps, values];
}

/**
 * Get min/max values from series by dynamically scanning the data
 */
export function getSeriesMinMax(
  series: Series,
  timeWindow?: number,
): { min: number; max: number } {
  const cutoffTime = timeWindow ? series.lastTimestamp - timeWindow : 0;

  let min = Number.POSITIVE_INFINITY;
  let max = Number.NEGATIVE_INFINITY;
  let hasValidData = false;

  const { values: raw, index, size, validCount } = series;

  if (validCount === 0) {
    return { min: 0, max: 0 };
  }

  let startIdx: number;

  if (validCount < size) {
    startIdx = 0;
  } else {
    startIdx = index;
  }

  for (let i = 0; i < validCount; i++) {
    const idx = (startIdx + i) % size;
    const val = raw[idx];

    if (val && val.timestamp > 0 && val.timestamp >= cutoffTime) {
      hasValidData = true;
      if (val.value < min) min = val.value;
      if (val.value > max) max = val.value;
    }
  }

  // Return sensible defaults if no valid data
  if (!hasValidData) {
    return { min: 0, max: 0 };
  }

  return { min, max };
}

/**
 * Get series statistics
 */
export function getSeriesStats(series: Series): {
  min: number;
  max: number;
  count: number;
  latest: TimeSeriesValue | null;
  timeRange: { start: number; end: number } | null;
} {
  const { min, max } = getSeriesMinMax(series);

  // Get the latest value (most recently inserted)
  let latest: TimeSeriesValue | null = null;
  if (series.validCount > 0) {
    // The latest value is at (index - 1) position
    const latestIdx = (series.index - 1 + series.size) % series.size;
    const latestVal = series.values[latestIdx];
    if (latestVal && latestVal.timestamp > 0) {
      latest = latestVal;
    }
  }

  // Find time range
  let timeRange: { start: number; end: number } | null = null;
  if (series.validCount > 0) {
    const [timestamps] = extractDataFromSeries(series);
    if (timestamps.length > 0) {
      timeRange = {
        start: timestamps[0],
        end: timestamps[timestamps.length - 1],
      };
    }
  }

  return {
    min,
    max,
    count: series.validCount,
    latest,
    timeRange,
  };
}

/**
 * Convert series to uPlot-compatible data format
 */
export function seriesToUPlotData(
  series: Series,
  timeWindow?: number,
): [number[], number[]] {
  return extractDataFromSeries(series, timeWindow);
}

/**
 * Factory function to create a new time series with circular buffers
 * @param config Optional configuration object. If omitted or partial, defaults from DEFAULT_TIMESERIES_CONFIG are used.
 */
export const createTimeSeries = (
  config: Partial<TimeSeriesConfig> = {},
): TimeSeriesWithInsert => {
  const {
    sampleIntervalShort,
    sampleIntervalLong,
    retentionDurationShort,
    retentionDurationLong,
  } = { ...DEFAULT_TIMESERIES_CONFIG, ...config };

  const shortSize = Math.ceil(retentionDurationShort / sampleIntervalShort);
  const longSize = Math.ceil(retentionDurationLong / sampleIntervalLong);

  const emptyEntry: TimeSeriesValue = { value: 0, timestamp: 0 };

  const initialTimeSeries: TimeSeries = {
    current: null,
    short: {
      values: Array.from({ length: shortSize }, () => ({ ...emptyEntry })),
      index: 0,
      size: shortSize,
      lastTimestamp: 0,
      timeWindow: retentionDurationShort,
      sampleInterval: sampleIntervalShort,
      validCount: 0,
    },
    long: {
      values: Array.from({ length: longSize }, () => ({ ...emptyEntry })),
      index: 0,
      size: longSize,
      lastTimestamp: 0,
      timeWindow: retentionDurationLong,
      sampleInterval: sampleIntervalLong,
      validCount: 0,
    },
  };

  const insert = (series: TimeSeries, value: TimeSeriesValue): TimeSeries => {
    return produce(series, (draft) => {
      draft.current = value;

      // Insert into short buffer only if enough time has passed (downsampling)
      const shortSampleInterval = draft.short.sampleInterval;
      const timeSinceLastShort = value.timestamp - draft.short.lastTimestamp;

      if (timeSinceLastShort >= shortSampleInterval) {
        const shortOldValue = draft.short.values[draft.short.index];
        const isShortOverwriting = shortOldValue && shortOldValue.timestamp > 0;

        draft.short.values[draft.short.index] = value;
        draft.short.index = (draft.short.index + 1) % draft.short.size;
        draft.short.lastTimestamp = value.timestamp;

        if (!isShortOverwriting) {
          draft.short.validCount++;
        }
      }

      // Insert into long buffer only if enough time has passed (downsampling)
      const longSampleInterval = draft.long.sampleInterval;
      const timeSinceLastLong = value.timestamp - draft.long.lastTimestamp;

      if (timeSinceLastLong >= longSampleInterval) {
        const longOldValue = draft.long.values[draft.long.index];
        const isLongOverwriting = longOldValue && longOldValue.timestamp > 0;

        draft.long.values[draft.long.index] = value;
        draft.long.index = (draft.long.index + 1) % draft.long.size;
        draft.long.lastTimestamp = value.timestamp;

        if (!isLongOverwriting) {
          draft.long.validCount++;
        }
      }
    });
  };

  return { initialTimeSeries, insert };
};

/**
 * Helper to get valid data count from series
 */
export function getValidDataCount(series: Series): number {
  return series.validCount;
}

/**
 * Helper to check if series is full
 */
export function isSeriesFull(series: Series): boolean {
  return series.validCount >= series.size;
}

/**
 * Reset series to empty state
 */
export function resetSeries(series: Series): void {
  series.values.fill({ value: 0, timestamp: 0 });
  series.index = 0;
  series.lastTimestamp = 0;
  series.validCount = 0;
}

/** Retention window never shrinks below this, so repeated trims can't erase a series. */
export const MIN_RETENTION_MS = 50 * 60 * 1000; // 50 min

/**
 * Shrinks a series' retention window by cutMs (down to minRetentionMs), reallocating
 * a smaller `values` array so the discarded entries are actually freed on next GC
 * rather than just being ignored by readers.
 */
export function trimSeries(
  series: Series,
  cutMs: number,
  minRetentionMs: number = MIN_RETENTION_MS,
): Series {
  const newTimeWindow = Math.max(series.timeWindow - cutMs, minRetentionMs);
  if (newTimeWindow >= series.timeWindow) return series; // already at floor

  const [timestamps, values] = extractDataFromSeries(series, newTimeWindow);
  const newSize = Math.max(1, Math.ceil(newTimeWindow / series.sampleInterval));

  const newValues: (TimeSeriesValue | null)[] = Array.from(
    { length: newSize },
    () => ({ value: 0, timestamp: 0 }),
  );
  const count = Math.min(timestamps.length, newSize);
  for (let i = 0; i < count; i++) {
    newValues[i] = { value: values[i], timestamp: timestamps[i] };
  }

  return {
    ...series,
    values: newValues,
    size: newSize,
    index: count % newSize,
    validCount: count,
    timeWindow: newTimeWindow,
  };
}

/**
 * Trims the `long` series of every TimeSeries-shaped field on a namespace store's
 * state object. Duck-types fields rather than requiring per-machine registration,
 * so it works generically across every machine's store.
 */
function isTimeSeries(value: unknown): value is TimeSeries {
  return (
    typeof value === "object" &&
    value !== null &&
    "current" in value &&
    "long" in value &&
    "short" in value
  );
}

export function trimTimeSeriesFields<S extends Record<string, unknown>>(
  state: S,
  cutMs: number,
): Partial<S> {
  const patch: Partial<S> = {};
  for (const [key, value] of Object.entries(state)) {
    if (isTimeSeries(value)) {
      (patch as Record<string, unknown>)[key] = {
        ...value,
        long: trimSeries(value.long, cutMs),
      };
    }
  }
  return patch;
}

/**
 * Aligns target series values with a given set of timestamps.
 * For each timestamp in dataTimestamps, finds the most recent target value
 * that was set before or at that timestamp (step function interpolation).
 */
export function alignTargetSeriesToTimestamps(
  targetSeries: TimeSeries,
  dataTimestamps: number[],
  fallbackValue: number,
): number[] {
  if (dataTimestamps.length === 0) {
    return [];
  }

  const [targetTimestamps, targetValues] = extractDataFromSeries(
    targetSeries.long,
  );

  if (targetTimestamps.length === 0) {
    return new Array(dataTimestamps.length).fill(fallbackValue);
  }

  const result: number[] = [];
  let targetIndex = 0;

  for (const dataTs of dataTimestamps) {
    while (
      targetIndex < targetTimestamps.length - 1 &&
      targetTimestamps[targetIndex + 1] <= dataTs
    ) {
      targetIndex++;
    }

    if (targetTimestamps[targetIndex] > dataTs) {
      result.push(fallbackValue);
    } else {
      result.push(targetValues[targetIndex]);
    }
  }

  return result;
}
