/**
 * IndexedDB-based persistence layer for TimeSeries data
 * 
 * This module provides persistent storage for long timeseries data using IndexedDB.
 * Short timeseries data remains in memory only for performance.
 * 
 * Database schema:
 * - Store: "timeseries_data"
 * - Key: [namespaceId, seriesName, timestamp]
 * - Value: { value: number, timestamp: number }
 * 
 * The namespaceId is serialized from the NamespaceId object to uniquely identify
 * each machine instance's data.
 */

import { openDB, DBSchema, IDBPDatabase } from "idb";
import { TimeSeriesValue } from "./timeseries";

// Database configuration
const DB_NAME = "QiTechControlTimeSeries";
const DB_VERSION = 1;
const STORE_NAME = "timeseries_data";

/**
 * Database schema definition for TypeScript
 */
interface TimeSeriesDB extends DBSchema {
  timeseries_data: {
    key: string; // Composite key: "namespaceId:seriesName:timestamp"
    value: TimeSeriesValue & {
      namespaceId: string;
      seriesName: string;
    };
    indexes: {
      "by-namespace": string; // namespaceId
      "by-namespace-series": string; // "namespaceId:seriesName"
      "by-namespace-series-time": [string, number]; // ["namespaceId:seriesName", timestamp]
    };
  };
}

/**
 * Singleton database instance
 */
let dbInstance: IDBPDatabase<TimeSeriesDB> | null = null;

/**
 * Initialize or get the database instance
 */
async function getDB(): Promise<IDBPDatabase<TimeSeriesDB>> {
  if (dbInstance) {
    return dbInstance;
  }

  dbInstance = await openDB<TimeSeriesDB>(DB_NAME, DB_VERSION, {
    upgrade(db) {
      // Create the object store with composite key
      const store = db.createObjectStore(STORE_NAME, {
        keyPath: ["namespaceId", "seriesName", "timestamp"],
      });

      // Create indexes for efficient querying
      store.createIndex("by-namespace", "namespaceId", { unique: false });
      store.createIndex("by-namespace-series", ["namespaceId", "seriesName"], {
        unique: false,
      });
      store.createIndex(
        "by-namespace-series-time",
        ["namespaceId", "seriesName", "timestamp"],
        { unique: true },
      );
    },
  });

  return dbInstance;
}

/**
 * Store a single data point
 * @param namespaceId Serialized namespace identifier
 * @param seriesName Name of the time series (e.g., "pullerSpeed")
 * @param value The time series value to store
 */
export async function storeDataPoint(
  namespaceId: string,
  seriesName: string,
  value: TimeSeriesValue,
): Promise<void> {
  const db = await getDB();
  await db.put(STORE_NAME, {
    namespaceId,
    seriesName,
    ...value,
  });
}

/**
 * Store multiple data points in a single transaction (more efficient)
 * @param namespaceId Serialized namespace identifier
 * @param seriesName Name of the time series
 * @param values Array of time series values to store
 */
export async function storeDataPoints(
  namespaceId: string,
  seriesName: string,
  values: TimeSeriesValue[],
): Promise<void> {
  const db = await getDB();
  const tx = db.transaction(STORE_NAME, "readwrite");

  await Promise.all([
    ...values.map((value) =>
      tx.store.put({
        namespaceId,
        seriesName,
        ...value,
      }),
    ),
    tx.done,
  ]);
}

/**
 * Query data points within a time window
 * @param namespaceId Serialized namespace identifier
 * @param seriesName Name of the time series
 * @param startTime Start timestamp (inclusive)
 * @param endTime End timestamp (inclusive)
 * @returns Array of time series values in chronological order
 */
export async function queryDataPoints(
  namespaceId: string,
  seriesName: string,
  startTime?: number,
  endTime?: number,
): Promise<TimeSeriesValue[]> {
  const db = await getDB();

  // Use the composite index for efficient range queries
  const index = db
    .transaction(STORE_NAME)
    .store.index("by-namespace-series-time");

  let range: IDBKeyRange;

  if (startTime !== undefined && endTime !== undefined) {
    // Query for specific time range
    range = IDBKeyRange.bound(
      [namespaceId, seriesName, startTime],
      [namespaceId, seriesName, endTime],
    );
  } else if (startTime !== undefined) {
    // Query from startTime onwards
    range = IDBKeyRange.lowerBound([namespaceId, seriesName, startTime]);
  } else if (endTime !== undefined) {
    // Query up to endTime
    range = IDBKeyRange.upperBound([namespaceId, seriesName, endTime]);
  } else {
    // Query all data for this namespace:series combination
    range = IDBKeyRange.bound(
      [namespaceId, seriesName, 0],
      [namespaceId, seriesName, Number.MAX_SAFE_INTEGER],
    );
  }

  const results = await index.getAll(range);

  // Convert to TimeSeriesValue format (remove namespaceId and seriesName)
  return results.map(({ value, timestamp }) => ({ value, timestamp }));
}

/**
 * Get the latest N data points for a series
 * @param namespaceId Serialized namespace identifier
 * @param seriesName Name of the time series
 * @param count Number of latest points to retrieve
 * @returns Array of time series values in chronological order
 */
export async function queryLatestDataPoints(
  namespaceId: string,
  seriesName: string,
  count: number,
): Promise<TimeSeriesValue[]> {
  const db = await getDB();

  // Get all data for this series (we need to scan to find the latest)
  const index = db
    .transaction(STORE_NAME)
    .store.index("by-namespace-series-time");
  const range = IDBKeyRange.bound(
    [namespaceId, seriesName, 0],
    [namespaceId, seriesName, Number.MAX_SAFE_INTEGER],
  );

  const results = await index.getAll(range);

  // Sort by timestamp descending and take the first 'count' items
  const sorted = results
    .map(({ value, timestamp }) => ({ value, timestamp }))
    .sort((a, b) => b.timestamp - a.timestamp)
    .slice(0, count);

  // Reverse to get chronological order
  return sorted.reverse();
}

/**
 * Delete old data points before a given timestamp
 * @param namespaceId Serialized namespace identifier
 * @param seriesName Name of the time series
 * @param beforeTimestamp Delete all points before this timestamp
 * @returns Number of deleted entries
 */
export async function deleteOldDataPoints(
  namespaceId: string,
  seriesName: string,
  beforeTimestamp: number,
): Promise<number> {
  const db = await getDB();

  // Query old data points
  const index = db
    .transaction(STORE_NAME, "readwrite")
    .store.index("by-namespace-series-time");

  const range = IDBKeyRange.bound(
    [namespaceId, seriesName, 0],
    [namespaceId, seriesName, beforeTimestamp],
    false,
    true, // exclusive upper bound
  );

  const keys = await index.getAllKeys(range);
  const tx = db.transaction(STORE_NAME, "readwrite");

  await Promise.all([
    ...keys.map((key) => tx.store.delete(key as any)),
    tx.done,
  ]);

  return keys.length;
}

/**
 * Delete all data for a specific namespace and series
 * @param namespaceId Serialized namespace identifier
 * @param seriesName Name of the time series
 */
export async function clearSeriesData(
  namespaceId: string,
  seriesName: string,
): Promise<void> {
  const db = await getDB();

  const index = db
    .transaction(STORE_NAME, "readwrite")
    .store.index("by-namespace-series-time");

  const range = IDBKeyRange.bound(
    [namespaceId, seriesName, 0],
    [namespaceId, seriesName, Number.MAX_SAFE_INTEGER],
  );

  const keys = await index.getAllKeys(range);
  const tx = db.transaction(STORE_NAME, "readwrite");

  await Promise.all([
    ...keys.map((key) => tx.store.delete(key as any)),
    tx.done,
  ]);
}

/**
 * Delete all data for a specific namespace (all series)
 * @param namespaceId Serialized namespace identifier
 */
export async function clearNamespaceData(namespaceId: string): Promise<void> {
  const db = await getDB();

  const index = db
    .transaction(STORE_NAME, "readwrite")
    .store.index("by-namespace");

  const keys = await index.getAllKeys(namespaceId);
  const tx = db.transaction(STORE_NAME, "readwrite");

  await Promise.all([
    ...keys.map((key) => tx.store.delete(key as any)),
    tx.done,
  ]);
}

/**
 * Get statistics about stored data
 * @param namespaceId Serialized namespace identifier
 * @param seriesName Name of the time series
 */
export async function getSeriesStats(
  namespaceId: string,
  seriesName: string,
): Promise<{
  count: number;
  oldestTimestamp: number | null;
  newestTimestamp: number | null;
}> {
  const db = await getDB();

  const index = db
    .transaction(STORE_NAME)
    .store.index("by-namespace-series-time");

  const range = IDBKeyRange.bound(
    [namespaceId, seriesName, 0],
    [namespaceId, seriesName, Number.MAX_SAFE_INTEGER],
  );

  const results = await index.getAll(range);

  if (results.length === 0) {
    return {
      count: 0,
      oldestTimestamp: null,
      newestTimestamp: null,
    };
  }

  const timestamps = results.map((r) => r.timestamp);

  return {
    count: results.length,
    oldestTimestamp: Math.min(...timestamps),
    newestTimestamp: Math.max(...timestamps),
  };
}

/**
 * Close the database connection
 */
export async function closeDB(): Promise<void> {
  if (dbInstance) {
    dbInstance.close();
    dbInstance = null;
  }
}
