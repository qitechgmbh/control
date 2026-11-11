import { TimeWindowOption } from "./types";

export const DEFAULT_TIME_WINDOW_OPTIONS: TimeWindowOption[] = [
  { value: 10 * 1000, label: "10s" },
  { value: 30 * 1000, label: "30s" },
  { value: 1 * 60 * 1000, label: "1m" },
  { value: 5 * 60 * 1000, label: "5m" },
  { value: 10 * 60 * 1000, label: "10m" },
  { value: 30 * 60 * 1000, label: "30m" },
  { value: 1 * 60 * 60 * 1000, label: "1h" },
  { value: 6 * 60 * 60 * 1000, label: "6h" },
  { value: 24 * 60 * 60 * 1000, label: "24h" },
  { value: 7 * 24 * 60 * 60 * 1000, label: "7d" },
  { value: 30 * 24 * 60 * 60 * 1000, label: "30d" },
  { value: "all" as const, label: "Show All" },
];

export const DEFAULT_COLORS = {
  primary: "#3b82f6",
  grid: "#e2e8f0",
  axis: "#64748b",
  background: "#ffffff",
};
