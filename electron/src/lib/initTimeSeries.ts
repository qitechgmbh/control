/**
 * @file initTimeseries.ts
 * @description Initialize the timeseries IndexedDB system and cleanup service.
 * This should be called once when the application starts.
 */

import { useEffect } from "react";
import { startCleanupService } from "./timeseriesCleanup";
import { destroyAllWriters } from "./timeseriesWriter";
import { closeDB } from "./indexedDBTimeseries";

/**
 * Initialize the timeseries system
 * Call this once when the application starts
 */
export function initializeTimeseriesSystem(): void {
  console.log("Initializing timeseries system...");

  // Start the cleanup service with default configuration
  // Keep data for 24 hours, cleanup every hour
  startCleanupService({
    retentionPeriodMs: 24 * 60 * 60 * 1000, // 24 hours
    cleanupIntervalMs: 60 * 60 * 1000, // 1 hour
    enabled: true,
  });

  console.log("Timeseries system initialized");
}

/**
 * Cleanup the timeseries system
 * Call this when the application is shutting down
 */
export async function cleanupTimeseriesSystem(): Promise<void> {
  console.log("Cleaning up timeseries system...");

  // Destroy all writers (flush pending writes)
  await destroyAllWriters();

  // Close the database connection
  closeDB();

  console.log("Timeseries system cleaned up");
}

/**
 * Hook to initialize timeseries system on component mount
 * Use this in your root component
 */
export function useInitializeTimeseries(): void {
  // Initialize on mount
  useEffect(() => {
    initializeTimeseriesSystem();

    // Cleanup on unmount
    return () => {
      cleanupTimeseriesSystem();
    };
  }, []);
}
