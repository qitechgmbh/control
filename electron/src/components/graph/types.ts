import type { IChartApi, ISeriesApi } from "lightweight-charts";
import { IconName } from "@/components/Icon";
import { Unit } from "@/control/units";
import { TimeSeries } from "@/lib/timeseries";

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
  /** Optional ref to receive the lightweight-charts instance when the chart is created (e.g. for marker/target-line overlay positioning) */
  chartRefOut?: React.MutableRefObject<IChartApi | null>;
  /** Optional ref to receive the chart's mount element (overlays position themselves against it) */
  containerRefOut?: React.MutableRefObject<HTMLDivElement | null>;
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

export interface LiveModeHandlers {
  getCurrentLiveEndTimestamp: () => number;
  updateLiveData: () => void;
  handleLiveTimeWindow: (timeWindow: number | "all") => void;
  processNewHistoricalData: () => void;
}
export interface HistoricalModeHandlers {
  captureHistoricalFreezeTimestamp: () => number;
  getHistoricalEndTimestamp: () => number;
  handleHistoricalTimeWindow: (timeWindow: number | "all") => void;
  switchToHistoricalMode: (origin?: SwitchOrigin) => void;
  switchToLiveMode: () => void;
}

/** A config line that is actually rendered as its own (possibly invisible) LineSeries. */
export interface LineSeriesRef {
  api: ISeriesApi<"Line">;
  line: GraphLine;
  /** True for time-varying dashed target lines, drawn via the overlay instead of natively. */
  isOverlayDriven: boolean;
}

/** Chart-instance-scoped series handles, kept alongside the chart ref. */
export interface SeriesRefs {
  /** Index-aligned with getAllTimeSeries(newData). */
  dataSeries: ISeriesApi<"Line">[];
  /** One entry per visible config.lines entry, in order. */
  lineSeries: LineSeriesRef[];
}

export interface CreateChartParams {
  containerRef: React.RefObject<HTMLDivElement | null>;
  chartRef: React.RefObject<IChartApi | null>;
  /** Optional ref to expose the chart instance (e.g. for marker/target-line overlay positioning) */
  chartRefOut?: React.MutableRefObject<IChartApi | null>;
  seriesRefs: React.RefObject<SeriesRefs>;
  newData: BigGraphProps["newData"];
  config: BigGraphProps["config"];
  colors: {
    primary: string;
    grid: string;
    axis: string;
    background: string;
  };
  renderValue?: (value: number) => string;
  viewMode: "default" | "all" | "manual";
  selectedTimeWindow: number | "all";
  isLiveMode: boolean;
  startTimeRef: React.RefObject<number | null>;
  manualScaleRef: React.RefObject<{
    x: { min: number; max: number };
  } | null>;
  /** Set to true immediately before any programmatic setVisibleRange call, so the
   * range-change subscription can distinguish it from a user pan/zoom gesture. */
  suppressRangeEventRef: React.RefObject<boolean>;
  /** True only while a real pointer/wheel gesture is in progress on the chart.
   * The range-change subscription requires this before treating a range change
   * as user-driven, since resizes and other internal events can also fire it. */
  isUserInteractingRef: React.RefObject<boolean>;
  graphId: string;
  syncGraph?: BigGraphProps["syncGraph"];
  getHistoricalEndTimestamp: () => number;
  setViewMode: React.Dispatch<
    React.SetStateAction<"default" | "all" | "manual">
  >;
  setIsLiveMode: React.Dispatch<React.SetStateAction<boolean>>;
  setCursorValue: React.Dispatch<React.SetStateAction<number | null>>;
  setCursorValues: React.Dispatch<React.SetStateAction<(number | null)[]>>;
  visibleSeries: boolean[];
  showFromTimestamp?: number | null;
}
