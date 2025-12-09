import React from "react";
import { Page } from "@/components/Page";
import {
  AutoSyncedBigGraph,
  SyncedFloatingControlPanel,
  useGraphSync,
  type GraphConfig,
} from "@/components/graph";
import { useRuntimeMetrics } from "./useRuntimeMetrics";
import { useRuntimeMetricSeries } from "./runtimeSeries";

export function MetricsGraphsPage() {
  // Start / keep the global poller alive
  useRuntimeMetrics(true, 5000);

  // Shared series (fed by global poller)
  const series = useRuntimeMetricSeries();

  const syncHook = useGraphSync("runtime-metrics");

  const jitterConfig: GraphConfig = {
    title: "Loop jitter (avg, µs)",
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

  return (
    <Page className="pb-27">
      <div className="flex flex-col gap-4">
        {series.jitter && (
          <AutoSyncedBigGraph
            syncHook={syncHook}
            newData={{
              newData: series.jitter,
              color: "#f97316",
            }}
            unit={undefined}
            renderValue={(v) => v.toFixed(1)}
            config={jitterConfig}
            graphId="runtime_jitter"
          />
        )}

        {series.cpu && (
          <AutoSyncedBigGraph
            syncHook={syncHook}
            newData={{
              newData: series.cpu,
              color: "#3b82f6",
            }}
            unit="%"
            renderValue={(v) => v.toFixed(1)}
            config={cpuConfig}
            graphId="runtime_cpu"
          />
        )}

        {series.mem && (
          <AutoSyncedBigGraph
            syncHook={syncHook}
            newData={{
              newData: series.mem,
              color: "#10b981",
            }}
            unit={undefined}
            renderValue={(v) => v.toFixed(1)}
            config={memConfig}
            graphId="runtime_memory"
          />
        )}

        {series.rx && series.tx && (
          <AutoSyncedBigGraph
            syncHook={syncHook}
            newData={[
              {
                newData: series.rx,
                title: "RX",
                color: "#10b981",
              },
              {
                newData: series.tx,
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

        {series.preempt && (
          <AutoSyncedBigGraph
            syncHook={syncHook}
            newData={{
              newData: series.preempt,
              color: "#6366f1",
            }}
            unit={undefined}
            renderValue={(v) => v.toFixed(1)}
            config={preemptConfig}
            graphId="runtime_preemptions"
          />
        )}
      </div>

      <SyncedFloatingControlPanel controlProps={syncHook.controlProps} />
    </Page>
  );
}
