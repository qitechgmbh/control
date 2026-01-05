/**
 * @file timeseriesHybrid.ts
 * @description Backward-compatible timeseries API that uses IndexedDB under the hood.
 * Provides the same interface as the old RAM-based system for easy migration.
 */

import { produce } from "immer";
import { useState, useEffect, useCallback } from "react";
import { storeDataPoint } from "./indexedDBTimeseries";
import { queryRecentDataPoints, TimeSeriesDataPoint } from "./indexedDBTimeseries";

/**
 * Interface for a single data point (same as before)
 */
export interface TimeSeriesValue {
  value: number;
  timestamp: number;
}

/**
 * Series configuration (simplified - IndexedDB handles storage)
 */
export type SeriesConfig = {
  /** Maximum points to keep in memory for immediate display */
  liveBufferSize: number;
  /** Time window for queries in milliseconds */
  timeWindow: number;
};

/**
 * Series type that maintains a small live buffer
 * Backward compatible with old timeseries API
 */
export type Series = {
  /** Small buffer of recent points for immediate display */
  liveBuffer: TimeSeriesValue[];
  /** Alias for liveBuffer (backward compatibility) */
  values: TimeSeriesValue[];
  /** Series key for IndexedDB queries */
  seriesKey: string;
  /** Configuration */
  config: SeriesConfig;
  /** Last timestamp seen */
  lastTimestamp: number;
  /** Index for circular buffer (compatibility) */
  index: number;
  /** Size of buffer (compatibility) */
  size: number;
  /** Valid count (compatibility) */
  validCount: number;
};

/**
 * Interface for the time series state (similar to before)
 */
export interface TimeSeries {
  current: TimeSeriesValue | null;
  short: Series;
  long: Series;
}

/**
 * Return type of createTimeSeries (same as before)
 */
export interface TimeSeriesWithInsert {
  initialTimeSeries: TimeSeries;
  insert: (series: TimeSeries, valueObj: TimeSeriesValue) => TimeSeries;
}

/**
 * Configuration object for creating a time series (simplified)
 */
export interface TimeSeriesConfig {
  /** Series key for IndexedDB storage */
  seriesKey: string;
  /** Live buffer size for immediate display (default: 250 points) */
  liveBufferSize?: number;
  /** Short time window in ms (default: 5s) */
  retentionDurationShort?: number;
  /** Long time window in ms (default: 1h) */
  retentionDurationLong?: number;
}

/**
 * Default configuration values
 */
export const DEFAULT_TIMESERIES_CONFIG = {
  liveBufferSize: 250,
  retentionDurationShort: 5000,
  retentionDurationLong: 60 * 60 * 1000,
};

/**
 * Extract data from series (combines IndexedDB + live buffer)
 */
export function extractDataFromSeries(
  series: Series,
  timeWindow?: number,
): [number[], number[]] {
  const cutoffTime = timeWindow
    ? series.lastTimestamp - timeWindow
    : series.lastTimestamp - series.config.timeWindow;

  // Filter live buffer
  const validPoints = series.liveBuffer.filter(
    (p) => p.timestamp >= cutoffTime && p.timestamp > 0,
  );

  const timestamps = validPoints.map((p) => p.timestamp);
  const values = validPoints.map((p) => p.value);

  return [timestamps, values];
}

/**
 * Get min/max from live buffer
 */
export function getSeriesMinMax(
  series: Series,
  timeWindow?: number,
): { min: number; max: number } {
  const [, values] = extractDataFromSeries(series, timeWindow);

  if (values.length === 0) {
    return { min: 0, max: 0 };
  }

  return {
    min: Math.min(...values),
    max: Math.max(...values),
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
  const latest =
    series.liveBuffer.length > 0
      ? series.liveBuffer[series.liveBuffer.length - 1]
      : null;

  let timeRange: { start: number; end: number } | null = null;
  if (series.liveBuffer.length > 0) {
    timeRange = {
      start: series.liveBuffer[0].timestamp,
      end: series.liveBuffer[series.liveBuffer.length - 1].timestamp,
    };
  }

  return {
    min,
    max,
    count: series.liveBuffer.length,
    latest,
    timeRange,
  };
}

/**
 * Convert series to uPlot-compatible data format (same as before)
 */
export function seriesToUPlotData(
  series: Series,
  timeWindow?: number,
): [number[], number[]] {
  return extractDataFromSeries(series, timeWindow);
}

/**
 * Factory function to create a new time series
 * Now uses IndexedDB but maintains the same API
 */
export const createTimeSeries = (
  config: TimeSeriesConfig,
): TimeSeriesWithInsert => {
  const {
    seriesKey,
    liveBufferSize = DEFAULT_TIMESERIES_CONFIG.liveBufferSize,
    retentionDurationShort = DEFAULT_TIMESERIES_CONFIG.retentionDurationShort,
    retentionDurationLong = DEFAULT_TIMESERIES_CONFIG.retentionDurationLong,
  } = config;

  const initialTimeSeries: TimeSeries = {
    current: null,
    short: {
      liveBuffer: [],
      values: [], // Backward compatibility alias
      seriesKey: `${seriesKey}:short`,
      config: {
        liveBufferSize,
        timeWindow: retentionDurationShort,
      },
      lastTimestamp: 0,
      index: 0,
      size: liveBufferSize,
      validCount: 0,
    },
    long: {
      liveBuffer: [],
      values: [], // Backward compatibility alias
      seriesKey: `${seriesKey}:long`,
      config: {
        liveBufferSize: Math.floor(liveBufferSize / 5), // Fewer points for long series
        timeWindow: retentionDurationLong,
      },
      lastTimestamp: 0,
      index: 0,
      size: Math.floor(liveBufferSize / 5),
      validCount: 0,
    },
  };

  const insert = (series: TimeSeries, value: TimeSeriesValue): TimeSeries => {
    // Write to IndexedDB asynchronously (fire and forget)
    storeDataPoint(series.short.seriesKey, value.timestamp, value.value).catch(
      (err) => console.error("Error storing datapoint:", err),
    );

    return produce(series, (draft) => {
      draft.current = value;

      // Update short series live buffer
      draft.short.liveBuffer.push(value);
      draft.short.values = draft.short.liveBuffer; // Keep alias in sync
      draft.short.lastTimestamp = value.timestamp;
      draft.short.validCount = draft.short.liveBuffer.length;

      // Trim buffer if too large
      if (draft.short.liveBuffer.length > draft.short.config.liveBufferSize) {
        draft.short.liveBuffer.shift();
        draft.short.values = draft.short.liveBuffer; // Keep alias in sync
        draft.short.index = (draft.short.index + 1) % draft.short.size;
      }

      // Update long series live buffer (less frequently)
      const shouldAddToLong =
        draft.long.liveBuffer.length === 0 ||
        value.timestamp - draft.long.lastTimestamp > 1000; // 1s interval

      if (shouldAddToLong) {
        draft.long.liveBuffer.push(value);
        draft.long.values = draft.long.liveBuffer; // Keep alias in sync
        draft.long.lastTimestamp = value.timestamp;
        draft.long.validCount = draft.long.liveBuffer.length;

        // Trim buffer if too large
        if (draft.long.liveBuffer.length > draft.long.config.liveBufferSize) {
          draft.long.liveBuffer.shift();
          draft.long.values = draft.long.liveBuffer; // Keep alias in sync
          draft.long.index = (draft.long.index + 1) % draft.long.size;
        }
      }
    });
  };

  return { initialTimeSeries, insert };
};

/**
 * Helper to get valid data count from series (backward compatibility)
 */
export function getValidDataCount(series: Series): number {
  return series.validCount;
}

/**
 * Helper to check if series is full (backward compatibility)
 */
export function isSeriesFull(series: Series): boolean {
  return series.validCount >= series.size;
}

/**
 * Reset series to empty state (backward compatibility)
 */
export function resetSeries(series: Series): Series {
  return {
    ...series,
    liveBuffer: [],
    values: [],
    index: 0,
    lastTimestamp: 0,
    validCount: 0,
  };
}

/**
 * Hook to use timeseries with automatic IndexedDB loading
 * Provides the same data structure as before but merges IndexedDB data
 */
export function useTimeSeriesWithHistory(
  seriesKey: string,
  liveData: TimeSeries | null,
  timeWindow?: "short" | "long",
): {
  timestamps: number[];
  values: number[];
  isLoading: boolean;
} {
  const [historicalData, setHistoricalData] = useState<TimeSeriesDataPoint[]>(
    [],
  );
  const [isLoading, setIsLoading] = useState(true);

  const series = timeWindow === "long" ? liveData?.long : liveData?.short;
  const actualSeriesKey = series?.seriesKey || `${seriesKey}:short`;

  // Load historical data from IndexedDB
  useEffect(() => {
    let mounted = true;

    const loadData = async () => {
      try {
        setIsLoading(true);
        const points = await queryRecentDataPoints(actualSeriesKey, 1000);
        if (mounted) {
          setHistoricalData(points);
        }
      } catch (err) {
        console.error("Error loading historical data:", err);
      } finally {
        if (mounted) {
          setIsLoading(false);
        }
      }
    };

    loadData();

    // Refresh periodically
    const interval = setInterval(loadData, 1000);

    return () => {
      mounted = false;
      clearInterval(interval);
    };
  }, [actualSeriesKey]);

  // Merge historical and live data
  const { timestamps, values } = useCallback(() => {
    if (!series) {
      return { timestamps: [], values: [] };
    }

    // Combine IndexedDB data with live buffer
    const allPoints = [...historicalData];

    // Add live buffer points that aren't in IndexedDB yet
    const lastHistoricalTimestamp =
      historicalData.length > 0
        ? historicalData[historicalData.length - 1].timestamp
        : 0;

    series.liveBuffer.forEach((point) => {
      if (point.timestamp > lastHistoricalTimestamp) {
        allPoints.push({
          id: "",
          seriesKey: series.seriesKey,
          timestamp: point.timestamp,
          value: point.value,
        });
      }
    });

    // Sort by timestamp
    allPoints.sort((a, b) => a.timestamp - b.timestamp);

    // Apply time window filter
    const cutoffTime = series.config.timeWindow
      ? series.lastTimestamp - series.config.timeWindow
      : 0;
    const filtered = allPoints.filter((p) => p.timestamp >= cutoffTime);

    return {
      timestamps: filtered.map((p) => p.timestamp),
      values: filtered.map((p) => p.value),
    };
  }, [series, historicalData])();

  return { timestamps, values, isLoading };
}
