/**
 * IndexedDB Storage Management Utilities
 * 
 * This module provides user-facing utilities for managing IndexedDB storage:
 * - Check storage usage and quota
 * - Clear data for specific namespaces or series
 * - Export data to CSV/JSON
 * - Import data from backups
 * - Monitor storage health
 * 
 * These utilities can be integrated into settings pages or developer tools.
 */

import { TimeSeriesValue } from "./timeseries";
import {
  queryDataPoints,
  clearSeriesData,
  clearNamespaceData,
  getSeriesStats,
  closeDB,
} from "./timeseriesDB";

/**
 * Storage statistics for a namespace
 */
export interface NamespaceStorageStats {
  namespaceId: string;
  series: {
    name: string;
    count: number;
    oldestTimestamp: number | null;
    newestTimestamp: number | null;
    estimatedSizeBytes: number;
  }[];
  totalDataPoints: number;
  estimatedTotalSizeBytes: number;
}

/**
 * Overall storage health information
 */
export interface StorageHealth {
  usedBytes: number;
  quotaBytes: number;
  percentUsed: number;
  namespaces: NamespaceStorageStats[];
}

/**
 * Get browser's IndexedDB storage usage
 */
export async function getStorageUsage(): Promise<{
  usage: number;
  quota: number;
  percentUsed: number;
}> {
  if (!navigator.storage || !navigator.storage.estimate) {
    throw new Error("Storage API not supported");
  }

  const estimate = await navigator.storage.estimate();
  const usage = estimate.usage || 0;
  const quota = estimate.quota || 0;
  const percentUsed = quota > 0 ? (usage / quota) * 100 : 0;

  return { usage, quota, percentUsed };
}

/**
 * Get statistics for a specific namespace and series
 */
export async function getNamespaceSeriesStats(
  namespaceId: string,
  seriesName: string,
): Promise<{
  count: number;
  oldestTimestamp: number | null;
  newestTimestamp: number | null;
  estimatedSizeBytes: number;
}> {
  const stats = await getSeriesStats(namespaceId, seriesName);

  // Estimate size: ~40 bytes per data point (value, timestamp, keys, metadata)
  const estimatedSizeBytes = stats.count * 40;

  return {
    ...stats,
    estimatedSizeBytes,
  };
}

/**
 * Clear all data for a specific series
 * @returns true if successful
 */
export async function clearSeries(
  namespaceId: string,
  seriesName: string,
): Promise<boolean> {
  try {
    await clearSeriesData(namespaceId, seriesName);
    console.log(`Cleared data for ${namespaceId}:${seriesName}`);
    return true;
  } catch (error) {
    console.error(`Failed to clear ${namespaceId}:${seriesName}:`, error);
    return false;
  }
}

/**
 * Clear all data for a namespace (all series)
 * @returns true if successful
 */
export async function clearNamespace(namespaceId: string): Promise<boolean> {
  try {
    await clearNamespaceData(namespaceId);
    console.log(`Cleared all data for namespace ${namespaceId}`);
    return true;
  } catch (error) {
    console.error(`Failed to clear namespace ${namespaceId}:`, error);
    return false;
  }
}

/**
 * Export series data to CSV format
 */
export async function exportToCSV(
  namespaceId: string,
  seriesName: string,
  startTime?: number,
  endTime?: number,
): Promise<string> {
  const dataPoints = await queryDataPoints(
    namespaceId,
    seriesName,
    startTime,
    endTime,
  );

  // Create CSV header
  let csv = "timestamp,value,datetime\n";

  // Add data rows
  for (const point of dataPoints) {
    const datetime = new Date(point.timestamp).toISOString();
    csv += `${point.timestamp},${point.value},${datetime}\n`;
  }

  return csv;
}

/**
 * Export series data to JSON format
 */
export async function exportToJSON(
  namespaceId: string,
  seriesName: string,
  startTime?: number,
  endTime?: number,
): Promise<string> {
  const dataPoints = await queryDataPoints(
    namespaceId,
    seriesName,
    startTime,
    endTime,
  );

  const exportData = {
    namespaceId,
    seriesName,
    exportTimestamp: Date.now(),
    startTime,
    endTime,
    dataPoints: dataPoints.map((point) => ({
      ...point,
      datetime: new Date(point.timestamp).toISOString(),
    })),
  };

  return JSON.stringify(exportData, null, 2);
}

/**
 * Download data as a file
 */
export function downloadFile(
  content: string,
  filename: string,
  mimeType: string,
): void {
  const blob = new Blob([content], { type: mimeType });
  const url = URL.createObjectURL(blob);
  const a = document.createElement("a");
  a.href = url;
  a.download = filename;
  document.body.appendChild(a);
  a.click();
  document.body.removeChild(a);
  URL.revokeObjectURL(url);
}

/**
 * Export series data and download as CSV file
 */
export async function downloadSeriesAsCSV(
  namespaceId: string,
  seriesName: string,
  startTime?: number,
  endTime?: number,
): Promise<void> {
  const csv = await exportToCSV(namespaceId, seriesName, startTime, endTime);
  const timestamp = new Date().toISOString().replace(/[:.]/g, "-");
  const filename = `${seriesName}_${timestamp}.csv`;
  downloadFile(csv, filename, "text/csv");
}

/**
 * Export series data and download as JSON file
 */
export async function downloadSeriesAsJSON(
  namespaceId: string,
  seriesName: string,
  startTime?: number,
  endTime?: number,
): Promise<void> {
  const json = await exportToJSON(namespaceId, seriesName, startTime, endTime);
  const timestamp = new Date().toISOString().replace(/[:.]/g, "-");
  const filename = `${seriesName}_${timestamp}.json`;
  downloadFile(json, filename, "application/json");
}

/**
 * Parse and validate imported JSON data
 */
export function parseImportedJSON(jsonString: string): {
  namespaceId: string;
  seriesName: string;
  dataPoints: TimeSeriesValue[];
} {
  const data = JSON.parse(jsonString);

  if (
    !data.namespaceId ||
    !data.seriesName ||
    !Array.isArray(data.dataPoints)
  ) {
    throw new Error("Invalid import format");
  }

  // Validate data points
  const dataPoints: TimeSeriesValue[] = data.dataPoints.map(
    (point: any, index: number) => {
      if (
        typeof point.timestamp !== "number" ||
        typeof point.value !== "number"
      ) {
        throw new Error(`Invalid data point at index ${index}`);
      }
      return {
        timestamp: point.timestamp,
        value: point.value,
      };
    },
  );

  return {
    namespaceId: data.namespaceId,
    seriesName: data.seriesName,
    dataPoints,
  };
}

/**
 * Close the IndexedDB connection
 * Useful when clearing browser storage or before reinstalling
 */
export async function closeDatabase(): Promise<void> {
  await closeDB();
}

/**
 * Check if persistence is available in this browser
 */
export function isPersistenceAvailable(): boolean {
  return (
    typeof indexedDB !== "undefined" &&
    typeof navigator.storage !== "undefined" &&
    typeof navigator.storage.estimate === "function"
  );
}

/**
 * Request persistent storage (prevents browser from clearing data)
 */
export async function requestPersistentStorage(): Promise<boolean> {
  if (!navigator.storage || !navigator.storage.persist) {
    return false;
  }

  try {
    const isPersisted = await navigator.storage.persisted();
    if (isPersisted) {
      return true;
    }

    const result = await navigator.storage.persist();
    return result;
  } catch (error) {
    console.error("Failed to request persistent storage:", error);
    return false;
  }
}

/**
 * Check if storage is persisted (won't be cleared by browser)
 */
export async function isStoragePersisted(): Promise<boolean> {
  if (!navigator.storage || !navigator.storage.persisted) {
    return false;
  }

  try {
    return await navigator.storage.persisted();
  } catch (error) {
    console.error("Failed to check persistence status:", error);
    return false;
  }
}

/**
 * Format bytes to human-readable string
 */
export function formatBytes(bytes: number, decimals: number = 2): string {
  if (bytes === 0) return "0 Bytes";

  const k = 1024;
  const dm = decimals < 0 ? 0 : decimals;
  const sizes = ["Bytes", "KB", "MB", "GB"];

  const i = Math.floor(Math.log(bytes) / Math.log(k));

  return parseFloat((bytes / Math.pow(k, i)).toFixed(dm)) + " " + sizes[i];
}

/**
 * Format timestamp to human-readable date/time
 */
export function formatTimestamp(timestamp: number): string {
  return new Date(timestamp).toLocaleString();
}

/**
 * Calculate time range duration in human-readable format
 */
export function formatDuration(startTime: number, endTime: number): string {
  const durationMs = endTime - startTime;
  const seconds = Math.floor(durationMs / 1000);
  const minutes = Math.floor(seconds / 60);
  const hours = Math.floor(minutes / 60);
  const days = Math.floor(hours / 24);

  if (days > 0) {
    return `${days}d ${hours % 24}h`;
  } else if (hours > 0) {
    return `${hours}h ${minutes % 60}m`;
  } else if (minutes > 0) {
    return `${minutes}m ${seconds % 60}s`;
  } else {
    return `${seconds}s`;
  }
}
