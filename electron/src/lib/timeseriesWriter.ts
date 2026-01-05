/**
 * @file timeseriesWriter.ts
 * @description Utilities for writing timeseries data to IndexedDB from namespace events.
 */

import { storeDataPoint, storeDataPointsBatch } from "./indexedDBTimeseries";

/**
 * Configuration for a timeseries writer
 */
export interface TimeSeriesWriterConfig {
  /** How often to flush batched writes (ms) */
  batchFlushInterval?: number;
  /** Maximum batch size before forcing a flush */
  maxBatchSize?: number;
  /** Whether to enable batching */
  enableBatching?: boolean;
}

const DEFAULT_CONFIG: Required<TimeSeriesWriterConfig> = {
  batchFlushInterval: 100, // Flush every 100ms
  maxBatchSize: 50, // Flush after 50 points
  enableBatching: true,
};

/**
 * A writer that batches data points for efficient IndexedDB writes
 */
export class TimeSeriesWriter {
  private config: Required<TimeSeriesWriterConfig>;
  private batch: Array<{
    seriesKey: string;
    timestamp: number;
    value: number;
  }> = [];
  private flushTimer: NodeJS.Timeout | null = null;
  private isDestroyed = false;

  constructor(config: TimeSeriesWriterConfig = {}) {
    this.config = { ...DEFAULT_CONFIG, ...config };

    if (this.config.enableBatching) {
      this.startFlushTimer();
    }
  }

  /**
   * Write a single data point (will be batched if batching is enabled)
   */
  async write(
    seriesKey: string,
    timestamp: number,
    value: number,
  ): Promise<void> {
    if (this.isDestroyed) {
      console.warn("TimeSeriesWriter: Attempted to write to destroyed writer");
      return;
    }

    if (!this.config.enableBatching) {
      // Write immediately
      await storeDataPoint(seriesKey, timestamp, value);
      return;
    }

    // Add to batch
    this.batch.push({ seriesKey, timestamp, value });

    // Flush if batch is full
    if (this.batch.length >= this.config.maxBatchSize) {
      await this.flush();
    }
  }

  /**
   * Flush all pending writes to IndexedDB
   */
  async flush(): Promise<void> {
    if (this.batch.length === 0) {
      return;
    }

    const toWrite = [...this.batch];
    this.batch = [];

    try {
      await storeDataPointsBatch(toWrite);
    } catch (error) {
      console.error("TimeSeriesWriter: Error flushing batch", error);
      // Re-add to batch on error (basic retry mechanism)
      this.batch.unshift(...toWrite);
    }
  }

  /**
   * Start the periodic flush timer
   */
  private startFlushTimer(): void {
    this.flushTimer = setInterval(() => {
      this.flush().catch((err) => {
        console.error("TimeSeriesWriter: Error in periodic flush", err);
      });
    }, this.config.batchFlushInterval);
  }

  /**
   * Stop the flush timer and flush any remaining data
   */
  async destroy(): Promise<void> {
    this.isDestroyed = true;

    if (this.flushTimer) {
      clearInterval(this.flushTimer);
      this.flushTimer = null;
    }

    await this.flush();
  }
}

/**
 * Global writer instances keyed by namespace ID
 */
const writerInstances = new Map<string, TimeSeriesWriter>();

/**
 * Get or create a writer for a namespace
 */
export function getWriterForNamespace(
  namespaceId: string,
  config?: TimeSeriesWriterConfig,
): TimeSeriesWriter {
  let writer = writerInstances.get(namespaceId);
  if (!writer) {
    writer = new TimeSeriesWriter(config);
    writerInstances.set(namespaceId, writer);
  }
  return writer;
}

/**
 * Destroy a writer for a namespace (cleanup)
 */
export async function destroyWriterForNamespace(
  namespaceId: string,
): Promise<void> {
  const writer = writerInstances.get(namespaceId);
  if (writer) {
    await writer.destroy();
    writerInstances.delete(namespaceId);
  }
}

/**
 * Destroy all writers (cleanup on app shutdown)
 */
export async function destroyAllWriters(): Promise<void> {
  const promises = Array.from(writerInstances.values()).map((writer) =>
    writer.destroy(),
  );
  await Promise.all(promises);
  writerInstances.clear();
}

/**
 * Helper to generate a series key from namespace and field name
 */
export function generateSeriesKey(
  machineType: string,
  serial: number,
  fieldName: string,
): string {
  return `${machineType}:${serial}:${fieldName}`;
}
