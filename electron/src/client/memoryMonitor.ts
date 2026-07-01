/**
 * Periodic heap monitoring for the Electron renderer process.
 *
 * Logs periodic memory snapshots and fires warning/critical callbacks
 * to help detect and diagnose V8 heap OOM crashes before they happen.
 *
 * Requires `--enable-precise-memory-info` flag in Electron main process.
 */

export interface MemorySnapshot {
  /** Total heap size in MB */
  heapTotal: number;
  /** Currently used heap in MB */
  heapUsed: number;
  /** External memory (ArrayBuffers, etc.) in MB */
  external: number;
  /** Resident Set Size (total process memory) in MB */
  rss: number;
  /** Timestamp of the snapshot */
  timestamp: number;
  /** Heap growth since the last snapshot in MB */
  growthSinceLast: number;
}

/**
 * Performance.memory is exposed by Chromium when the
 * `--enable-precise-memory-info` flag is set.
 */
interface ChromiumMemoryInfo {
  totalJSHeapSize: number;
  usedJSHeapSize: number;
  jsHeapSizeLimit: number;
}

/**
 * Gets the raw Chromium memory info if available.
 * Returns null in non-Electron environments or when the flag is not set.
 */
function getChromiumMemory(): ChromiumMemoryInfo | null {
  const perf = performance as { memory?: ChromiumMemoryInfo };
  return perf.memory ?? null;
}

/**
 * Formats bytes to MB with 1 decimal place.
 */
function toMB(bytes: number): number {
  return Math.round((bytes / (1024 * 1024)) * 10) / 10;
}

export interface MemoryMonitorOptions {
  /** Interval in ms between checks (default: 30000 = 30s) */
  intervalMs: number;
  /** Fraction of heap used that triggers a warning log (default: 0.6 = 60%) */
  warningThreshold: number;
  /** Fraction of heap used that triggers critical callback (default: 0.85 = 85%) */
  criticalThreshold: number;
}

const DEFAULTS: MemoryMonitorOptions = {
  intervalMs: 30000,
  warningThreshold: 0.6,
  criticalThreshold: 0.85,
};

export class MemoryMonitor {
  private lastSnapshot: MemorySnapshot | null = null;
  private intervalId: ReturnType<typeof setInterval> | null = null;
  private options: MemoryMonitorOptions;

  /** Called when heap usage exceeds criticalThreshold */
  public onCritical?: (snapshot: MemorySnapshot) => void;
  /** Called on every check with the current snapshot */
  public onCheck?: (snapshot: MemorySnapshot) => void;

  constructor(options: Partial<MemoryMonitorOptions> = {}) {
    this.options = { ...DEFAULTS, ...options };
  }

  /**
   * Start periodic heap monitoring.
   * Safe to call multiple times — subsequent calls are no-ops.
   */
  start(): void {
    if (this.intervalId !== null) return;

    // Run an immediate first check
    this.check();

    this.intervalId = setInterval(() => {
      this.check();
    }, this.options.intervalMs);
  }

  /**
   * Stop periodic heap monitoring.
   */
  stop(): void {
    if (this.intervalId !== null) {
      clearInterval(this.intervalId);
      this.intervalId = null;
    }
  }

  /**
   * Returns the most recent snapshot or null if no check has run yet.
   */
  getLastSnapshot(): MemorySnapshot | null {
    return this.lastSnapshot;
  }

  /**
   * Perform a single memory check and log results.
   */
  private check(): void {
    const mem = getChromiumMemory();
    if (!mem) return; // Not available (missing flag or non-Electron environment)

    const heapTotal = toMB(mem.totalJSHeapSize);
    const heapUsed = toMB(mem.usedJSHeapSize);
    const growthSinceLast = this.lastSnapshot
      ? Math.round((heapUsed - this.lastSnapshot.heapUsed) * 10) / 10
      : 0;

    const snapshot: MemorySnapshot = {
      heapTotal,
      heapUsed,
      external: 0, // Not directly exposed by performance.memory, would need process.memoryUsage() in main
      rss: 0, // Not available in renderer
      timestamp: Date.now(),
      growthSinceLast,
    };

    const usageRatio = heapUsed / heapTotal;

    // Log every check at debug level
    console.debug(
      `[MemoryMonitor] Heap: ${heapUsed}MB / ${heapTotal}MB ` +
        `(${Math.round(usageRatio * 100)}%) ` +
        `| Growth: ${growthSinceLast > 0 ? "+" : ""}${growthSinceLast}MB`,
    );

    // Warning threshold
    if (usageRatio > this.options.warningThreshold) {
      console.warn(
        `[MemoryMonitor] WARNING: ${Math.round(usageRatio * 100)}% heap used ` +
          `(${heapUsed}MB / ${heapTotal}MB)`,
      );
    }

    // Critical threshold
    if (usageRatio > this.options.criticalThreshold) {
      console.error(
        `[MemoryMonitor] CRITICAL: ${Math.round(usageRatio * 100)}% heap used ` +
          `(${heapUsed}MB / ${heapTotal}MB)` +
          `— application may crash soon!`,
      );
      this.onCritical?.(snapshot);
    }

    this.onCheck?.(snapshot);
    this.lastSnapshot = snapshot;
  }
}

/**
 * Singleton instance with default configuration.
 * Start with: `memoryMonitor.start()`
 * Stop with: `memoryMonitor.stop()`
 */
export const memoryMonitor = new MemoryMonitor();
