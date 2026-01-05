/**
 * @file timeseriesCleanup.ts
 * @description Automatic cleanup service for old timeseries data in IndexedDB.
 */

import { deleteOldDataPoints, getStorageStats } from "./indexedDBTimeseries";

/**
 * Configuration for cleanup service
 */
export interface CleanupConfig {
  /** How long to keep data (in milliseconds) */
  retentionPeriodMs: number;
  /** How often to run cleanup (in milliseconds) */
  cleanupIntervalMs: number;
  /** Whether cleanup is enabled */
  enabled: boolean;
}

/**
 * Default cleanup configuration
 * Keep data for 24 hours, cleanup every hour
 */
export const DEFAULT_CLEANUP_CONFIG: CleanupConfig = {
  retentionPeriodMs: 24 * 60 * 60 * 1000, // 24 hours
  cleanupIntervalMs: 60 * 60 * 1000, // 1 hour
  enabled: true,
};

/**
 * Cleanup service that automatically removes old data
 */
export class TimeSeriesCleanupService {
  private config: CleanupConfig;
  private intervalId: NodeJS.Timeout | null = null;
  private isRunning = false;

  constructor(config: Partial<CleanupConfig> = {}) {
    this.config = { ...DEFAULT_CLEANUP_CONFIG, ...config };
  }

  /**
   * Start the cleanup service
   */
  start(): void {
    if (this.isRunning || !this.config.enabled) {
      console.warn(
        "TimeSeriesCleanupService: Already running or disabled",
      );
      return;
    }

    console.log("TimeSeriesCleanupService: Starting cleanup service", {
      retentionHours: this.config.retentionPeriodMs / (60 * 60 * 1000),
      cleanupIntervalHours: this.config.cleanupIntervalMs / (60 * 60 * 1000),
    });

    this.isRunning = true;

    // Run cleanup immediately on start
    this.runCleanup();

    // Set up periodic cleanup
    this.intervalId = setInterval(() => {
      this.runCleanup();
    }, this.config.cleanupIntervalMs);
  }

  /**
   * Stop the cleanup service
   */
  stop(): void {
    if (!this.isRunning) {
      return;
    }

    console.log("TimeSeriesCleanupService: Stopping cleanup service");

    if (this.intervalId) {
      clearInterval(this.intervalId);
      this.intervalId = null;
    }

    this.isRunning = false;
  }

  /**
   * Run cleanup immediately
   */
  async runCleanup(): Promise<void> {
    try {
      const startTime = Date.now();

      // Get stats before cleanup
      const statsBefore = await getStorageStats();
      console.log("TimeSeriesCleanupService: Storage stats before cleanup", {
        totalPoints: statsBefore.totalPoints,
        seriesCount: statsBefore.seriesCount,
        oldestTimestamp: statsBefore.oldestTimestamp,
        newestTimestamp: statsBefore.newestTimestamp,
      });

      // Calculate cutoff time
      const cutoffTime = Date.now() - this.config.retentionPeriodMs;

      // Delete old data
      const deletedCount = await deleteOldDataPoints(cutoffTime);

      // Get stats after cleanup
      const statsAfter = await getStorageStats();

      const duration = Date.now() - startTime;

      console.log("TimeSeriesCleanupService: Cleanup completed", {
        deletedPoints: deletedCount,
        remainingPoints: statsAfter.totalPoints,
        durationMs: duration,
      });
    } catch (error) {
      console.error("TimeSeriesCleanupService: Error during cleanup", error);
    }
  }

  /**
   * Update cleanup configuration
   */
  updateConfig(config: Partial<CleanupConfig>): void {
    const wasRunning = this.isRunning;

    if (wasRunning) {
      this.stop();
    }

    this.config = { ...this.config, ...config };

    if (wasRunning && this.config.enabled) {
      this.start();
    }
  }

  /**
   * Get current configuration
   */
  getConfig(): CleanupConfig {
    return { ...this.config };
  }

  /**
   * Check if service is running
   */
  isActive(): boolean {
    return this.isRunning;
  }
}

/**
 * Global cleanup service instance
 */
let globalCleanupService: TimeSeriesCleanupService | null = null;

/**
 * Get or create the global cleanup service
 */
export function getCleanupService(
  config?: Partial<CleanupConfig>,
): TimeSeriesCleanupService {
  if (!globalCleanupService) {
    globalCleanupService = new TimeSeriesCleanupService(config);
  } else if (config) {
    globalCleanupService.updateConfig(config);
  }
  return globalCleanupService;
}

/**
 * Start the global cleanup service
 */
export function startCleanupService(
  config?: Partial<CleanupConfig>,
): TimeSeriesCleanupService {
  const service = getCleanupService(config);
  service.start();
  return service;
}

/**
 * Stop the global cleanup service
 */
export function stopCleanupService(): void {
  if (globalCleanupService) {
    globalCleanupService.stop();
  }
}
