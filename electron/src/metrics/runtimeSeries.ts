import { useEffect, useState } from "react";
import { createTimeSeries, TimeSeries } from "@/lib/timeseries";
import type { RuntimeMetricsSample } from "./useRuntimeMetrics";

const ONE_SECOND = 1000;
const FIVE_SECOND = 5 * ONE_SECOND;

// 5 s window, 1 s bucket, 1 h history
const { initialTimeSeries: jitterInitial, insert: insertJitter } =
  createTimeSeries({ sampleIntervalShort: FIVE_SECOND });
const { initialTimeSeries: cpuInitial, insert: insertCpu } = createTimeSeries({
  sampleIntervalShort: FIVE_SECOND,
});
const { initialTimeSeries: memInitial, insert: insertMem } = createTimeSeries({
  sampleIntervalShort: FIVE_SECOND,
});
const { initialTimeSeries: rxInitial, insert: insertRx } = createTimeSeries({
  sampleIntervalShort: FIVE_SECOND,
});
const { initialTimeSeries: txInitial, insert: insertTx } = createTimeSeries({
  sampleIntervalShort: FIVE_SECOND,
});
const { initialTimeSeries: preemptInitial, insert: insertPreempt } =
  createTimeSeries({ sampleIntervalShort: FIVE_SECOND });

export type RuntimeSeriesState = {
  jitter: TimeSeries | null;
  cpu: TimeSeries | null;
  mem: TimeSeries | null;
  rx: TimeSeries | null;
  tx: TimeSeries | null;
  preempt: TimeSeries | null;
};

let baseTimestampMs: number | null = null;

let currentSeries: RuntimeSeriesState = {
  jitter: jitterInitial,
  cpu: cpuInitial,
  mem: memInitial,
  rx: rxInitial,
  tx: txInitial,
  preempt: preemptInitial,
};

const subscribers = new Set<(s: RuntimeSeriesState) => void>();

function notifySubscribers() {
  for (const cb of subscribers) cb(currentSeries);
}

/**
 * Push one sample into the shared time-series.
 * Always records; UI toggles only control what is displayed.
 */
export function pushRuntimeSample(
  sample: RuntimeMetricsSample,
  cpuPercent: number | null,
  preemptionRate: number | null,
) {
  if (baseTimestampMs === null) {
    baseTimestampMs = sample.timestamp_ms;
  }
  const t = sample.timestamp_ms - baseTimestampMs; // ms since first sample

  let { jitter, cpu, mem, rx, tx, preempt } = currentSeries;

  // avg jitter in µs
  jitter = insertJitter(jitter ?? jitterInitial, {
    value: sample.jitter_avg_ns / 1e3,
    timestamp: t,
  });

  if (cpuPercent != null) {
    cpu = insertCpu(cpu ?? cpuInitial, {
      value: cpuPercent,
      timestamp: t,
    });
  }

  const rssMb = sample.rss_bytes / (1024 * 1024);
  mem = insertMem(mem ?? memInitial, {
    value: rssMb,
    timestamp: t,
  });

  const rxMbit = (sample.rx_rate_bytes_per_sec * 8.0) / 1_000_000.0;
  const txMbit = (sample.tx_rate_bytes_per_sec * 8.0) / 1_000_000.0;

  rx = insertRx(rx ?? rxInitial, {
    value: rxMbit,
    timestamp: t,
  });
  tx = insertTx(tx ?? txInitial, {
    value: txMbit,
    timestamp: t,
  });

  if (preemptionRate != null) {
    preempt = insertPreempt(preempt ?? preemptInitial, {
      value: preemptionRate,
      timestamp: t,
    });
  }

  currentSeries = { jitter, cpu, mem, rx, tx, preempt };
  notifySubscribers();
}
/**
 * Read shared time-series; history survives unmount/remount.
 */
export function useRuntimeMetricSeries(): RuntimeSeriesState {
  const [state, setState] = useState<RuntimeSeriesState>(currentSeries);

  useEffect(() => {
    const cb = (s: RuntimeSeriesState) => setState(s);
    subscribers.add(cb);
    cb(currentSeries);
    return () => {
      subscribers.delete(cb);
    };
  }, []);

  return state;
}
