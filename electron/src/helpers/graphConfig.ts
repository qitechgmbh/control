import { IconName } from "@/components/Icon";
import { GraphLine } from "./BigGraph";

export type GraphConfig = {
  title?: string;
  icon?: IconName;
  lines?: GraphLine[];
  timeWindows?: Array<{ value: number | "all"; label: string }>;
  defaultTimeWindow?: number | "all";
  exportFilename?: string;
  colors?: {
    primary?: string;
    grid?: string;
    axis?: string;
    background?: string;
  };
};

type GraphConfigInternal = {
  title: string;
  icon?: IconName;
  lines: GraphLine[];
  timeWindows: Array<{ value: number | "all"; label: string }>;
  defaultTimeWindow: number | "all";
  exportFilename?: string;
  colors: {
    primary: string;
    grid: string;
    axis: string;
    background: string;
  };
};

export function populateConfigDefaults(
  config: GraphConfig,
): GraphConfigInternal {
  return {
    title: config.title ?? "Graph",
    icon: config.icon ?? "lu:ChartSpline",
    lines: config.lines ?? [],
    timeWindows: config.timeWindows ?? DEFAULT_TIME_WINDOW_OPTIONS,
    defaultTimeWindow: config.defaultTimeWindow ?? 30 * 60 * 1000,
    exportFilename: config.exportFilename,
    colors: {
      primary: config.colors?.primary ?? "#3b82f6",
      grid: config.colors?.grid ?? "#e2e8f0",
      axis: config.colors?.axis ?? "#64748b",
      background: config.colors?.background ?? "#ffffff",
    },
  };
}

// Default time window options with "Show All" included
const DEFAULT_TIME_WINDOW_OPTIONS = [
  { value: 10 * 1000, label: "10s" },
  { value: 30 * 1000, label: "30s" },
  { value: 1 * 60 * 1000, label: "1m" },
  { value: 5 * 60 * 1000, label: "5m" },
  { value: 10 * 60 * 1000, label: "10m" },
  { value: 30 * 60 * 1000, label: "30m" },
  { value: 1 * 60 * 60 * 1000, label: "1h" },
  { value: "all" as const, label: "Show All" },
];
