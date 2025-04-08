import { produce } from "immer";

/**
 * Interface for the current value (kept as an object for API compatibility)
 */
export interface TimeSeriesValue {
  value: number;
  timestamp: number;
}

/**
 * Interface for the store state using TypedArrays
 */
export interface TimeSeries {
  current: TimeSeriesValue | null;
  seriesValues: Float32Array; // Array of time series values
  seriesTimestamps: Float32Array; // Array of corresponding timestamps
  writeIndex: number; // Current write position in the circular buffer
  filledCount: number; // Number of valid entries in the buffer
  lastBucketTimestamp?: number; // Track the last bucket timestamp
}

/**
 * Return type of createTimeSeries
 */
export interface TimeSeriesWithInsert {
  initialTimeSeries: TimeSeries;
  insert: (series: TimeSeries, valueObj: TimeSeriesValue) => TimeSeries;
}

/**
 * Factory function to create a time series data structure with an immutable insert function
 *
 * @param {number} sampleInterval - Interval in ms to sample values
 * @param {number} retentionDuration - How long to keep values in ms
 * @returns {TimeSeriesWithInsert} - Object containing initial TimeSeries and insert function
 */
export const createTimeSeries = (
  sampleInterval: number,
  retentionDuration: number,
): TimeSeriesWithInsert => {
  // Calculate exact buffer size needed
  const bufferSize: number = Math.ceil(retentionDuration / sampleInterval);

  // Create initial TimeSeries with pre-allocated TypedArrays
  const initialTimeSeries: TimeSeries = {
    current: null,
    seriesValues: new Float32Array(bufferSize),
    seriesTimestamps: new Float32Array(bufferSize),
    writeIndex: 0,
    filledCount: 0,
    lastBucketTimestamp: -1,
  };

  /**
   * Insert a value into the time series, returning a new TimeSeries object
   * Uses Immer's produce for immutability
   *
   * @param {TimeSeries} series - The current time series state
   * @param {Object} valueObj - Object containing value and timestamp
   * @returns {TimeSeries} - New time series with the inserted value
   */
  const insert = (series: TimeSeries, value: TimeSeriesValue): TimeSeries => {
    return produce(series, (draft) => {
      // Update current value unconditionally
      draft.current = value;

      // Calculate the bucket for this timestamp
      const timeseriesBucket: number =
        Math.floor(value.timestamp / sampleInterval) * sampleInterval;

      // Check if we already have an entry for this bucket
      if (draft.lastBucketTimestamp === timeseriesBucket) {
        // Same bucket, no need to update the time series arrays
        return;
      }

      // New bucket - update the TypedArrays
      draft.seriesTimestamps[draft.writeIndex] = timeseriesBucket;
      draft.seriesValues[draft.writeIndex] = value.value;

      // Update tracking variables
      draft.lastBucketTimestamp = timeseriesBucket;

      // Update metadata
      if (draft.filledCount < bufferSize) {
        draft.filledCount++;
      }

      // Advance the write pointer for next time
      draft.writeIndex = (draft.writeIndex + 1) % bufferSize;
    });
  };

  return {
    initialTimeSeries,
    insert,
  };
};
