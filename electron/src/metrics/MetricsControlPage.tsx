import React from "react";
import { Page } from "@/components/Page";
import { useRuntimeMetrics } from "./useRuntimeMetrics";
import { SectionTitle } from "@/components/SectionTitle";
import {
  renderValueToReactNode,
  renderUnitSymbol,
  getUnitIcon,
  type Unit,
} from "@/control/units";
import { Icon } from "@/components/Icon";

type MetricCardProps = {
  title: string;
  value: React.ReactNode;
  unit?: Unit;
};

function renderMetricWithUnit(
  value: number | null | undefined,
  unit: Unit,
  digits = 1,
) {
  if (value == null) {
    return "—";
  }

  return (
    <>
      {renderValueToReactNode(value, unit, (v) => v.toFixed(digits))}
      <span className="text-muted-foreground ml-1 text-xs">
        {renderUnitSymbol(unit)}
      </span>
    </>
  );
}

function MetricCard({ title, value, unit }: MetricCardProps) {
  const iconName = unit ? getUnitIcon(unit) : "lu:StickyNote";

  return (
    <div className="flex flex-col gap-4 rounded-lg border border-slate-200 bg-white p-4 shadow-sm">
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-2">
          {iconName && (
            <span className="inline-flex h-7 w-7 items-center justify-center rounded-full bg-slate-100 text-slate-500">
              <Icon name={iconName} className="h-4 w-4" />
            </span>
          )}
          <h2 className="text-sm font-medium text-slate-700">{title}</h2>
        </div>
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
        <SectionTitle title="Backend Metrics" />
        <div className="grid gap-6 md:grid-cols-2">
          {/* Loop jitter: max, in µs */}
          <MetricCard
            title="Max Loop Jitter"
            unit={"µs"}
            value={renderMetricWithUnit(jitterMaxUs, "µs")}
          />

          {/* CPU usage */}
          <MetricCard
            title="CPU Usage"
            unit={"%"}
            value={renderMetricWithUnit(cpuPercent, "%")}
          />

          {/* CPU time */}
          <MetricCard
            title="CPU Time"
            unit={"s"}
            value={renderMetricWithUnit(cpuTimeSec, "s")}
          />

          {/* Memory usage */}
          <MetricCard
            title="Memory Usage"
            unit={"MiB"}
            value={renderMetricWithUnit(rssMb, "MiB")}
          />

          {/* Total page faults (composite, no single unit) */}
          <MetricCard
            title="Page Faults"
            value={
              minorFaults != null || majorFaults != null
                ? `${minorFaults ?? "–"} minor per second | ${majorFaults ?? "–"} major`
                : "—"
            }
          />

          {/* EtherCAT throughput (RX | TX) */}
          <MetricCard
            title="EtherCAT Throughput (RX | TX)"
            unit={"Mbit/s"}
            value={
              rxMbit != null && txMbit != null ? (
                <span className="flex items-center gap-1">
                  {renderMetricWithUnit(rxMbit, "Mbit/s")}
                  <span className="text-muted-foreground mx-1 text-xs">|</span>
                  {renderMetricWithUnit(txMbit, "Mbit/s")}
                </span>
              ) : (
                "—"
              )
            }
          />

          {/* Preemptions per second */}
          <MetricCard
            title="Preemptions"
            unit={"/s"}
            value={
              preemptionRate != null
                ? renderMetricWithUnit(preemptionRate, "/s")
                : "—"
            }
          />
        </div>
      </div>
    </Page>
  );
}
