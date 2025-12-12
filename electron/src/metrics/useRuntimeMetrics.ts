import { useEffect, useState } from "react";
import { pushRuntimeSample } from "./runtimeSeries";

// Talk directly to the Rust API on port 3001.
// If you later proxy via Vite/Electron, make this configurable.
const API_BASE = "http://127.0.0.1:3001";
const RUNTIME_URL = `${API_BASE}/api/v1/metrics/runtime/latest`;

export type RuntimeMetricsSample = {
  timestamp_ms: number;
  rss_bytes: number;
  cpu_time_seconds: number;
  minor_faults: number;
  major_faults: number;
  jitter_min_ns: number;
  jitter_avg_ns: number;
  jitter_max_ns: number;
  rx_rate_bytes_per_sec: number;
  tx_rate_bytes_per_sec: number;
  rt_loop_cpu_time_seconds?: number | null;
  rt_nr_switches: number | null;
  rt_nr_voluntary_switches: number | null;
  rt_nr_involuntary_switches: number | null;
};

type MetricsState = {
  sample: RuntimeMetricsSample | null;
  cpuPercent: number | null;
  preemptionRate: number | null;
};

// ---- shared module-level state (global poller) ----

let currentState: MetricsState = {
  sample: null,
  cpuPercent: null,
  preemptionRate: null,
};

let lastSample: RuntimeMetricsSample | null = null;
let pollIntervalMs = 5000;
let pollTimer: ReturnType<typeof setInterval> | null = null;

const subscribers = new Set<(s: MetricsState) => void>();

function notifySubscribers() {
  for (const cb of subscribers) cb(currentState);
}

async function fetchOnce() {
  try {
    const res = await fetch(RUNTIME_URL);
    if (!res.ok) {
      console.error(
        "Runtime metrics fetch failed:",
        res.status,
        res.statusText,
      );
      return;
    }

    const json = (await res.json()) as RuntimeMetricsSample | null;
    if (!json) {
      // backend returned null (no sample yet)
      return;
    }

    const current = json;
    const last = lastSample;

    let cpuPercent: number | null = null;
    let preemptionRate: number | null = null;

    if (last) {
      const dtWallSec = (current.timestamp_ms - last.timestamp_ms) / 1000.0;
      if (dtWallSec > 0) {
        const dtCpu = current.cpu_time_seconds - last.cpu_time_seconds;
        cpuPercent = (dtCpu / dtWallSec) * 100.0;

        if (
          current.rt_nr_involuntary_switches != null &&
          last.rt_nr_involuntary_switches != null
        ) {
          const dtPreempt =
            current.rt_nr_involuntary_switches -
            last.rt_nr_involuntary_switches;
          preemptionRate = dtPreempt / dtWallSec;
        }
      }
    }

    lastSample = current;

    currentState = {
      sample: current,
      cpuPercent,
      preemptionRate,
    };

    // feed into shared time-series for graphs
    pushRuntimeSample(current, cpuPercent, preemptionRate);

    notifySubscribers();
  } catch (err) {
    console.error("Failed to fetch runtime metrics:", err);
  }
}

function startPolling() {
  if (pollTimer != null) return;
  void fetchOnce();
  pollTimer = setInterval(fetchOnce, pollIntervalMs);
}

/**
 * Shared runtime metrics hook.
 *
 * - One global poller per app (module-level).
 * - Polling continues even if no components are mounted.
 * - Components just subscribe to the latest state.
 */
export function useRuntimeMetrics(
  enabled: boolean,
  intervalMs: number = 5000,
): MetricsState {
  const [state, setState] = useState<MetricsState>(currentState);

  // Subscribe to shared state
  useEffect(() => {
    const cb = (s: MetricsState) => setState(s);
    subscribers.add(cb);
    cb(currentState);

    return () => {
      subscribers.delete(cb);
      // do NOT stop polling here; recording continues in background
    };
  }, []);

  // Control poller configuration
  useEffect(() => {
    pollIntervalMs = intervalMs;

    if (enabled) {
      if (pollTimer != null) {
        clearInterval(pollTimer);
        pollTimer = null;
      }
      startPolling();
    }
  }, [enabled, intervalMs]);

  return state;
}

export default useRuntimeMetrics;
