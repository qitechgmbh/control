/**
 * Configuration helper for Gluetex machine settings
 */

const GLUETEX_XL_MODE_KEY = "gluetex_xl_mode";
const GLUETEX_GRAPH_SAMPLE_INTERVAL_KEY = "gluetex_graph_sample_interval";
const GLUETEX_GRAPH_RETENTION_KEY = "gluetex_graph_retention";

export const GLUETEX_TRAVERSE_MAX_STANDARD = 180; // mm
export const GLUETEX_TRAVERSE_MAX_XL = 385; // mm

// Default graph config: 1s sample interval, 1h retention
export const DEFAULT_GRAPH_SAMPLE_INTERVAL = 1000; // ms
export const DEFAULT_GRAPH_RETENTION = 60 * 60 * 1000; // ms (1 hour)

/**
 * Get the graph long buffer configuration from localStorage
 */
export function getGluetexGraphConfig(): {
  sampleInterval: number;
  retention: number;
} {
  const storedInterval = localStorage.getItem(
    GLUETEX_GRAPH_SAMPLE_INTERVAL_KEY,
  );
  const storedRetention = localStorage.getItem(GLUETEX_GRAPH_RETENTION_KEY);

  return {
    sampleInterval: storedInterval
      ? Number(storedInterval)
      : DEFAULT_GRAPH_SAMPLE_INTERVAL,
    retention: storedRetention
      ? Number(storedRetention)
      : DEFAULT_GRAPH_RETENTION,
  };
}

/**
 * Set the graph long buffer configuration in localStorage
 */
export function setGluetexGraphConfig(
  sampleInterval: number,
  retention: number,
): void {
  localStorage.setItem(
    GLUETEX_GRAPH_SAMPLE_INTERVAL_KEY,
    sampleInterval.toString(),
  );
  localStorage.setItem(GLUETEX_GRAPH_RETENTION_KEY, retention.toString());
}

/**
 * Get the XL mode setting from localStorage
 * @returns true if XL mode is enabled, false otherwise
 */
export function getGluetexXLMode(): boolean {
  const stored = localStorage.getItem(GLUETEX_XL_MODE_KEY);
  return stored === "true";
}

/**
 * Set the XL mode setting in localStorage
 * @param enabled - true to enable XL mode, false to disable
 */
export function setGluetexXLMode(enabled: boolean): void {
  localStorage.setItem(GLUETEX_XL_MODE_KEY, enabled.toString());
}

/**
 * Get the maximum traverse limit based on XL mode
 * @returns the maximum traverse limit in mm
 */
export function getGluetexTraverseMax(): number {
  return getGluetexXLMode()
    ? GLUETEX_TRAVERSE_MAX_XL
    : GLUETEX_TRAVERSE_MAX_STANDARD;
}
