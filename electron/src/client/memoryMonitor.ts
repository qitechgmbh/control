import { trimAllNamespaceTimeSeries } from "./socketioStore";

const POLL_INTERVAL_MS = 30_000;
const WARN_RATIO = 0.85;
const TRIM_CUT_MS = 10 * 60 * 1000; // cut oldest 10 minutes per trim

interface PerformanceMemory {
  usedJSHeapSize: number;
  totalJSHeapSize: number;
  jsHeapSizeLimit: number;
}

/**
 * Polls the renderer's heap usage against its real ceiling (jsHeapSizeLimit,
 * set via --max-old-space-size in main.ts) and trims the oldest graph data
 * once usage crosses WARN_RATIO, to relieve memory pressure before OOM.
 */
export function startMemoryMonitor(): () => void {
  const perf = performance as Performance & { memory?: PerformanceMemory };
  if (!perf.memory) {
    console.warn("performance.memory unavailable; heap monitor disabled");
    return () => {};
  }

  const intervalId = setInterval(() => {
    const { usedJSHeapSize, jsHeapSizeLimit } = perf.memory!;
    if (!jsHeapSizeLimit) return;
    const ratio = usedJSHeapSize / jsHeapSizeLimit;

    if (ratio >= WARN_RATIO) {
      console.warn(
        `[memoryMonitor] heap usage ${(ratio * 100).toFixed(1)}% of limit ` +
          `(${(usedJSHeapSize / 1048576).toFixed(0)}MB / ${(jsHeapSizeLimit / 1048576).toFixed(0)}MB) — trimming graph buffers`,
      );
      trimAllNamespaceTimeSeries(TRIM_CUT_MS);
    }
  }, POLL_INTERVAL_MS);

  return () => clearInterval(intervalId);
}
