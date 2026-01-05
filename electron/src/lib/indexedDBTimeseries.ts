/**
 * @file indexedDBTimeseries.ts
 * @description IndexedDB-based storage for timeseries data points.
 * Replaces RAM-based storage for better memory management.
 */

import { openDB, DBSchema, IDBPDatabase } from "idb";

/**
 * Data point stored in IndexedDB
 */
export interface TimeSeriesDataPoint {
  /** Composite key: seriesKey + timestamp */
  id: string;
  /** Series identifier (e.g., "mock1:serial123:amplitude1") */
  seriesKey: string;
  /** Timestamp in milliseconds */
  timestamp: number;
  /** The actual value */
  value: number;
}

/**
 * Database schema
 */
interface TimeSeriesDB extends DBSchema {
  datapoints: {
    key: string; // id field
    value: TimeSeriesDataPoint;
    indexes: {
      "by-series": string; // seriesKey
      "by-series-time": [string, number]; // [seriesKey, timestamp]
      "by-timestamp": number; // timestamp for cleanup
    };
  };
}

const DB_NAME = "timeseriesDB";
const DB_VERSION = 1;
const STORE_NAME = "datapoints";

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
      // Create object store
      const store = db.createObjectStore(STORE_NAME, { keyPath: "id" });

      // Create indexes
      store.createIndex("by-series", "seriesKey", { unique: false });
      store.createIndex("by-series-time", ["seriesKey", "timestamp"], {
        unique: false,
      });
      store.createIndex("by-timestamp", "timestamp", { unique: false });
    },
  });

  return dbInstance;
}

/**
 * Generate a unique ID for a datapoint
 */
function generateDataPointId(seriesKey: string, timestamp: number): string {
  return `${seriesKey}:${timestamp}`;
}

/**
 * Store a single data point
 */
export async function storeDataPoint(
  seriesKey: string,
  timestamp: number,
  value: number,
): Promise<void> {
  const db = await getDB();
  const dataPoint: TimeSeriesDataPoint = {
    id: generateDataPointId(seriesKey, timestamp),
    seriesKey,
    timestamp,
    value,
  };

  await db.put(STORE_NAME, dataPoint);
}

/**
 * Store multiple data points in a batch (more efficient)
 */
export async function storeDataPointsBatch(
  points: Array<{
    seriesKey: string;
    timestamp: number;
    value: number;
  }>,
): Promise<void> {
  const db = await getDB();
  const tx = db.transaction(STORE_NAME, "readwrite");
  const store = tx.objectStore(STORE_NAME);

  await Promise.all(
    points.map((point) => {
      const dataPoint: TimeSeriesDataPoint = {
        id: generateDataPointId(point.seriesKey, point.timestamp),
        seriesKey: point.seriesKey,
        timestamp: point.timestamp,
        value: point.value,
      };
      return store.put(dataPoint);
    }),
  );

  await tx.done;
}

/**
 * Query data points for a specific series within a time range
 */
export async function queryDataPoints(
  seriesKey: string,
  startTime: number,
  endTime: number,
): Promise<TimeSeriesDataPoint[]> {
  const db = await getDB();
  const tx = db.transaction(STORE_NAME, "readonly");
  const index = tx.objectStore(STORE_NAME).index("by-series-time");

  const range = IDBKeyRange.bound(
    [seriesKey, startTime],
    [seriesKey, endTime],
  );

  const results = await index.getAll(range);
  await tx.done;

  return results;
}

/**
 * Query the most recent N data points for a series
 */
export async function queryRecentDataPoints(
  seriesKey: string,
  limit: number,
): Promise<TimeSeriesDataPoint[]> {
  const db = await getDB();
  const tx = db.transaction(STORE_NAME, "readonly");
  const index = tx.objectStore(STORE_NAME).index("by-series-time");

  // Get all points for this series (they're already sorted by timestamp due to compound index)
  const range = IDBKeyRange.bound(
    [seriesKey, 0],
    [seriesKey, Number.MAX_SAFE_INTEGER],
  );

  let cursor = await index.openCursor(range, "prev"); // Reverse order to get newest first
  const results: TimeSeriesDataPoint[] = [];

  while (cursor && results.length < limit) {
    results.push(cursor.value);
    cursor = await cursor.continue();
  }

  await tx.done;

  // Reverse to get chronological order
  return results.reverse();
}

/**
 * Get the latest data point for a series
 */
export async function getLatestDataPoint(
  seriesKey: string,
): Promise<TimeSeriesDataPoint | null> {
  const results = await queryRecentDataPoints(seriesKey, 1);
  return results.length > 0 ? results[0] : null;
}

/**
 * Delete data points older than a given timestamp (for cleanup)
 */
export async function deleteOldDataPoints(
  olderThanTimestamp: number,
): Promise<number> {
  const db = await getDB();
  const tx = db.transaction(STORE_NAME, "readwrite");
  const store = tx.objectStore(STORE_NAME);
  const index = store.index("by-timestamp");

  const range = IDBKeyRange.upperBound(olderThanTimestamp);
  let cursor = await index.openCursor(range);
  let deleteCount = 0;

  while (cursor) {
    await cursor.delete();
    deleteCount++;
    cursor = await cursor.continue();
  }

  await tx.done;
  return deleteCount;
}

/**
 * Delete all data points for a specific series
 */
export async function deleteSeriesData(seriesKey: string): Promise<number> {
  const db = await getDB();
  const tx = db.transaction(STORE_NAME, "readwrite");
  const store = tx.objectStore(STORE_NAME);
  const index = store.index("by-series");

  const range = IDBKeyRange.only(seriesKey);
  let cursor = await index.openCursor(range);
  let deleteCount = 0;

  while (cursor) {
    await cursor.delete();
    deleteCount++;
    cursor = await cursor.continue();
  }

  await tx.done;
  return deleteCount;
}

/**
 * Get statistics about stored data
 */
export async function getStorageStats(): Promise<{
  totalPoints: number;
  seriesCount: number;
  oldestTimestamp: number | null;
  newestTimestamp: number | null;
}> {
  const db = await getDB();
  const tx = db.transaction(STORE_NAME, "readonly");
  const store = tx.objectStore(STORE_NAME);

  const totalPoints = await store.count();

  // Get unique series keys
  const index = store.index("by-series");
  const seriesKeys = new Set<string>();
  let cursor = await index.openCursor(null, "nextunique");
  while (cursor) {
    seriesKeys.add(cursor.value.seriesKey);
    cursor = await cursor.continue();
  }

  // Get timestamp range
  const timeIndex = store.index("by-timestamp");
  let oldestTimestamp: number | null = null;
  let newestTimestamp: number | null = null;

  const oldestCursor = await timeIndex.openCursor(null, "next");
  if (oldestCursor) {
    oldestTimestamp = oldestCursor.value.timestamp;
  }

  const newestCursor = await timeIndex.openCursor(null, "prev");
  if (newestCursor) {
    newestTimestamp = newestCursor.value.timestamp;
  }

  await tx.done;

  return {
    totalPoints,
    seriesCount: seriesKeys.size,
    oldestTimestamp,
    newestTimestamp,
  };
}

/**
 * Clear all data from the database
 */
export async function clearAllData(): Promise<void> {
  const db = await getDB();
  const tx = db.transaction(STORE_NAME, "readwrite");
  await tx.objectStore(STORE_NAME).clear();
  await tx.done;
}

/**
 * Close the database connection
 */
export function closeDB(): void {
  if (dbInstance) {
    dbInstance.close();
    dbInstance = null;
  }
}
