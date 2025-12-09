import React, { useEffect, useState } from "react";
import { Page } from "@/components/Page";
import {
  AutoSyncedBigGraph,
  SyncedFloatingControlPanel,
  useGraphSync,
  type GraphConfig,
} from "@/components/graph";
import { createTimeSeries, TimeSeries } from "@/lib/timeseries";
import { useMetricsSettings } from "./useMetricSettings";
import { useRuntimeMetrics } from "./useRuntimeMetrics";

const ONE_SECOND = 1000;
const FIVE_SECOND = 5 * ONE_SECOND;
const ONE_HOUR = 60 * 60 * ONE_SECOND;

// Per-graph TimeSeries factories
const { initialTimeSeries: jitterInitial, insert: insertJitter } =
  createTimeSeries(FIVE_SECOND, ONE_SECOND, FIVE_SECOND, ONE_HOUR);
const { initialTimeSeries: cpuInitial, insert: insertCpu } =
  createTimeSeries(FIVE_SECOND, ONE_SECOND, FIVE_SECOND, ONE_HOUR);
const { initialTimeSeries: memInitial, insert: insertMem } =
  createTimeSeries(FIVE_SECOND, ONE_SECOND, FIVE_SECOND, ONE_HOUR);
const { initialTimeSeries: rxInitial, insert: insertRx } =
  createTimeSeries(FIVE_SECOND, ONE_SECOND, FIVE_SECOND, ONE_HOUR);
const { initialTimeSeries: txInitial, insert: insertTx } =
  createTimeSeries(FIVE_SECOND, ONE_SECOND, FIVE_SECOND, ONE_HOUR);
const {
  initialTimeSeries: preemptInitial,
  insert: insertPreempt,
} = createTimeSeries(FIVE_SECOND, ONE_SECOND, FIVE_SECOND, ONE_HOUR);

export function MetricsGraphsPage() {
  const { settings, anyEnabled } = useMetricsSettings();

  // Always keep polling running; this feeds this page's time-series.
  const { sample, cpuPercent, preemptionRate } =
    useRuntimeMetrics(true, 5000);

  const syncHook = useGraphSync("runtime-metrics");

  // Use relative time so X axis is small (order of seconds)
  const [baseTimestamp, setBaseTimestamp] = useState<number | null>(null);

  const [jitterSeries, setJitterSeries] =
    useState<TimeSeries | null>(jitterInitial);
  const [cpuSeries, setCpuSeries] =
    useState<TimeSeries | null>(cpuInitial);
  const [memSeries, setMemSeries] =
    useState<TimeSeries | null>(memInitial);
  const [rxSeries, setRxSeries] = useState<TimeSeries | null>(rxInitial);
  const [txSeries, setTxSeries] = useState<TimeSeries | null>(txInitial);
  const [preemptSeries, setPreemptSeries] =
    useState<TimeSeries | null>(preemptInitial);

  useEffect(() => {
    if (!sample) return;

    if (baseTimestamp === null) {
      setBaseTimestamp(sample.timestamp_ms);
      return;
    }

    // ms since first sample; window is 5 s
    const ts = sample.timestamp_ms - baseTimestamp;

    if (settings.showJitter) {
      setJitterSeries((prev) =>
        insertJitter(prev ?? jitterInitial, {
          value: sample.jitter_max_ns / 1e3, // µs
          timestamp: ts,
        }),
      );
    }

    if (settings.showCpu && cpuPercent != null) {
      setCpuSeries((prev) =>
        insertCpu(prev ?? cpuInitial, {
          value: cpuPercent,
          timestamp: ts,
        }),
      );
    }

    if (settings.showMemory) {
      const rssMb = sample.rss_bytes / (1024 * 1024);
      setMemSeries((prev) =>
        insertMem(prev ?? memInitial, {
          value: rssMb,
          timestamp: ts,
        }),
      );
    }

    if (settings.showIo) {
      const rxMbit =
        (sample.rx_rate_bytes_per_sec * 8.0) / 1_000_000.0;
      const txMbit =
        (sample.tx_rate_bytes_per_sec * 8.0) / 1_000_000.0;

      setRxSeries((prev) =>
        insertRx(prev ?? rxInitial, {
          value: rxMbit,
          timestamp: ts,
        }),
      );
      setTxSeries((prev) =>
        insertTx(prev ?? txInitial, {
          value: txMbit,
          timestamp: ts,
        }),
      );
    }

    if (settings.showPreemption && preemptionRate != null) {
      setPreemptSeries((prev) =>
        insertPreempt(prev ?? preemptInitial, {
          value: preemptionRate, // 1/s
          timestamp: ts,
        }),
      );
    }
  }, [
    sample,
    cpuPercent,
    preemptionRate,
    settings.showJitter,
    settings.showCpu,
    settings.showMemory,
    settings.showIo,
    settings.showPreemption,
    baseTimestamp,
  ]);

  const jitterConfig: GraphConfig = {
    title: "Loop jitter (max, µs)",
    icon: "lu:Activity",
    colors: {
      primary: "#f97316",
      grid: "#e2e8f0",
      background: "#ffffff",
    },
    exportFilename: "runtime_jitter",
  };

  const cpuConfig: GraphConfig = {
    title: "CPU usage (%)",
    icon: "lu:Cpu",
    colors: {
      primary: "#3b82f6",
      grid: "#e2e8f0",
      background: "#ffffff",
    },
    exportFilename: "runtime_cpu",
  };

  const memConfig: GraphConfig = {
    title: "Memory usage (MiB)",
    icon: "lu:MemoryStick",
    colors: {
      primary: "#10b981",
      grid: "#e2e8f0",
      background: "#ffffff",
    },
    exportFilename: "runtime_memory",
  };

  const ioConfig: GraphConfig = {
    title: "EtherCAT throughput (Mbit/s)",
    icon: "lu:Network",
    colors: {
      primary: "#10b981",
      grid: "#e2e8f0",
      background: "#ffffff",
    },
    exportFilename: "runtime_io",
    showLegend: true,
  };

  const preemptConfig: GraphConfig = {
    title: "Preemptions per second (1/s)",
    icon: "lu:Timer",
    colors: {
      primary: "#6366f1",
      grid: "#e2e8f0",
      background: "#ffffff",
    },
    exportFilename: "runtime_preemptions",
  };

  const nothingEnabled = !anyEnabled;

  return (
    <Page className="pb-27">
      {nothingEnabled ? (
        <div className="text-sm text-slate-600">
          All metrics are hidden. Enable metrics on the{" "}
          <strong>Metrics → Control</strong> page.
        </div>
      ) : (
        <>
          <div className="flex flex-col gap-4">
            {settings.showJitter && jitterSeries && (
              <AutoSyncedBigGraph
                syncHook={syncHook}
                newData={{
                  newData: jitterSeries,
                  color: "#f97316",
                }}
                unit={undefined}
                renderValue={(v) => v.toFixed(1)}
                config={jitterConfig}
                graphId="runtime_jitter"
              />
            )}

            {settings.showCpu && cpuSeries && (
              <AutoSyncedBigGraph
                syncHook={syncHook}
                newData={{
                  newData: cpuSeries,
                  color: "#3b82f6",
                }}
                unit="%"
                renderValue={(v) => v.toFixed(1)}
                config={cpuConfig}
                graphId="runtime_cpu"
              />
            )}

            {settings.showMemory && memSeries && (
              <AutoSyncedBigGraph
                syncHook={syncHook}
                newData={{
                  newData: memSeries,
                  color: "#10b981",
                }}
                unit={undefined}
                renderValue={(v) => v.toFixed(1)}
                config={memConfig}
                graphId="runtime_memory"
              />
            )}

            {settings.showIo && rxSeries && txSeries && (
              <AutoSyncedBigGraph
                syncHook={syncHook}
                newData={[
                  {
                    newData: rxSeries,
                    title: "RX",
                    color: "#10b981",
                  },
                  {
                    newData: txSeries,
                    title: "TX",
                    color: "#ef4444",
                  },
                ]}
                unit={undefined}
                renderValue={(v) => v.toFixed(2)}
                config={ioConfig}
                graphId="runtime_io"
              />
            )}

            {settings.showPreemption && preemptSeries && (
              <AutoSyncedBigGraph
                syncHook={syncHook}
                newData={{
                  newData: preemptSeries,
                  color: "#6366f1",
                }}
                unit={undefined}
                renderValue={(v) => v.toFixed(1)}
                config={preemptConfig}
                graphId="runtime_preemptions"
              />
            )}
          </div>

          <SyncedFloatingControlPanel
            controlProps={syncHook.controlProps}
          />
        </>
      )}
    </Page>
  );
}