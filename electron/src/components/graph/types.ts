import type { Chart } from "chart.js";
import type { Marker } from "@/stores/markerStore";
import { IconName } from "@/components/Icon";
import { Unit } from "@/control/units";
import { TimeSeries } from "@/lib/timeseries";
import { ExportResultDialogProps } from "@/components/ExportResultDialog";

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
  /** Charts with the same group share their hover crosshair. */
  cursorGroup?: string;
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
  /** Config property path whose history is loaded from the database, e.g. `screw.target_rpm`. */
  history?: string;
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
  /** Measurement path whose history is loaded from the database, e.g. `motor.rpm`. */
  history?: string;
  /** Factor applied to database values when the UI shows a converted unit (e.g. 100 for %). */
  historyScale?: number;
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
  /** Receives the Chart.js instance when it is created, and null when destroyed */
  onChart?: (chart: Chart | null) => void;
  /** User markers drawn as vertical lines */
  markers?: Marker[];
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
  exportDialogProps?: ExportResultDialogProps;
  onAddMarker?: () => void;
  onManageMarkers?: () => void;
  timeWindowOptions?: TimeWindowOption[];
  showFromTimestamp?: number | null;
  onShowFromChange?: (timestamp: number | null) => void;
};
