/**
 * IndexedDB-backed TimeSeries implementation
 * 
 * This module provides a persistent TimeSeries implementation that:
 * 1. Keeps short timeseries (5s window) in RAM only for performance
 * 2. Persists long timeseries (1h+ window) to IndexedDB automatically
 * 3. Loads data from IndexedDB on initialization
 * 4. Maintains API compatibility with the original timeseries.ts
 * 
 * Usage is nearly identical to createTimeSeries() but with persistence:
 * 
 * ```ts
 * const { initialTimeSeries, insert } = createPersistentTimeSeries(
 *   "namespace-id",
 *   "seriesName"
 * );
 * ```
 */

import { produce } from "immer";
import {
  TimeSeries,
  TimeSeriesValue,
  TimeSeriesConfig,
  DEFAULT_TIMESERIES_CONFIG,
  Series,
} from "./timeseries";
import {
  storeDataPoint,
  queryDataPoints,
  deleteOldDataPoints,
  clearSeriesData,
} from "./timeseriesDB";

/**
 * Configuration specific to persistent timeseries
 */
export interface PersistentTimeSeriesConfig extends TimeSeriesConfig {
  /** Enable/disable persistence (useful for testing) */
  enablePersistence?: boolean;
  /** Maximum retention time for data in IndexedDB (default: 7 days) */
  maxRetentionTime?: number;
  /** How often to cleanup old data (default: 1 hour) */
  cleanupInterval?: number;
  /** 
   * Maximum window to load from IndexedDB on init (default: retentionDurationLong).
   * Set to 0 to skip historical loading entirely.
   */
  initialLoadWindowMs?: number;
  /**
   * When true, historical data loads lazily in the background AFTER 
   * the timeseries is already active with live data (default: true).
   * This ensures charts render immediately with live data.
   */
  lazyLoadHistory?: boolean;
}

/**
 * Default configuration for persistent timeseries
 * 
 * Long series buffer: 10 minutes at 20ms = 30,000 points (~240KB per series)
 * Max retention in DB: 7 days (~100MB per series)
 * Cleanup interval: 1 hour
 * Lazy load: enabled by default so charts render immediately with live data
 */
export const DEFAULT_PERSISTENT_CONFIG: Required<PersistentTimeSeriesConfig> = {
  ...DEFAULT_TIMESERIES_CONFIG,
  enablePersistence: true,
  maxRetentionTime: 7 * 24 * 60 * 60 * 1000, // 7 days
  cleanupInterval: 60 * 60 * 1000, // 1 hour
  initialLoadWindowMs: 10 * 60 * 1000, // load last 10 minutes of history
  lazyLoadHistory: true, // load history in background after live data starts
};

/**
 * Return type with insert function and initialization promise
 */
export interface PersistentTimeSeriesWithInsert {
  initialTimeSeries: TimeSeries;
  insert: (series: TimeSeries, valueObj: TimeSeriesValue) => TimeSeries;
  /** 
   * Promise that resolves when initial data is loaded from IndexedDB.
   * With lazyLoadHistory=true, this resolves immediately with empty series,
   * and onHistoryLoaded callback is fired later when history is ready.
   */
  ready: Promise<TimeSeries>;
  /** Callback fired when historical data finishes loading (for lazy loading) */
  onHistoryLoaded: (callback: (historicalSeries: TimeSeries) => void) => void;
  /** Manually trigger cleanup of old data */
  cleanup: () => Promise<void>;
}

/**
 * Load historical data from IndexedDB to populate the long series buffer
 * @param namespaceId Serialized namespace identifier
 * @param seriesName Name of the time series
 * @param retentionDuration How much history to load
 */
async function loadHistoricalData(
  namespaceId: string,
  seriesName: string,
  loadWindowMs: number,
): Promise<TimeSeriesValue[]> {
  const now = Date.now();
  const startTime = now - loadWindowMs;

  try {
    const rawData = await queryDataPoints(
      namespaceId,
      seriesName,
      startTime,
      now,
    );

    // Filter out invalid/corrupted entries defensively to avoid breaking charts
    const dataPoints = rawData.filter(
      (p) =>
        Number.isFinite(p.timestamp) &&
        Number.isFinite(p.value) &&
        p.timestamp > 0,
    );

    // If we filtered out everything but had records, proactively clear the series
    if (rawData.length > 0 && dataPoints.length === 0) {
      console.warn(
        `All historical points for ${namespaceId}:${seriesName} were invalid; clearing series in IndexedDB to recover`,
      );
      await clearSeriesData(namespaceId, seriesName).catch((err) =>
        console.error(`Failed to clear corrupted series ${namespaceId}:${seriesName}`, err),
      );
    }

    return dataPoints;
  } catch (error) {
    console.error(
      `Failed to load historical data for ${namespaceId}:${seriesName}`,
      error,
    );

    // Attempt automatic recovery by clearing the series to prevent UI breakage
    try {
      await clearSeriesData(namespaceId, seriesName);
      console.warn(
        `Cleared ${namespaceId}:${seriesName} after load failure; proceeding with empty history`,
      );
    } catch (clearErr) {
      console.error(
        `Failed to clear series ${namespaceId}:${seriesName} after load failure`,
        clearErr,
      );
    }

    return [];
  }
}

/**
 * Populate a Series buffer with historical data from IndexedDB
 * @param series The series to populate
 * @param dataPoints Historical data points
 * @returns Updated series with historical data
 */
function populateSeriesFromHistory(
  series: Series,
  dataPoints: TimeSeriesValue[],
): Series {
  if (dataPoints.length === 0) {
    return series;
  }

  // Sort by timestamp to ensure chronological order
  const sorted = [...dataPoints].sort((a, b) => a.timestamp - b.timestamp);

  // Filter by sample interval (downsample if needed)
  const sampled: TimeSeriesValue[] = [];
  let lastTimestamp = 0;

  for (const point of sorted) {
    if (point.timestamp - lastTimestamp >= series.sampleInterval) {
      sampled.push(point);
      lastTimestamp = point.timestamp;
    }
  }

  // Populate the circular buffer
  const newSeries = { ...series };
  let writeIndex = 0;

  for (const point of sampled.slice(-series.size)) {
    // Take only the most recent data that fits in the buffer
    newSeries.values[writeIndex] = point;
    writeIndex = (writeIndex + 1) % series.size;
  }

  newSeries.index = writeIndex;
  newSeries.validCount = Math.min(sampled.length, series.size);

  if (sampled.length > 0) {
    newSeries.lastTimestamp = sampled[sampled.length - 1].timestamp;
  }

  return newSeries;
}

/**
 * Create a persistent time series with IndexedDB backing
 * @param namespaceId Unique identifier for the namespace (must be serialized)
 * @param seriesName Name of the time series (e.g., "pullerSpeed")
 * @param config Optional configuration
 * @returns Object with initial timeseries, insert function, and ready promise
 */
export const createPersistentTimeSeries = (
  namespaceId: string,
  seriesName: string,
  config: Partial<PersistentTimeSeriesConfig> = {},
): PersistentTimeSeriesWithInsert => {
  const fullConfig = { ...DEFAULT_PERSISTENT_CONFIG, ...config };
  const {
    sampleIntervalShort,
    sampleIntervalLong,
    retentionDurationShort,
    retentionDurationLong,
    enablePersistence,
    maxRetentionTime,
    cleanupInterval,
    initialLoadWindowMs,
    lazyLoadHistory,
  } = fullConfig;

  const shortSize = Math.ceil(retentionDurationShort / sampleIntervalShort);
  const longSize = Math.ceil(retentionDurationLong / sampleIntervalLong);

  const emptyEntry: TimeSeriesValue = { value: 0, timestamp: 0 };

  // Create initial state with empty buffers
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

  // Track last cleanup time
  let lastCleanup = Date.now();

  /**
   * Cleanup old data from IndexedDB
   */
  const cleanup = async (): Promise<void> => {
    if (!enablePersistence) return;

    try {
      const cutoffTime = Date.now() - maxRetentionTime;
      await deleteOldDataPoints(namespaceId, seriesName, cutoffTime);
      lastCleanup = Date.now();
    } catch (error) {
      console.error(`Cleanup failed for ${namespaceId}:${seriesName}`, error);
    }
  };

  /**
   * Check if cleanup is needed and trigger if necessary
   */
  const maybeCleanup = async (): Promise<void> => {
    const timeSinceLastCleanup = Date.now() - lastCleanup;
    if (timeSinceLastCleanup >= cleanupInterval) {
      await cleanup();
    }
  };

  /**
   * Insert function that persists long series data to IndexedDB
   */
  const insert = (series: TimeSeries, value: TimeSeriesValue): TimeSeries => {
    const newSeries = produce(series, (draft) => {
      draft.current = value;

      // Insert into short buffer (RAM only)
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

      // Insert into long buffer (RAM + IndexedDB)
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

        // Persist to IndexedDB (async, fire-and-forget)
        if (enablePersistence) {
          storeDataPoint(namespaceId, seriesName, value).catch((error) => {
            console.error(
              `Failed to persist data point for ${namespaceId}:${seriesName}`,
              error,
            );
          });

          // Maybe trigger cleanup
          maybeCleanup().catch((error) => {
            console.error(
              `Cleanup check failed for ${namespaceId}:${seriesName}`,
              error,
            );
          });
        }
      }
    });

    return newSeries;
  };

  // History loaded callbacks for lazy loading
  const historyCallbacks: ((historicalSeries: TimeSeries) => void)[] = [];
  let historyLoadedSeries: TimeSeries | null = null;

  /**
   * Register a callback to be called when historical data is loaded (lazy mode)
   */
  const onHistoryLoaded = (callback: (historicalSeries: TimeSeries) => void): void => {
    // If history already loaded, call immediately
    if (historyLoadedSeries !== null) {
      callback(historyLoadedSeries);
    } else {
      historyCallbacks.push(callback);
    }
  };

  /**
   * Fire history loaded callbacks
   */
  const fireHistoryCallbacks = (series: TimeSeries): void => {
    historyLoadedSeries = series;
    for (const callback of historyCallbacks) {
      try {
        callback(series);
      } catch (error) {
        console.error(`History callback error for ${namespaceId}:${seriesName}`, error);
      }
    }
  };

  /**
   * Load historical data and populate series
   */
  const loadAndPopulateHistory = async (): Promise<TimeSeries> => {
    try {
      // Limit initial load window to avoid UI stalls when history is large
      const loadWindowMs = Math.min(
        retentionDurationLong,
        initialLoadWindowMs,
      );

      // Skip loading if window is 0
      if (loadWindowMs === 0) {
        return initialTimeSeries;
      }

      const historicalData = await loadHistoricalData(
        namespaceId,
        seriesName,
        loadWindowMs,
      );

      // Populate the long series buffer
      const populatedLong = populateSeriesFromHistory(
        initialTimeSeries.long,
        historicalData,
      );

      return {
        ...initialTimeSeries,
        long: populatedLong,
      };
    } catch (error) {
      console.error(
        `Failed to initialize persistent timeseries for ${namespaceId}:${seriesName}`,
        error,
      );
      return initialTimeSeries;
    }
  };

  // Determine ready behavior based on lazyLoadHistory setting
  let ready: Promise<TimeSeries>;

  if (!enablePersistence) {
    // Persistence disabled: resolve immediately with empty series
    ready = Promise.resolve(initialTimeSeries);
  } else if (lazyLoadHistory) {
    // Lazy loading: resolve immediately, load history in background
    ready = Promise.resolve(initialTimeSeries);
    
    // Start background loading after a small delay to let UI render first
    setTimeout(() => {
      loadAndPopulateHistory().then((loadedSeries) => {
        fireHistoryCallbacks(loadedSeries);
      });
    }, 100); // Small delay to ensure live data starts flowing first
  } else {
    // Blocking mode: wait for history before resolving
    ready = loadAndPopulateHistory();
  }

  return {
    initialTimeSeries,
    insert,
    ready,
    onHistoryLoaded,
    cleanup,
  };
};

/**
 * Helper hook to wait for timeseries to be ready with historical data
 * This can be used in React components to ensure data is loaded before rendering
 */
export async function waitForTimeSeriesReady(
  series: PersistentTimeSeriesWithInsert,
): Promise<TimeSeries> {
  return await series.ready;
}
