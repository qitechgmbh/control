/**
 * @file useTimeSeriesData.ts
 * @description React hook for reading timeseries data from IndexedDB for charts.
 * Replaces RAM-based timeseries with efficient IndexedDB queries.
 */

import { useEffect, useState, useCallback, useRef } from "react";
import {
  queryDataPoints,
  queryRecentDataPoints,
  TimeSeriesDataPoint,
} from "./indexedDBTimeseries";

/**
 * Configuration for the timeseries data hook
 */
export interface UseTimeSeriesDataConfig {
  /** Series key to query */
  seriesKey: string;
  /** Time window in milliseconds (e.g., 5000 for 5 seconds) */
  timeWindowMs?: number;
  /** Maximum number of recent points to load (alternative to time window) */
  maxPoints?: number;
  /** How often to refresh data (ms). Set to 0 to disable auto-refresh */
  refreshIntervalMs?: number;
  /** Whether to automatically load data on mount */
  autoLoad?: boolean;
}

/**
 * Return type for the hook
 */
export interface TimeSeriesData {
  /** Array of timestamps in chronological order */
  timestamps: number[];
  /** Array of values corresponding to timestamps */
  values: number[];
  /** Whether data is currently loading */
  isLoading: boolean;
  /** Latest data point */
  latest: { timestamp: number; value: number } | null;
  /** Number of data points */
  count: number;
  /** Min/max values in the dataset */
  range: { min: number; max: number } | null;
  /** Manually trigger a data reload */
  reload: () => Promise<void>;
}

/**
 * Hook to read timeseries data from IndexedDB
 */
export function useTimeSeriesData(
  config: UseTimeSeriesDataConfig,
): TimeSeriesData {
  const [data, setData] = useState<TimeSeriesDataPoint[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const refreshTimerRef = useRef<NodeJS.Timeout | null>(null);
  const isMountedRef = useRef(true);

  const {
    seriesKey,
    timeWindowMs,
    maxPoints,
    refreshIntervalMs = 100, // Default 100ms refresh
    autoLoad = true,
  } = config;

  /**
   * Load data from IndexedDB
   */
  const loadData = useCallback(async () => {
    setIsLoading(true);

    try {
      let results: TimeSeriesDataPoint[];

      if (maxPoints !== undefined) {
        // Load by point limit
        results = await queryRecentDataPoints(seriesKey, maxPoints);
      } else if (timeWindowMs !== undefined) {
        // Load by time window
        const now = Date.now();
        const startTime = now - timeWindowMs;
        results = await queryDataPoints(seriesKey, startTime, now);
      } else {
        // Load recent 1000 points by default
        results = await queryRecentDataPoints(seriesKey, 1000);
      }

      if (isMountedRef.current) {
        setData(results);
      }
    } catch (error) {
      console.error("Error loading timeseries data:", error);
    } finally {
      if (isMountedRef.current) {
        setIsLoading(false);
      }
    }
  }, [seriesKey, timeWindowMs, maxPoints]);

  /**
   * Set up auto-refresh
   */
  useEffect(() => {
    if (refreshIntervalMs > 0) {
      refreshTimerRef.current = setInterval(() => {
        loadData();
      }, refreshIntervalMs);

      return () => {
        if (refreshTimerRef.current) {
          clearInterval(refreshTimerRef.current);
        }
      };
    }
  }, [refreshIntervalMs, loadData]);

  /**
   * Initial load
   */
  useEffect(() => {
    if (autoLoad) {
      loadData();
    }

    return () => {
      isMountedRef.current = false;
    };
  }, [autoLoad, loadData]);

  /**
   * Process data into arrays
   */
  const timestamps = data.map((d) => d.timestamp);
  const values = data.map((d) => d.value);

  /**
   * Calculate latest
   */
  const latest =
    data.length > 0
      ? {
          timestamp: data[data.length - 1].timestamp,
          value: data[data.length - 1].value,
        }
      : null;

  /**
   * Calculate range
   */
  const range =
    values.length > 0
      ? {
          min: Math.min(...values),
          max: Math.max(...values),
        }
      : null;

  return {
    timestamps,
    values,
    isLoading,
    latest,
    count: data.length,
    range,
    reload: loadData,
  };
}

/**
 * Hook for multiple timeseries (e.g., multiple channels)
 */
export interface UseMultipleTimeSeriesConfig {
  /** Array of series configurations */
  series: Array<{
    key: string;
    seriesKey: string;
  }>;
  /** Time window in milliseconds */
  timeWindowMs?: number;
  /** Maximum number of points per series */
  maxPoints?: number;
  /** Refresh interval in milliseconds */
  refreshIntervalMs?: number;
}

/**
 * Return type for multiple series
 */
export interface MultipleTimeSeriesData {
  /** Map of series key to timeseries data */
  data: Map<string, { timestamps: number[]; values: number[] }>;
  /** Whether any series is loading */
  isLoading: boolean;
  /** Reload all series */
  reload: () => Promise<void>;
}

/**
 * Hook to read multiple timeseries efficiently
 */
export function useMultipleTimeSeries(
  config: UseMultipleTimeSeriesConfig,
): MultipleTimeSeriesData {
  const [seriesData, setSeriesData] = useState<
    Map<string, { timestamps: number[]; values: number[] }>
  >(new Map());
  const [isLoading, setIsLoading] = useState(false);
  const refreshTimerRef = useRef<NodeJS.Timeout | null>(null);
  const isMountedRef = useRef(true);

  const {
    series,
    timeWindowMs,
    maxPoints,
    refreshIntervalMs = 100,
  } = config;

  /**
   * Load all series data
   */
  const loadData = useCallback(async () => {
    setIsLoading(true);

    try {
      // Load all series in parallel
      const results = await Promise.all(
        series.map(async ({ key, seriesKey }) => {
          let points: TimeSeriesDataPoint[];

          if (maxPoints !== undefined) {
            points = await queryRecentDataPoints(seriesKey, maxPoints);
          } else if (timeWindowMs !== undefined) {
            const now = Date.now();
            const startTime = now - timeWindowMs;
            points = await queryDataPoints(seriesKey, startTime, now);
          } else {
            points = await queryRecentDataPoints(seriesKey, 1000);
          }

          return {
            key,
            timestamps: points.map((p) => p.timestamp),
            values: points.map((p) => p.value),
          };
        }),
      );

      if (isMountedRef.current) {
        const newMap = new Map(
          results.map((r) => [r.key, { timestamps: r.timestamps, values: r.values }]),
        );
        setSeriesData(newMap);
      }
    } catch (error) {
      console.error("Error loading multiple timeseries:", error);
    } finally {
      if (isMountedRef.current) {
        setIsLoading(false);
      }
    }
  }, [series, timeWindowMs, maxPoints]);

  /**
   * Set up auto-refresh
   */
  useEffect(() => {
    if (refreshIntervalMs > 0) {
      refreshTimerRef.current = setInterval(() => {
        loadData();
      }, refreshIntervalMs);

      return () => {
        if (refreshTimerRef.current) {
          clearInterval(refreshTimerRef.current);
        }
      };
    }
  }, [refreshIntervalMs, loadData]);

  /**
   * Initial load
   */
  useEffect(() => {
    loadData();

    return () => {
      isMountedRef.current = false;
    };
  }, [loadData]);

  return {
    data: seriesData,
    isLoading,
    reload: loadData,
  };
}
