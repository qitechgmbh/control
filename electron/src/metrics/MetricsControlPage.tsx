import React from "react";
import { Page } from "@/components/Page";
import { SelectionGroup } from "@/control/SelectionGroup";

import { useMetricsSettings } from "./useMetricSettings";
import { useRuntimeMetrics } from "./useRuntimeMetrics";

type MetricCardProps = {
  title: string;
  value: string;
  enabled: boolean;
  onToggle: (on: boolean) => void;
};

function MetricCard({ title, value, enabled, onToggle }: MetricCardProps) {
  return (
    <div className="flex flex-col gap-4 rounded-lg border border-slate-200 bg-white p-4 shadow-sm">
      <div className="flex items-center justify-between">
        <h2 className="text-sm font-medium text-slate-700">{title}</h2>
        <span className="text-base font-semibold text-slate-900">
          {value}
        </span>
      </div>

      <div className="flex justify-end">
        <SelectionGroup<"On" | "Off">
          value={enabled ? "On" : "Off"}
          orientation="horizontal"
          className="grid grid-cols-2 gap-2"
          options={{
            Off: {
              children: "Off",
              icon: "lu:CirclePause",
              // OFF = red
              isActiveClassName: "bg-red-600",
              className: "h-full",
            },
            On: {
              children: "On",
              icon: "lu:CirclePlay",
              // ON = green
              isActiveClassName: "bg-green-600",
              className: "h-full",
            },
          }}
          onChange={(val) => onToggle(val === "On")}
        />
      </div>
    </div>
  );
}

export function MetricsControlPage() {
  const { settings, updateSetting } = useMetricsSettings();

  // Always poll; toggles only control what is *shown*, not polling itself.
  const { sample, cpuPercent, preemptionRate } =
    useRuntimeMetrics(true, 5000);

  const rssMb =
    sample != null ? sample.rss_bytes / (1024 * 1024) : null;
  const jitterMaxUs =
    sample != null ? sample.jitter_max_ns / 1e3 : null;
  const rxMbit =
    sample != null
      ? (sample.rx_rate_bytes_per_sec * 8.0) / 1_000_000.0
      : null;
  const txMbit =
    sample != null
      ? (sample.tx_rate_bytes_per_sec * 8.0) / 1_000_000.0
      : null;
  const cpuTimeSec = sample != null ? sample.cpu_time_seconds : null;
  const minorFaults = sample != null ? sample.minor_faults : null;
  const majorFaults = sample != null ? sample.major_faults : null;

  return (
    <Page>
      <div className="space-y-6">
        <p className="text-sm text-slate-600">
          Enable only the metrics you need visually. Values are refreshed
          every 5 seconds.
        </p>

        <div className="grid gap-6 md:grid-cols-2">
          <MetricCard
            title="Loop jitter (max)"
            value={
              jitterMaxUs != null
                ? `${jitterMaxUs.toFixed(1)} µs`
                : "—"
            }
            enabled={settings.showJitter}
            onToggle={(on) => updateSetting("showJitter", on)}
          />

          <MetricCard
            title="CPU usage / time"
            value={
              cpuPercent != null && cpuTimeSec != null
                ? `${cpuPercent.toFixed(1)} % • ${cpuTimeSec.toFixed(
                    1,
                  )} s`
                : cpuTimeSec != null
                  ? `${cpuTimeSec.toFixed(1)} s`
                  : "—"
            }
            enabled={settings.showCpu}
            onToggle={(on) => updateSetting("showCpu", on)}
          />

          <MetricCard
            title="Memory usage / page faults"
            value={
              rssMb != null
                ? `${rssMb.toFixed(1)} MiB • ${
                    minorFaults ?? "–"
                  } minor / ${majorFaults ?? "–"} major`
                : "—"
            }
            enabled={settings.showMemory}
            onToggle={(on) => updateSetting("showMemory", on)}
          />

          <MetricCard
            title="EtherCAT throughput (RX / TX)"
            value={
              rxMbit != null && txMbit != null
                ? `${rxMbit.toFixed(2)} / ${txMbit.toFixed(
                    2,
                  )} Mbit/s`
                : "—"
            }
            enabled={settings.showIo}
            onToggle={(on) => updateSetting("showIo", on)}
          />

          <MetricCard
            title="Preemptions"
            value={
              preemptionRate != null
                ? `${preemptionRate.toFixed(1)} /s`
                : "—"
            }
            enabled={settings.showPreemption}
            onToggle={(on) => updateSetting("showPreemption", on)}
          />
        </div>
      </div>
    </Page>
  );
}