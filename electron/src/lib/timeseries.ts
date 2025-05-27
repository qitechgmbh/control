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
  short: SeriesMeta;
  long: SeriesMeta;
}

/**
 * Return type of createTimeSeries
 */
export interface TimeSeriesWithInsert {
  initialTimeSeries: TimeSeries;
  insert: (series: TimeSeries, valueObj: TimeSeriesValue) => TimeSeries;
}

/**
 * Factory function to create a new time series without value storage
 */
export const createTimeSeries = (
  sampleIntervalShort: number,
  sampleIntervalLong: number,
  retentionDurationShort: number,
  retentionDurationLong: number
): TimeSeriesWithInsert => {

  const initialTimeSeries: TimeSeries = {
    current: null,
    short: {
      lastTimestamp: 0,
      timeWindow: retentionDurationShort,
    },
    long: {
      lastTimestamp: 0,
      timeWindow: retentionDurationLong,
    },
  };

  const insert = (series: TimeSeries, value: TimeSeriesValue): TimeSeries => {
    return produce(series, (draft) => {
      draft.current = value;

      // Just update metadata (no value storage)
      draft.short.lastTimestamp = value.timestamp;

      draft.long.lastTimestamp = value.timestamp;
    });
  };

  return { initialTimeSeries, insert };
};

type SeriesMeta = {
  lastTimestamp: number;
  timeWindow: number;
};
