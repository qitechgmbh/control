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
import { getUnitIcon, type Unit } from "@/control/units";

export function MetricsGraphsPage() {
  // Start / keep the global poller alive
  useRuntimeMetrics(true, 5000);

  // Shared series (fed by global poller)
  const series = useRuntimeMetricSeries();

  const syncHook = useGraphSync("runtime-metrics");

  const jitterUnit: Unit = "µs";
  const cpuUnit: Unit = "%";
  const memUnit: Unit = "MiB";
  const ioUnit: Unit = "Mbit/s";
  const preemptUnit: Unit = "/s";

  const jitterConfig: GraphConfig = {
    title: "Average Loop Jitter",
    icon: getUnitIcon(jitterUnit),
    colors: {
      primary: "#f97316",
      grid: "#e2e8f0",
      background: "#ffffff",
    },
    exportFilename: "runtime_jitter",
  };

  const cpuConfig: GraphConfig = {
    title: "CPU Usage",
    icon: getUnitIcon(cpuUnit),
    colors: {
      primary: "#3b82f6",
      grid: "#e2e8f0",
      background: "#ffffff",
    },
    exportFilename: "runtime_cpu",
  };

  const memConfig: GraphConfig = {
    title: "Memory Usage",
    icon: getUnitIcon(memUnit),
    colors: {
      primary: "#10b981",
      grid: "#e2e8f0",
      background: "#ffffff",
    },
    exportFilename: "runtime_memory",
  };

  const ioConfig: GraphConfig = {
    title: "EtherCAT Throughput",
    icon: getUnitIcon(ioUnit),
    colors: {
      primary: "#10b981",
      grid: "#e2e8f0",
      background: "#ffffff",
    },
    exportFilename: "runtime_io",
    showLegend: true,
  };

  const preemptConfig: GraphConfig = {
    title: "Preemptions Per Second",
    icon: getUnitIcon(preemptUnit),
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
            unit={jitterUnit}
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
            unit={cpuUnit}
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
            unit={memUnit}
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
            unit={ioUnit}
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
            unit={preemptUnit}
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
