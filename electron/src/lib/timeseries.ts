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
 * Series with circular buffer, min/max deques, and path
 */
type Series = {
  values: (TimeSeriesValue | null)[];
  index: number;
  size: number;
  lastTimestamp: number;
  timeWindow: number;
  minDeque: number[];
  maxDeque: number[];
  path: [number, number][];
};

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

  const createEmptySeries = (size: number, window: number): Series => ({
    values: Array.from({ length: size }, () => ({ ...emptyEntry })),
    index: 0,
    size,
    lastTimestamp: 0,
    timeWindow: window,
    minDeque: [],
    maxDeque: [],
    path: [],
  });

  const initialTimeSeries: TimeSeries = {
    current: null,
    short: createEmptySeries(shortSize, retentionDurationShort),
    long: createEmptySeries(longSize, retentionDurationLong),
  };

  const updateDeques = (
    series: Series,
    value: TimeSeriesValue,
    index: number
  ) => {
    // Maintain minDeque (increasing)
    while (
      series.minDeque.length &&
      series.values[series.minDeque[series.minDeque.length - 1]]!.value > value.value
    ) {
      series.minDeque.pop();
    }
    series.minDeque.push(index);

    // Maintain maxDeque (decreasing)
    while (
      series.maxDeque.length &&
      series.values[series.maxDeque[series.maxDeque.length - 1]]!.value < value.value
    ) {
      series.maxDeque.pop();
    }
    series.maxDeque.push(index);

    // Remove outdated from the front
    const cutoff = value.timestamp - series.timeWindow;
    while (
      series.minDeque.length &&
      series.values[series.minDeque[0]]!.timestamp < cutoff
    ) {
      series.minDeque.shift();
    }
    while (
      series.maxDeque.length &&
      series.values[series.maxDeque[0]]!.timestamp < cutoff
    ) {
      series.maxDeque.shift();
    }
  };

  const updatePath = (series: Series, value: TimeSeriesValue) => {
    series.path.push([value.timestamp, value.value]);
    const cutoff = value.timestamp - series.timeWindow;
    while (
      series.path.length &&
      series.path[0][0] < cutoff
    ) {
      series.path.shift(); // remove oldest
    }
  };

  const insert = (series: TimeSeries, value: TimeSeriesValue): TimeSeries => {
    return produce(series, (draft) => {
      draft.current = value;

      const insertIntoSeries = (s: Series) => {
        s.values[s.index] = value;
        updateDeques(s, value, s.index);
        updatePath(s, value);
        s.index = (s.index + 1) % s.size;
        s.lastTimestamp = value.timestamp;
      };

      insertIntoSeries(draft.short);
      insertIntoSeries(draft.long);
    });
  };

  return { initialTimeSeries, insert };
};
