import React from "react";
import { Page } from "@/components/Page";
import { useRuntimeMetrics } from "./useRuntimeMetrics";

type MetricCardProps = {
  title: string;
  value: string;
};

function MetricCard({ title, value }: MetricCardProps) {
  return (
    <div className="flex flex-col gap-4 rounded-lg border border-slate-200 bg-white p-4 shadow-sm">
      <div className="flex items-center justify-between">
        <h2 className="text-sm font-medium text-slate-700">{title}</h2>
        <span className="text-base font-semibold text-slate-900">{value}</span>
      </div>
    </div>
  );
}

export function MetricsControlPage() {
  // Global poller: immediate first fetch, then every 5 seconds
  const { sample, cpuPercent, preemptionRate } = useRuntimeMetrics(true, 5000);

  const rssMb = sample != null ? sample.rss_bytes / (1024 * 1024) : null;

  // jitter_*_ns is nanoseconds; divide by 1e3 to get microseconds (µs)
  const jitterMaxUs = sample != null ? sample.jitter_max_ns / 1e3 : null;

  const rxMbit =
    sample != null ? (sample.rx_rate_bytes_per_sec * 8.0) / 1_000_000.0 : null;
  const txMbit =
    sample != null ? (sample.tx_rate_bytes_per_sec * 8.0) / 1_000_000.0 : null;

  const cpuTimeSec = sample != null ? sample.cpu_time_seconds : null;
  const minorFaults = sample != null ? sample.minor_faults : null;
  const majorFaults = sample != null ? sample.major_faults : null;

  return (
    <Page>
      <div className="space-y-6">
        <p className="text-sm text-slate-600">
          Runtime metrics are refreshed immediately once and then every 5
          seconds.
        </p>

        <div className="grid gap-6 md:grid-cols-2">
          <MetricCard
            title="Loop jitter (max, µs)"
            value={jitterMaxUs != null ? `${jitterMaxUs.toFixed(1)} µs` : "—"}
          />

          <MetricCard
            title="CPU usage / time"
            value={
              cpuTimeSec != null && cpuPercent != null
                ? `${cpuPercent.toFixed(1)} % • ${cpuTimeSec.toFixed(1)} s`
                : cpuTimeSec != null
                  ? `${cpuTimeSec.toFixed(1)} s`
                  : "—"
            }
          />

          <MetricCard
            title="Memory usage"
            value={rssMb != null ? `${rssMb.toFixed(1)} MiB` : "—"}
          />

          <MetricCard
            title="Page faults (total)"
            value={
              minorFaults != null || majorFaults != null
                ? `${minorFaults ?? "–"} minor / ${majorFaults ?? "–"} major`
                : "—"
            }
          />

          <MetricCard
            title="EtherCAT throughput (RX / TX)"
            value={
              rxMbit != null && txMbit != null
                ? `${rxMbit.toFixed(2)} / ${txMbit.toFixed(2)} Mbit/s`
                : "—"
            }
          />

          <MetricCard
            title="Preemptions"
            value={
              preemptionRate != null ? `${preemptionRate.toFixed(1)} /s` : "—"
            }
          />
        </div>
      </div>
    </Page>
  );
}
