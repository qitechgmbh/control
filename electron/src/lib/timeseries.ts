import { produce } from "immer";

/**
 * Interface for a single data point
 */
export interface TimeSeriesValue {
  value: number;
  timestamp: number;
}

/**
 * Min/Max tracker using deques for O(1) operations
 */
class MinMaxTracker {
  private minDeque: { value: number; index: number }[] = [];
  private maxDeque: { value: number; index: number }[] = [];
  private globalIndex = 0;

  push(value: number): void {
    // Remove elements from back while current value is smaller (for min)
    while (this.minDeque.length && this.minDeque[this.minDeque.length - 1].value >= value) {
      this.minDeque.pop();
    }
    this.minDeque.push({ value, index: this.globalIndex });

    // Remove elements from back while current value is larger (for max)
    while (this.maxDeque.length && this.maxDeque[this.maxDeque.length - 1].value <= value) {
      this.maxDeque.pop();
    }
    this.maxDeque.push({ value, index: this.globalIndex });

    this.globalIndex++;
  }

  removeOldest(): void {
    const oldestIndex = this.globalIndex - this.getValidSize();

    // Remove elements from front that are too old
    while (this.minDeque.length && this.minDeque[0].index <= oldestIndex) {
      this.minDeque.shift();
    }
    while (this.maxDeque.length && this.maxDeque[0].index <= oldestIndex) {
      this.maxDeque.shift();
    }
  }

  getMin(): number {
    return this.minDeque.length ? this.minDeque[0].value : 0;
  }

  getMax(): number {
    return this.maxDeque.length ? this.maxDeque[0].value : 0;
  }

  private getValidSize(): number {
    // This should be set by the parent Series
    return Math.max(this.minDeque.length, this.maxDeque.length);
  }

  reset(): void {
    this.minDeque = [];
    this.maxDeque = [];
    this.globalIndex = 0;
  }
}

/**
 * Enhanced Series type with min/max tracking
 */
type Series = {
  values: (TimeSeriesValue | null)[];
  index: number;
  size: number;
  lastTimestamp: number;
  timeWindow: number;
  sampleInterval: number;
  minMaxTracker: MinMaxTracker;
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
 * Extract data with time window filtering
 */
export function extractDataFromSeries(series: Series, timeWindow?: number): [number[], number[]] {
  const timestamps: number[] = [];
  const values: number[] = [];
  const cutoffTime = timeWindow ? series.lastTimestamp - timeWindow : 0;

  const { values: raw, index, size } = series;
  for (let i = 0; i < size; i++) {
    const idx = (index + i) % size;
    const val = raw[idx];
    if (val && val.timestamp > 0 && val.timestamp >= cutoffTime) {
      timestamps.push(val.timestamp);
      values.push(val.value);
    }
  }

  return [timestamps, values];
}

/**
 * Get min/max values from series in O(1) time
 */
export function getSeriesMinMax(series: Series): { min: number; max: number } {
  return {
    min: series.minMaxTracker.getMin(),
    max: series.minMaxTracker.getMax()
  };
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
  const latest = series.validCount > 0 ? series.values[(series.index - 1 + series.size) % series.size] : null;

  // Find time range
  let timeRange: { start: number; end: number } | null = null;
  if (series.validCount > 0) {
    const [timestamps] = extractDataFromSeries(series);
    if (timestamps.length > 0) {
      timeRange = {
        start: timestamps[0],
        end: timestamps[timestamps.length - 1]
      };
    }
  }

  return {
    min,
    max,
    count: series.validCount,
    latest,
    timeRange
  };
}

/**
 * Convert series to uPlot-compatible data format
 */
export function seriesToUPlotData(series: Series, timeWindow?: number): [number[], number[]] {
  return extractDataFromSeries(series, timeWindow);
}

/**
 * Factory function to create a new time series with circular buffers and min/max tracking
 */
export const createTimeSeries = (
  sampleIntervalShort: number,
  sampleIntervalLong: number,
  retentionDurationShort: number,
  retentionDurationLong: number
): TimeSeriesWithInsert => {
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
      minMaxTracker: new MinMaxTracker(),
      validCount: 0,
    },
    long: {
      values: Array.from({ length: longSize }, () => ({ ...emptyEntry })),
      index: 0,
      size: longSize,
      lastTimestamp: 0,
      timeWindow: retentionDurationLong,
      sampleInterval: sampleIntervalLong,
      minMaxTracker: new MinMaxTracker(),
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

        if (isShortOverwriting) {
          draft.short.minMaxTracker.removeOldest();
        } else {
          draft.short.validCount++;
        }
        draft.short.minMaxTracker.push(value.value);
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

        if (isLongOverwriting) {
          draft.long.minMaxTracker.removeOldest();
        } else {
          draft.long.validCount++;
        }
        draft.long.minMaxTracker.push(value.value);
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
  series.minMaxTracker.reset();
}
