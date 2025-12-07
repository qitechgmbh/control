import { IconName } from "@/components/Icon";
import { Unit } from "@/control/units";
import { TimeSeries } from "@/lib/timeseries";
import { RefObject } from "react";

// Prop-based sync types
export type PropGraphSync = {
  timeWindow: number | "all";
  viewMode: "default" | "all" | "manual";
  isLiveMode: boolean;
  xRange?: { min: number; max: number };
  historicalFreezeTimestamp?: number | null;
  showFromTimestamp?: number | null;
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
  type: "threshold" | "target";
  value: number;
  color: string;
  label?: string;
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
  switchToHistoricalMode: () => void;
  switchToLiveMode: () => void;
}

export interface CreateChartParams {
  containerRef: React.RefObject<HTMLDivElement | null>;
  uplotRef: React.RefObject<uPlot | null>;
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
    y: { min: number; max: number };
  } | null>;
  animationRefs: AnimationRefs;
  handlerRefs: HandlerRefs;
  graphId: string;
  syncGraph?: BigGraphProps["syncGraph"];
  getHistoricalEndTimestamp: () => number;
  updateYAxisScale: (xMin?: number, xMax?: number) => void;
  setViewMode: React.Dispatch<
    React.SetStateAction<"default" | "all" | "manual">
  >;
  setIsLiveMode: React.Dispatch<React.SetStateAction<boolean>>;
  setCursorValue: React.Dispatch<React.SetStateAction<number | null>>;
  setCursorValues: React.Dispatch<React.SetStateAction<(number | null)[]>>;
  visibleSeries: boolean[];
  showFromTimestamp?: number | null;
}

export interface AnimationState {
  isAnimating: boolean;
  startTime: number;
  fromValue: number;
  toValue: number;
  fromTimestamp: number;
  toTimestamp: number;
  targetIndex: number;
}

export interface AnimationRefs {
  animationFrame: React.RefObject<number | null>;
  animationState: React.RefObject<AnimationState>;
  lastRenderedData: React.RefObject<{
    timestamps: number[];
    values: number[];
  }>;
  realPointsCount: React.RefObject<number>;
}

export interface HandlerRefs {
  isUserZoomingRef: RefObject<boolean>;
  isDraggingRef: RefObject<boolean>;
  lastDragXRef: RefObject<number | null>;
  isPinchingRef: RefObject<boolean>;
  lastPinchDistanceRef: RefObject<number | null>;
  pinchCenterRef: RefObject<{ x: number; y: number } | null>;
  touchStartRef: RefObject<{
    x: number;
    y: number;
    time: number;
  } | null>;
  touchDirectionRef: RefObject<"horizontal" | "vertical" | "unknown">;
}
