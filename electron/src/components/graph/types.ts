import { IconName } from "@/components/Icon";
import { Unit } from "@/control/units";
import { TimeSeries } from "@/lib/timeseries";

// Prop-based sync types
export type PropGraphSync = {
  timeWindow: number | "all";
  viewMode: "default" | "all" | "manual";
  isLiveMode: boolean;
  xRange?: { min: number; max: number };
  onTimeWindowChange?: (graphId: string, timeWindow: number | "all") => void;
  onViewModeChange?: (
    graphId: string,
    viewMode: "default" | "all" | "manual",
    isLiveMode: boolean,
  ) => void;
  onZoomChange?: (
    graphId: string,
    xRange: { min: number; max: number },
  ) => void;
};

// Configuration types for additional lines
export type GraphLine = {
  type: "threshold" | "target" | "reference";
  value: number;
  label: string;
  color: string;
  width?: number;
  dash?: number[];
  show?: boolean;
};

export type GraphConfig = {
  title: string;
  icon?: IconName;
  lines?: GraphLine[];
  timeWindows?: Array<{ value: number | "all"; label: string }>;
  defaultTimeWindow?: number | "all";
  exportFilename?: string;
  showLegend?: boolean;
  colors?: {
    primary?: string;
    grid?: string;
    axis?: string;
    background?: string;
  };
};

export type BigGraphProps = {
  newData: TimeSeries | null;
  unit?: Unit;
  renderValue?: (value: number) => string;
  config: GraphConfig;
  graphId: string;
  syncGraph?: PropGraphSync;
};

export type TimeWindowOption = {
  value: number | "all";
  label: string;
};

export type ControlProps = {
  timeWindow: number | "all";
  isLiveMode: boolean;
  onTimeWindowChange: (timeWindow: number | "all") => void;
  onSwitchToLive: () => void;
  onSwitchToHistorical: () => void;
  onExport?: () => void;
  timeWindowOptions?: TimeWindowOption[];
};
