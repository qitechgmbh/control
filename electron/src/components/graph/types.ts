import type { Chart } from "chart.js";
import { IconName } from "@/components/Icon";
import { Unit } from "@/control/units";
import { TimeSeries } from "@/lib/timeseries";
import type { Marker } from "@/stores/markerStore";

export type SwitchOrigin = "button" | "gesture";

// Prop-based sync types
export type PropGraphSync = {
  timeWindow: number | "all";
  viewMode: "default" | "all" | "manual";
  isLiveMode: boolean;
  xRange?: { min: number; max: number };
  historicalFreezeTimestamp?: number | null;
  showFromTimestamp?: number | null;
  historicalSwitchOrigin: SwitchOrigin;
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
type BaseGraphLine = {
  color: string;
  width?: number;
  dash?: number[];
  show?: boolean;
};

type ThresholdLine = BaseGraphLine & {
  type: "threshold";
  value: number;
  label?: string;
};

type TargetLine = BaseGraphLine & {
  type: "target";
  value: number;
  label?: string;
  targetSeries?: TimeSeries;
};

type UserMarkerLine = BaseGraphLine & {
  type: "user_marker";
  value: number;
  label: string;
  markerTimestamp: number;
};

export type GraphLine = ThresholdLine | TargetLine | UserMarkerLine;

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

// Support for multiple series
export type SeriesData = {
  newData: TimeSeries | null;
  title?: string;
  color?: string;
  lines?: GraphLine[];
};

export type DataSeries = SeriesData | SeriesData[];

export type BigGraphProps = {
  newData: DataSeries;
  unit?: Unit;
  renderValue?: (
    value: number,
    seriesIndex?: number,
    seriesTitle?: string,
  ) => string;
  config: GraphConfig;
  graphId: string;
  syncGraph?: PropGraphSync;
  /** User-placed timeline markers, rendered as vertical annotations. */
  markers?: Marker[];
  /** Optional ref to receive the Chart.js instance when the chart is created (e.g. for marker timestamp tracking). */
  chartRefOut?: React.MutableRefObject<Chart | null>;
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
  onSwitchToHistorical: (origin: SwitchOrigin) => void;
  onExport?: () => void | Promise<void>;
  onAddMarker?: () => void;
  onManageMarkers?: () => void;
  timeWindowOptions?: TimeWindowOption[];
  showFromTimestamp?: number | null;
  onShowFromChange?: (timestamp: number | null) => void;
};
