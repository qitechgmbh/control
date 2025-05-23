import { produce } from "immer";

/**
 * Interface for a single data point
 */
export interface TimeSeriesValue {
  value: number;
  timestamp: number;
}
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
 * Factory function to create a new time series with circular buffers
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
    },
    long: {
      values: Array.from({ length: longSize }, () => ({ ...emptyEntry })),
      index: 0,
      size: longSize,
      lastTimestamp: 0,
      timeWindow: retentionDurationLong,
    },
  };

  const insert = (series: TimeSeries, value: TimeSeriesValue): TimeSeries => {
    return produce(series, (draft) => {
      draft.current = value;

      // Insert into short buffer
      draft.short.values[draft.short.index] = value;
      draft.short.index = (draft.short.index + 1) % draft.short.size;
      draft.short.lastTimestamp = value.timestamp;

      // Insert into long buffer
      draft.long.values[draft.long.index] = value;
      draft.long.index = (draft.long.index + 1) % draft.long.size;
      draft.long.lastTimestamp = value.timestamp;
    });
  };

  return { initialTimeSeries, insert };
};

type Series = {
  values: (TimeSeriesValue | null)[];
  index: number;
  size: number;
  lastTimestamp: number;
  timeWindow: number;
};


