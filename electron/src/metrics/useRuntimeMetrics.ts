import { useEffect, useRef, useState } from "react";

const API_BASE = "http://localhost:3001";
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
  rt_nr_switches: number | null;
  rt_nr_voluntary_switches: number | null;
  rt_nr_involuntary_switches: number | null;
};

/**
 * Polls the backend runtime metrics endpoint and computes:
 * - latest sample
 * - process CPU usage (%), derived from cpu_time_seconds deltas
 * - preemption rate (involuntary switches per second)
 */
export function useRuntimeMetrics(
  enabled: boolean,
  intervalMs: number = 5000,
) {
  const [sample, setSample] = useState<RuntimeMetricsSample | null>(null);
  const [cpuPercent, setCpuPercent] = useState<number | null>(null);
  const [preemptionRate, setPreemptionRate] = useState<number | null>(null);

  const lastRef = useRef<RuntimeMetricsSample | null>(null);

  useEffect(() => {
    if (!enabled) {
      setSample(null);
      setCpuPercent(null);
      setPreemptionRate(null);
      lastRef.current = null;
      return;
    }

    let cancelled = false;

    const fetchOnce = async () => {
      try {
        const res = await fetch(RUNTIME_URL);
        if (!res.ok) return;
        const json = (await res.json()) as RuntimeMetricsSample | null;
        if (!json || cancelled) return;

        const current = json;
        const last = lastRef.current;

        if (last) {
          const dtWallSec =
            (current.timestamp_ms - last.timestamp_ms) / 1000.0;
          if (dtWallSec > 0) {
            const dtCpu = current.cpu_time_seconds - last.cpu_time_seconds;
            setCpuPercent((dtCpu / dtWallSec) * 100.0);

            if (
              current.rt_nr_involuntary_switches != null &&
              last.rt_nr_involuntary_switches != null
            ) {
              const dtPreempt =
                current.rt_nr_involuntary_switches -
                last.rt_nr_involuntary_switches;
              setPreemptionRate(dtPreempt / dtWallSec);
            }
          }
        }

        lastRef.current = current;
        setSample(current);
      } catch (err) {
        console.error("Failed to fetch runtime metrics:", err);
      }
    };

    fetchOnce();
    const id = setInterval(fetchOnce, intervalMs);

    return () => {
      cancelled = true;
      clearInterval(id);
    };
  }, [enabled, intervalMs]);

  return { sample, cpuPercent, preemptionRate };
}

// Also export as default, in case some code uses default import
export default useRuntimeMetrics;