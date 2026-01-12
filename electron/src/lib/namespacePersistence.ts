/**
 * Namespace integration helpers for persistent timeseries
 * 
 * This module provides utilities to easily integrate IndexedDB-backed
 * persistent timeseries into Socket.io namespace implementations.
 * 
 * The key challenge is that we need to:
 * 1. Initialize timeseries with historical data from IndexedDB
 * 2. Maintain compatibility with existing namespace hooks
 * 3. Handle async initialization gracefully
 * 
 * Usage in a namespace:
 * 
 * ```ts
 * import { createNamespacePersistentTimeSeries } from '@/lib/namespacePersistence';
 * 
 * const seriesFactory = createNamespacePersistentTimeSeries(
 *   (namespaceId) => ({ 
 *     pullerSpeed: 'pullerSpeed',
 *     spoolRpm: 'spoolRpm',
 *     // ... other series
 *   })
 * );
 * 
 * export const createWinder2NamespaceStore = (): StoreApi<Winder2NamespaceStore> => {
 *   const series = seriesFactory.createInitialSeries(namespaceId);
 *   
 *   return create<Winder2NamespaceStore>(() => ({
 *     state: null,
 *     defaultState: null,
 *     ...series.initialState,
 *   }));
 * };
 * 
 * // Later, load historical data when the namespace is mounted
 * useEffect(() => {
 *   series.loadHistoricalData().then(historicalSeries => {
 *     store.setState(historicalSeries);
 *   });
 * }, []);
 * ```
 */

import { serializeNamespaceId, NamespaceId } from "@/client/socketioStore";
import { TimeSeries, TimeSeriesValue } from "./timeseries";
import {
  createPersistentTimeSeries,
  PersistentTimeSeriesWithInsert,
  PersistentTimeSeriesConfig,
} from "./timeseriesPersistent";

/**
 * Configuration for a series within a namespace
 */
export interface SeriesDefinition {
  /** Name of the series (used as key in store and DB) */
  name: string;
  /** Optional configuration override */
  config?: Partial<PersistentTimeSeriesConfig>;
}

/**
 * Type helper for series definitions map
 */
export type SeriesDefinitionMap = Record<string, string | SeriesDefinition>;

/**
 * Result of creating namespace series
 */
export interface NamespaceSeriesResult<T extends SeriesDefinitionMap> {
  /** Initial timeseries state for the store */
  initialState: Record<keyof T, TimeSeries>;
  /** Insert functions for each series */
  insertFns: Record<keyof T, (series: TimeSeries, value: TimeSeriesValue) => TimeSeries>;
  /** Promise that resolves when all series are loaded with historical data */
  ready: Promise<Record<keyof T, TimeSeries>>;
  /** Cleanup function to remove old data */
  cleanup: () => Promise<void>;
}

/**
 * Create persistent timeseries for a namespace with type safety
 * 
 * @param namespaceId The namespace ID (will be serialized)
 * @param seriesDefinitions Map of series names to definitions
 * @param globalConfig Optional global configuration for all series
 * @returns Object with initial state, insert functions, and ready promise
 */
export function createNamespacePersistentTimeSeries<T extends SeriesDefinitionMap>(
  namespaceId: NamespaceId,
  seriesDefinitions: T,
  globalConfig?: Partial<PersistentTimeSeriesConfig>,
): NamespaceSeriesResult<T> {
  const serializedNamespaceId = serializeNamespaceId(namespaceId);

  const initialState: Record<string, TimeSeries> = {};
  const insertFns: Record<string, (series: TimeSeries, value: TimeSeriesValue) => TimeSeries> = {};
  const readyPromises: Promise<[string, TimeSeries]>[] = [];
  const cleanupFns: (() => Promise<void>)[] = [];

  // Create persistent timeseries for each definition
  for (const [key, definition] of Object.entries(seriesDefinitions)) {
    const seriesName = typeof definition === "string" ? definition : definition.name;
    const config = typeof definition === "string" ? globalConfig : { ...globalConfig, ...definition.config };

    const persistentSeries = createPersistentTimeSeries(
      serializedNamespaceId,
      seriesName,
      config,
    );

    initialState[key] = persistentSeries.initialTimeSeries;
    insertFns[key] = persistentSeries.insert;
    cleanupFns.push(persistentSeries.cleanup);

    // Add to ready promises
    readyPromises.push(
      persistentSeries.ready.then((loadedSeries) => [key, loadedSeries] as [string, TimeSeries]),
    );
  }

  // Combine all ready promises
  const ready = Promise.all(readyPromises).then((results) => {
    const loadedState: Record<string, TimeSeries> = {};
    for (const [key, series] of results) {
      loadedState[key] = series;
    }
    return loadedState as Record<keyof T, TimeSeries>;
  });

  // Combined cleanup function
  const cleanup = async () => {
    await Promise.all(cleanupFns.map((fn) => fn()));
  };

  return {
    initialState: initialState as Record<keyof T, TimeSeries>,
    insertFns: insertFns as Record<keyof T, (series: TimeSeries, value: TimeSeriesValue) => TimeSeries>,
    ready,
    cleanup,
  };
}

/**
 * Factory pattern for creating series across multiple namespace instances
 * This is useful when you have multiple machine instances using the same namespace structure
 */
export class NamespaceSeriesFactory<T extends SeriesDefinitionMap> {
  private seriesDefinitions: T;
  private globalConfig?: Partial<PersistentTimeSeriesConfig>;

  constructor(
    seriesDefinitions: T,
    globalConfig?: Partial<PersistentTimeSeriesConfig>,
  ) {
    this.seriesDefinitions = seriesDefinitions;
    this.globalConfig = globalConfig;
  }

  /**
   * Create series for a specific namespace instance
   */
  create(namespaceId: NamespaceId): NamespaceSeriesResult<T> {
    return createNamespacePersistentTimeSeries(
      namespaceId,
      this.seriesDefinitions,
      this.globalConfig,
    );
  }
}

/**
 * Helper to create insert functions that are bound to a specific series instance
 * This is useful in the old createTimeSeries pattern where insert functions are created once
 * 
 * @deprecated This is for backward compatibility. New code should use createNamespacePersistentTimeSeries
 */
export function createCompatibleInsertFunctions<T extends SeriesDefinitionMap>(
  namespaceId: NamespaceId,
  seriesDefinitions: T,
  globalConfig?: Partial<PersistentTimeSeriesConfig>,
): {
  initialSeries: Record<keyof T, TimeSeries>;
  insertFns: Record<keyof T, (series: TimeSeries, value: TimeSeriesValue) => TimeSeries>;
  ready: Promise<Record<keyof T, TimeSeries>>;
} {
  const result = createNamespacePersistentTimeSeries(
    namespaceId,
    seriesDefinitions,
    globalConfig,
  );

  return {
    initialSeries: result.initialState,
    insertFns: result.insertFns,
    ready: result.ready,
  };
}
