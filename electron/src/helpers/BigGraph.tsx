import React, {
  useEffect,
  useRef,
  useState,
  useCallback,
  createContext,
  useContext,
  ReactNode,
} from "react";
import uPlot from "uplot";
import "uplot/dist/uPlot.min.css";
import { TimeSeries, seriesToUPlotData } from "@/lib/timeseries";
import { renderUnitSymbol, Unit, getUnitIcon } from "@/control/units";
import { TouchButton } from "@/components/touch/TouchButton";
import { Icon, IconName } from "@/components/Icon";
import { ControlCard } from "@/control/ControlCard";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { GraphExportData, exportGraphsToExcel } from "./excel_helpers";

// Sync Context for synchronized zooming (kept for backward compatibility)
type SyncState = {
  x: { min: number; max: number };
  timeWindow?: number | "all";
  viewMode?: "default" | "all" | "manual";
  isLiveMode?: boolean;
};

type GraphSyncContextType = {
  registerGraph: (id: string, updateFn: (state: SyncState) => void) => void;
  unregisterGraph: (id: string) => void;
  syncZoom: (fromId: string, state: SyncState) => void;
  syncTimeWindow: (fromId: string, timeWindow: number | "all") => void;
  syncViewMode: (
    fromId: string,
    viewMode: "default" | "all" | "manual",
    isLiveMode: boolean,
  ) => void;
  registerGraphData: (
    id: string,
    getDataFn: () => GraphExportData | null,
  ) => void;
  unregisterGraphData: (id: string) => void;
  exportAllGraphs: () => void;
  hasGraphs: () => boolean;
  hasRegisteredGraphs: boolean;
  switchToLiveMode: () => void;
  switchToHistoricalMode: () => void;
  handleTimeWindowChange: (timeWindow: number | "all") => void;
  getCurrentTimeWindow: () => number | "all";
  getIsLiveMode: () => boolean;
  getTimeWindowOptions: () => Array<{ value: number | "all"; label: string }>;
  showControls: boolean;
  setShowControls: (show: boolean) => void;
};

const GraphSyncContext = createContext<GraphSyncContextType | null>(null);

export function GraphSyncProvider({
  children,
  groupId,
  showControls = true,
}: {
  children: ReactNode;
  groupId: string;
  showControls?: boolean;
}) {
  const graphsRef = useRef<Map<string, (state: SyncState) => void>>(new Map());
  const graphDataRef = useRef<Map<string, () => GraphExportData | null>>(
    new Map(),
  );

  const [hasRegisteredGraphs, setHasRegisteredGraphs] = useState(false);
  const [currentTimeWindow, setCurrentTimeWindow] = useState<number | "all">(
    30 * 60 * 1000,
  );
  const [isLiveMode, setIsLiveMode] = useState(true);
  const [controlsVisible, setControlsVisible] = useState(showControls);

  const registerGraph = (id: string, updateFn: (state: SyncState) => void) => {
    graphsRef.current.set(id, updateFn);
  };

  const unregisterGraph = (id: string) => {
    graphsRef.current.delete(id);
  };

  const registerGraphData = (
    id: string,
    getDataFn: () => GraphExportData | null,
  ) => {
    graphDataRef.current.set(id, getDataFn);
    setHasRegisteredGraphs(graphDataRef.current.size > 0);
  };

  const unregisterGraphData = (id: string) => {
    graphDataRef.current.delete(id);
    setHasRegisteredGraphs(graphDataRef.current.size > 0);
  };

  const hasGraphs = () => {
    return graphDataRef.current.size > 0;
  };

  const syncZoom = (fromId: string, state: SyncState) => {
    graphsRef.current.forEach((updateFn, id) => {
      if (id !== fromId) {
        updateFn(state);
      }
    });
  };

  const syncTimeWindow = (fromId: string, timeWindow: number | "all") => {
    setCurrentTimeWindow(timeWindow);
    graphsRef.current.forEach((updateFn, id) => {
      if (id !== fromId) {
        updateFn({ x: { min: 0, max: 0 }, timeWindow });
      }
    });
  };

  const syncViewMode = (
    fromId: string,
    viewMode: "default" | "all" | "manual",
    isLiveMode: boolean,
  ) => {
    setIsLiveMode(isLiveMode);
    graphsRef.current.forEach((updateFn, id) => {
      if (id !== fromId) {
        updateFn({ x: { min: 0, max: 0 }, viewMode, isLiveMode });
      }
    });
  };

  const exportAllGraphs = () => {
    exportGraphsToExcel(graphDataRef.current, groupId);
  };

  const switchToLiveMode = () => {
    setIsLiveMode(true);
    graphsRef.current.forEach((updateFn) => {
      updateFn({
        x: { min: 0, max: 0 },
        viewMode: currentTimeWindow === "all" ? "all" : "default",
        isLiveMode: true,
      });
    });
  };

  const switchToHistoricalMode = () => {
    setIsLiveMode(false);
    graphsRef.current.forEach((updateFn) => {
      updateFn({
        x: { min: 0, max: 0 },
        viewMode: "manual",
        isLiveMode: false,
      });
    });
  };

  const handleTimeWindowChange = (timeWindow: number | "all") => {
    setCurrentTimeWindow(timeWindow);
    graphsRef.current.forEach((updateFn) => {
      updateFn({ x: { min: 0, max: 0 }, timeWindow });
    });
  };

  const getCurrentTimeWindow = () => currentTimeWindow;
  const getIsLiveMode = () => isLiveMode;
  const getTimeWindowOptions = () => DEFAULT_TIME_WINDOW_OPTIONS;
  const setShowControls = (show: boolean) => setControlsVisible(show);

  useEffect(() => {
    setControlsVisible(showControls);
  }, [showControls]);

  return (
    <GraphSyncContext.Provider
      value={{
        registerGraph,
        unregisterGraph,
        syncZoom,
        syncTimeWindow,
        syncViewMode,
        registerGraphData,
        unregisterGraphData,
        exportAllGraphs,
        hasGraphs,
        hasRegisteredGraphs,
        switchToLiveMode,
        switchToHistoricalMode,
        handleTimeWindowChange,
        getCurrentTimeWindow,
        getIsLiveMode,
        getTimeWindowOptions,
        showControls: controlsVisible,
        setShowControls,
      }}
    >
      {children}
    </GraphSyncContext.Provider>
  );
}

export const useGraphSync = () => {
  const context = useContext(GraphSyncContext);
  return context;
};

export function GraphControls({ groupId }: { groupId: string }) {
  const graphSync = useGraphSync();

  if (!graphSync || !graphSync.hasRegisteredGraphs || !graphSync.showControls) {
    return null;
  }

  const currentTimeWindow = graphSync.getCurrentTimeWindow();
  const isLiveMode = graphSync.getIsLiveMode();
  const timeWindowOptions = graphSync.getTimeWindowOptions();

  const getSelectedTimeWindowLabel = () => {
    const option = timeWindowOptions.find(
      (opt) => opt.value === currentTimeWindow,
    );
    return option ? option.label : "1m";
  };

  return (
    <ControlCard className="ml-auto w-fit py-4">
      <div className="flex items-center justify-end">
        <div className="flex items-center gap-3">
          <DropdownMenu>
            <DropdownMenuTrigger asChild>
              <TouchButton
                variant="outline"
                className="h-auto border-gray-300 bg-white px-3 py-3 text-base text-gray-900 hover:bg-gray-50"
              >
                {getSelectedTimeWindowLabel()}
                <Icon name="lu:ChevronDown" className="ml-2 size-4" />
              </TouchButton>
            </DropdownMenuTrigger>
            <DropdownMenuContent align="end">
              <DropdownMenuLabel className="text-base font-medium">
                Time Window
              </DropdownMenuLabel>
              <DropdownMenuSeparator />
              {timeWindowOptions.map((option) => (
                <DropdownMenuItem
                  key={option.value}
                  onClick={() => graphSync.handleTimeWindowChange(option.value)}
                  className={`min-h-[48px] px-4 py-3 text-base ${
                    currentTimeWindow === option.value ? "bg-blue-50" : ""
                  }`}
                >
                  {option.label}
                </DropdownMenuItem>
              ))}
            </DropdownMenuContent>
          </DropdownMenu>

          <TouchButton
            onClick={graphSync.switchToHistoricalMode}
            variant="outline"
            className={`h-auto px-3 py-3 text-base font-medium transition-colors ${
              !isLiveMode
                ? "bg-black text-white"
                : "border-gray-300 bg-white text-gray-700 hover:bg-gray-100"
            }`}
          >
            Historical
          </TouchButton>
          <TouchButton
            onClick={graphSync.switchToLiveMode}
            variant="outline"
            className={`h-auto px-3 py-3 text-base font-medium transition-colors ${
              isLiveMode
                ? "bg-black text-white"
                : "border-gray-300 bg-white text-gray-700 hover:bg-gray-100"
            }`}
          >
            Live
          </TouchButton>

          <div className="mx-2 h-8 w-px bg-gray-200"></div>

          <TouchButton
            onClick={graphSync.exportAllGraphs}
            variant="outline"
            className="h-auto bg-green-600 px-3 py-3 text-base font-medium text-white hover:bg-green-700"
          >
            Export
          </TouchButton>
        </div>
      </div>
    </ControlCard>
  );
}

export function FloatingControlPanel({ groupId }: { groupId: string }) {
  const graphSync = useGraphSync();
  const [isExpanded, setIsExpanded] = useState(false);

  if (!graphSync || !graphSync.hasRegisteredGraphs || !graphSync.showControls) {
    return null;
  }

  const currentTimeWindow = graphSync.getCurrentTimeWindow();
  const isLiveMode = graphSync.getIsLiveMode();
  const timeWindowOptions = graphSync.getTimeWindowOptions();

  const getSelectedTimeWindowLabel = () => {
    const option = timeWindowOptions.find(
      (opt) => opt.value === currentTimeWindow,
    );
    return option ? option.label : "1m";
  };

  return (
    <div className="fixed right-6 bottom-6 z-50">
      <ControlCard className="overflow-hidden px-4 py-4 transition-all duration-300 ease-in-out">
        <div
          className={`flex items-center ${isExpanded ? "gap-3" : "justify-center"}`}
        >
          <div
            className={`flex items-center gap-3 transition-all duration-300 ease-in-out ${
              isExpanded
                ? "max-w-none translate-x-0 opacity-100"
                : "w-0 max-w-0 overflow-hidden opacity-0"
            }`}
          >
            <DropdownMenu>
              <DropdownMenuTrigger asChild>
                <TouchButton
                  variant="outline"
                  className="h-auto border-gray-300 bg-white px-3 py-3 text-base text-gray-900 hover:bg-gray-50"
                >
                  {getSelectedTimeWindowLabel()}
                  <Icon name="lu:ChevronDown" className="ml-2 size-4" />
                </TouchButton>
              </DropdownMenuTrigger>
              <DropdownMenuContent align="end">
                <DropdownMenuLabel className="text-base font-medium">
                  Time Window
                </DropdownMenuLabel>
                <DropdownMenuSeparator />
                {timeWindowOptions.map((option) => (
                  <DropdownMenuItem
                    key={option.value}
                    onClick={() =>
                      graphSync.handleTimeWindowChange(option.value)
                    }
                    className={`min-h-[48px] px-4 py-3 text-base ${
                      currentTimeWindow === option.value ? "bg-blue-50" : ""
                    }`}
                  >
                    {option.label}
                  </DropdownMenuItem>
                ))}
              </DropdownMenuContent>
            </DropdownMenu>

            <TouchButton
              onClick={graphSync.switchToHistoricalMode}
              variant="outline"
              className={`h-auto px-3 py-3 text-base font-medium transition-colors ${
                !isLiveMode
                  ? "bg-black text-white"
                  : "border-gray-300 bg-white text-gray-700 hover:bg-gray-100"
              }`}
            >
              Historical
            </TouchButton>
            <TouchButton
              onClick={graphSync.switchToLiveMode}
              variant="outline"
              className={`h-auto px-3 py-3 text-base font-medium transition-colors ${
                isLiveMode
                  ? "bg-black text-white"
                  : "border-gray-300 bg-white text-gray-700 hover:bg-gray-100"
              }`}
            >
              Live
            </TouchButton>
            {isExpanded && <div className="h-8 w-px bg-gray-200"></div>}
            <TouchButton
              onClick={graphSync.exportAllGraphs}
              variant="outline"
              className="h-auto bg-green-600 px-3 py-3 text-base font-medium text-white hover:bg-green-700"
            >
              Export
            </TouchButton>
          </div>

          {isExpanded && <div className="h-8 w-px bg-gray-200"></div>}

          <TouchButton
            onClick={() => setIsExpanded(!isExpanded)}
            variant="outline"
            className="h-auto flex-shrink-0 bg-green-600 p-3 text-white hover:bg-green-700"
            icon={isExpanded ? "lu:Minus" : "lu:Plus"}
          />
        </div>
      </ControlCard>
    </div>
  );
}

export function FloatingExportButton({ groupId }: { groupId: string }) {
  return null;
}

// New prop-based sync types
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

type BigGraphProps = {
  newData: TimeSeries | null;
  unit?: Unit;
  renderValue?: (value: number) => string;
  config: GraphConfig;
  graphId: string;

  // Prop-based sync (new approach)
  syncGraph?: PropGraphSync;

  // Context-based sync (legacy support)
  syncGroupId?: string;
};

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

export function BigGraph({
  newData,
  unit,
  renderValue,
  config,
  syncGroupId,
  graphId,
  syncGraph,
}: BigGraphProps) {
  const containerRef = useRef<HTMLDivElement | null>(null);
  const uplotRef = useRef<uPlot | null>(null);
  const chartCreatedRef = useRef(false);

  // Initialize state from props or defaults
  const [viewMode, setViewMode] = useState<"default" | "all" | "manual">(
    syncGraph?.viewMode ?? "default",
  );
  const [isLiveMode, setIsLiveMode] = useState(syncGraph?.isLiveMode ?? true);
  const [selectedTimeWindow, setSelectedTimeWindow] = useState<number | "all">(
    syncGraph?.timeWindow ?? config.defaultTimeWindow ?? 30 * 60 * 1000,
  );

  const [cursorValue, setCursorValue] = useState<number | null>(null);
  const startTimeRef = useRef<number | null>(null);
  const manualScaleRef = useRef<{
    x: { min: number; max: number };
    y: { min: number; max: number };
  } | null>(null);
  const isUserZoomingRef = useRef(false);
  const isDraggingRef = useRef(false);
  const lastDragXRef = useRef<number | null>(null);
  const realPointsCountRef = useRef(0);
  const isPinchingRef = useRef(false);
  const lastPinchDistanceRef = useRef<number | null>(null);
  const pinchCenterRef = useRef<{ x: number; y: number } | null>(null);

  // Legacy sync context support
  const graphSync = syncGroupId ? useGraphSync() : null;
  const isSyncingRef = useRef(false);

  // Touch direction detection
  const touchStartRef = useRef<{ x: number; y: number; time: number } | null>(
    null,
  );
  const touchDirectionRef = useRef<"horizontal" | "vertical" | "unknown">(
    "unknown",
  );

  // Point-by-point animation refs
  const animationFrameRef = useRef<number | null>(null);
  const lastRenderedDataRef = useRef<{
    timestamps: number[];
    values: number[];
  }>({ timestamps: [], values: [] });
  const animationStateRef = useRef<{
    isAnimating: boolean;
    startTime: number;
    fromValue: number;
    toValue: number;
    fromTimestamp: number;
    toTimestamp: number;
    targetIndex: number;
  }>({
    isAnimating: false,
    startTime: 0,
    fromValue: 0,
    toValue: 0,
    fromTimestamp: 0,
    toTimestamp: 0,
    targetIndex: 0,
  });

  const POINT_ANIMATION_DURATION = 1000;
  const colors = {
    primary: config.colors?.primary ?? "#3b82f6",
    grid: config.colors?.grid ?? "#e2e8f0",
    axis: config.colors?.axis ?? "#64748b",
    background: config.colors?.background ?? "#ffffff",
  };

  // Prop-based sync state updates
  useEffect(() => {
    if (!syncGraph) return;

    let hasChanges = false;

    if (syncGraph.timeWindow !== selectedTimeWindow) {
      setSelectedTimeWindow(syncGraph.timeWindow);
      handleTimeWindowChangeInternal(syncGraph.timeWindow, true);
      hasChanges = true;
    }

    if (syncGraph.viewMode !== viewMode) {
      setViewMode(syncGraph.viewMode);
      hasChanges = true;
    }

    if (syncGraph.isLiveMode !== isLiveMode) {
      setIsLiveMode(syncGraph.isLiveMode);
      hasChanges = true;
    }

    if (syncGraph.xRange && uplotRef.current) {
      uplotRef.current.batch(() => {
        uplotRef.current!.setScale("x", {
          min: syncGraph.xRange!.min,
          max: syncGraph.xRange!.max,
        });

        if (newData?.long) {
          const [timestamps, values] = seriesToUPlotData(newData.long);
          updateYAxisScale(
            timestamps,
            values,
            syncGraph.xRange!.min,
            syncGraph.xRange!.max,
          );
        }
      });

      manualScaleRef.current = {
        x: syncGraph.xRange,
        y: manualScaleRef.current?.y ?? { min: 0, max: 1 },
      };
      hasChanges = true;
    }

    if (hasChanges && syncGraph.viewMode === "manual") {
      setViewMode("manual");
      setIsLiveMode(false);
    }
  }, [syncGraph]);

  // Legacy context sync handler
  const handleSyncUpdate = useCallback(
    (state: SyncState) => {
      if (!uplotRef.current) return;

      const wasUserZooming = isUserZoomingRef.current;
      isUserZoomingRef.current = false;

      if (state.viewMode !== undefined && state.isLiveMode !== undefined) {
        setViewMode(state.viewMode);
        setIsLiveMode(state.isLiveMode);

        if (state.isLiveMode) {
          stopAnimations();
          if (newData?.long) {
            const [timestamps, values] = seriesToUPlotData(newData.long);
            const fullData = buildUPlotData(timestamps, values);
            uplotRef.current.setData(fullData);

            if (timestamps.length > 0) {
              const latestTimestamp = timestamps[timestamps.length - 1];

              if (state.viewMode === "all") {
                const fullStart = startTimeRef.current ?? timestamps[0];
                uplotRef.current.setScale("x", {
                  min: fullStart,
                  max: latestTimestamp,
                });
                updateYAxisScale(
                  timestamps,
                  values,
                  fullStart,
                  latestTimestamp,
                );
              } else if (state.viewMode === "default") {
                const viewStart =
                  latestTimestamp - (selectedTimeWindow as number);
                uplotRef.current.setScale("x", {
                  min: viewStart,
                  max: latestTimestamp,
                });
                updateYAxisScale(
                  timestamps,
                  values,
                  viewStart,
                  latestTimestamp,
                );
              }
            }

            manualScaleRef.current = null;
            lastRenderedDataRef.current = { timestamps, values };
          }
        } else {
          stopAnimations();
          if (uplotRef.current.scales) {
            const xScale = uplotRef.current.scales.x;
            const yScale = uplotRef.current.scales.y;
            if (
              xScale &&
              yScale &&
              xScale.min !== undefined &&
              xScale.max !== undefined &&
              yScale.min !== undefined &&
              yScale.max !== undefined
            ) {
              manualScaleRef.current = {
                x: { min: xScale.min ?? 0, max: xScale.max ?? 0 },
                y: { min: yScale.min, max: yScale.max },
              };
            }
          }
        }
      } else if (state.timeWindow !== undefined) {
        setSelectedTimeWindow(state.timeWindow);
        handleTimeWindowChangeInternal(state.timeWindow, true);
      } else {
        setViewMode("manual");
        setIsLiveMode(false);

        uplotRef.current.batch(() => {
          uplotRef.current!.setScale("x", {
            min: state.x.min,
            max: state.x.max,
          });

          if (newData?.long) {
            const [timestamps, values] = seriesToUPlotData(newData.long);
            updateYAxisScale(timestamps, values, state.x.min, state.x.max);
          }
        });

        if (newData?.long) {
          const [_series, values] = seriesToUPlotData(newData.long);
          manualScaleRef.current = {
            x: { min: state.x.min, max: state.x.max },
            y: manualScaleRef.current?.y ?? {
              min: Math.min(...values),
              max: Math.max(...values),
            },
          };
        }
      }

      setTimeout(() => {
        isUserZoomingRef.current = wasUserZooming;
      }, 10);
    },
    [newData, selectedTimeWindow],
  );

  // Legacy context registration
  useEffect(() => {
    if (graphSync) {
      const getGraphData = (): GraphExportData | null => ({
        config,
        data: newData,
        unit,
        renderValue,
      });

      graphSync.registerGraphData(graphId, getGraphData);
      return () => graphSync.unregisterGraphData(graphId);
    }
  }, [graphSync, graphId, config, newData, unit, renderValue]);

  useEffect(() => {
    if (graphSync) {
      graphSync.registerGraph(graphId, handleSyncUpdate);
      return () => graphSync.unregisterGraph(graphId);
    }
  }, [graphSync, graphId, handleSyncUpdate]);

  const lerp = (start: number, end: number, t: number): number => {
    return start + (end - start) * t;
  };

  const animateNewPoint = (
    currentData: { timestamps: number[]; values: number[] },
    targetData: { timestamps: number[]; values: number[] },
  ) => {
    if (targetData.timestamps.length <= currentData.timestamps.length) {
      return;
    }

    const newIndex = currentData.timestamps.length;

    const prevValue =
      currentData.values[newIndex - 1] ?? targetData.values[newIndex];
    const prevTimestamp =
      currentData.timestamps[newIndex - 1] ?? targetData.timestamps[newIndex];
    const newValue = targetData.values[newIndex];
    const newTimestamp = targetData.timestamps[newIndex];

    if (animationFrameRef.current) {
      cancelAnimationFrame(animationFrameRef.current);
      animationFrameRef.current = null;
    }

    animationStateRef.current = {
      isAnimating: true,
      startTime: performance.now(),
      fromValue: prevValue,
      toValue: newValue,
      fromTimestamp: prevTimestamp,
      toTimestamp: newTimestamp,
      targetIndex: newIndex,
    };

    const animate = (currentTime: number) => {
      if (!uplotRef.current || !animationStateRef.current.isAnimating) return;

      const elapsed = currentTime - animationStateRef.current.startTime;
      const progress = Math.min(elapsed / POINT_ANIMATION_DURATION, 1);

      const animatedTimestamps = [...currentData.timestamps];
      const animatedValues = [...currentData.values];

      if (progress < 1) {
        const interpolatedTimestamp = lerp(
          animationStateRef.current.fromTimestamp,
          animationStateRef.current.toTimestamp,
          progress,
        );
        const interpolatedValue = lerp(
          animationStateRef.current.fromValue,
          animationStateRef.current.toValue,
          progress,
        );

        animatedTimestamps.push(interpolatedTimestamp);
        animatedValues.push(interpolatedValue);
      } else {
        animatedTimestamps.push(animationStateRef.current.toTimestamp);
        animatedValues.push(animationStateRef.current.toValue);

        lastRenderedDataRef.current = {
          timestamps: [...animatedTimestamps],
          values: [...animatedValues],
        };
        realPointsCountRef.current = animatedTimestamps.length;
        animationStateRef.current.isAnimating = false;
      }

      const animatedUData = buildUPlotData(
        animatedTimestamps,
        animatedValues,
        animatedTimestamps.length,
      );
      uplotRef.current.setData(animatedUData);

      if (isLiveMode && animatedTimestamps.length > 0) {
        const latestTimestamp =
          animatedTimestamps[animatedTimestamps.length - 1];

        if (viewMode === "default") {
          let xMin: number | undefined, xMax: number | undefined;

          if (selectedTimeWindow === "all") {
            const fullStart = startTimeRef.current ?? animatedTimestamps[0];
            xMin = fullStart;
            xMax = latestTimestamp;
          } else {
            xMin = latestTimestamp - (selectedTimeWindow as number);
            xMax = latestTimestamp;
          }

          uplotRef.current.setScale("x", { min: xMin, max: xMax });
          updateYAxisScale(animatedTimestamps, animatedValues, xMin, xMax);
        } else if (viewMode === "all") {
          const fullStart = startTimeRef.current ?? animatedTimestamps[0];
          uplotRef.current.setScale("x", {
            min: fullStart,
            max: latestTimestamp,
          });
          updateYAxisScale(
            animatedTimestamps,
            animatedValues,
            fullStart,
            latestTimestamp,
          );
        }
      }

      if (progress < 1) {
        animationFrameRef.current = requestAnimationFrame(animate);
      } else {
        if (targetData.timestamps.length > animatedTimestamps.length) {
          animateNewPoint(
            { timestamps: animatedTimestamps, values: animatedValues },
            targetData,
          );
        }
      }
    };

    animationFrameRef.current = requestAnimationFrame(animate);
  };

  const stopAnimations = () => {
    if (animationFrameRef.current) {
      cancelAnimationFrame(animationFrameRef.current);
      animationFrameRef.current = null;
    }
    animationStateRef.current.isAnimating = false;
  };

  const updateYAxisScale = (
    timestamps: number[],
    values: number[],
    xMin?: number,
    xMax?: number,
  ) => {
    if (!uplotRef.current || values.length === 0) return;

    let visibleValues: number[] = [];

    if (xMin !== undefined && xMax !== undefined) {
      for (let i = 0; i < timestamps.length; i++) {
        if (timestamps[i] >= xMin && timestamps[i] <= xMax) {
          visibleValues.push(values[i]);
        }
      }
    } else {
      visibleValues = [...values];
    }

    config.lines?.forEach((line) => {
      if (line.show !== false) {
        visibleValues.push(line.value);
      }
    });

    if (visibleValues.length === 0) {
      visibleValues = values;
    }

    const minY = Math.min(...visibleValues);
    const maxY = Math.max(...visibleValues);
    const range = maxY - minY || Math.abs(maxY) * 0.1 || 1;

    const yRange = {
      min: minY - range * 0.1,
      max: maxY + range * 0.1,
    };

    uplotRef.current.batch(() => {
      uplotRef.current!.setScale("y", yRange);
    });

    if (viewMode === "manual" && manualScaleRef.current) {
      manualScaleRef.current.y = yRange;
    }
  };

  const getRightmostVisibleTimestamp = () => {
    if (!uplotRef.current || !newData?.long) return null;
    const xScale = uplotRef.current.scales.x;
    if (!xScale || xScale.max === undefined) return null;
    return xScale.max;
  };

  const buildUPlotData = (
    timestamps: number[],
    values: number[],
    realPointsCount?: number,
  ): uPlot.AlignedData => {
    const uData: uPlot.AlignedData = [timestamps, values];

    if (realPointsCount !== undefined) {
      realPointsCountRef.current = realPointsCount;
    }

    config.lines?.forEach((line) => {
      if (line.show !== false) {
        uData.push(timestamps.map(() => line.value));
      }
    });

    return uData;
  };

  const createChart = () => {
    if (!containerRef.current || !newData?.long) return;

    const [timestamps, values] = seriesToUPlotData(newData.long);
    if (timestamps.length === 0) return;

    if (startTimeRef.current === null && timestamps.length > 0) {
      startTimeRef.current = timestamps[0];
    }

    const uData = buildUPlotData(timestamps, values);

    const lastTimestamp = timestamps[timestamps.length - 1] ?? 0;
    const fullStart = startTimeRef.current ?? timestamps[0] ?? 0;

    let initialMin: number, initialMax: number;
    if (viewMode === "manual" && manualScaleRef.current) {
      initialMin = manualScaleRef.current.x.min;
      initialMax = manualScaleRef.current.x.max;
    } else if (selectedTimeWindow === "all") {
      initialMin = fullStart;
      initialMax = lastTimestamp;
    } else {
      const defaultViewStart = Math.max(
        lastTimestamp - (selectedTimeWindow as number),
        fullStart,
      );
      initialMin = defaultViewStart;
      initialMax = lastTimestamp;
    }

    const initialVisibleValues: number[] = [];
    for (let i = 0; i < timestamps.length; i++) {
      if (timestamps[i] >= initialMin && timestamps[i] <= initialMax) {
        initialVisibleValues.push(values[i]);
      }
    }

    config.lines?.forEach((line) => {
      if (line.show !== false) {
        initialVisibleValues.push(line.value);
      }
    });

    if (initialVisibleValues.length === 0) {
      initialVisibleValues.push(...values);
      config.lines?.forEach((line) => {
        if (line.show !== false) {
          initialVisibleValues.push(line.value);
        }
      });
    }

    let initialYMin: number, initialYMax: number;
    if (initialVisibleValues.length > 0) {
      const minY = Math.min(...initialVisibleValues);
      const maxY = Math.max(...initialVisibleValues);
      const range = maxY - minY || Math.abs(maxY) * 0.1 || 1;

      initialYMin = minY - range * 0.1;
      initialYMax = maxY + range * 0.1;
    } else {
      initialYMin = -1;
      initialYMax = 1;
    }

    const rect = containerRef.current.getBoundingClientRect();
    const width = rect.width;
    const height = Math.min(rect.height, window.innerHeight * 0.5);

    const seriesConfig: uPlot.Series[] = [
      { label: "Time" },
      {
        label: "Value",
        stroke: colors.primary,
        width: 2,
        spanGaps: true,
        points: {
          show: (_u, _seriesIdx, dataIdx) => {
            return dataIdx < realPointsCountRef.current;
          },
          size: 6,
          stroke: colors.primary,
          fill: colors.primary,
          width: 2,
        },
      },
    ];

    config.lines?.forEach((line) => {
      if (line.show !== false) {
        seriesConfig.push({
          label: line.label,
          stroke: line.color,
          width: line.width ?? 1,
          dash: line.dash ?? (line.type === "threshold" ? [5, 5] : undefined),
          show: true,
        });
      }
    });

    if (uplotRef.current) {
      uplotRef.current.destroy();
      uplotRef.current = null;
    }

    uplotRef.current = new uPlot(
      {
        width,
        height,
        padding: [-10, 20, -10, 20],
        cursor: {
          show: true,
          x: true,
          y: true,
          drag: {
            x: true,
            y: false,
            setScale: true,
          },
          sync: { key: "myCursor" },
        },
        legend: {
          show: false,
        },
        hooks: {
          setScale: [
            (u) => {
              if (isUserZoomingRef.current) {
                const xScale = u.scales.x;
                if (xScale.min !== undefined && xScale.max !== undefined) {
                  const [timestamps, values] = seriesToUPlotData(newData.long);
                  updateYAxisScale(timestamps, values, xScale.min, xScale.max);

                  manualScaleRef.current = {
                    x: { min: xScale.min ?? 0, max: xScale.max ?? 0 },
                    y: {
                      min: u.scales.y?.min ?? 0,
                      max: u.scales.y?.max ?? 1,
                    },
                  };

                  setViewMode("manual");
                  setIsLiveMode(false);

                  // Prop-based sync
                  if (syncGraph?.onZoomChange) {
                    syncGraph.onZoomChange(graphId, {
                      min: xScale.min ?? 0,
                      max: xScale.max ?? 0,
                    });
                  }

                  // Legacy context sync
                  if (graphSync) {
                    setTimeout(() => {
                      graphSync.syncZoom(graphId, {
                        x: { min: xScale.min ?? 0, max: xScale.max ?? 0 },
                      });
                    }, 0);
                  }
                }
                isUserZoomingRef.current = false;
              }
            },
          ],

          setCursor: [
            (u) => {
              if (
                typeof u.cursor.idx === "number" &&
                u.data[1] &&
                u.data[1][u.cursor.idx] !== undefined
              ) {
                const timestamp = u.data[0][u.cursor.idx];
                const value = u.data[1][u.cursor.idx];
                const cur = newData?.current;

                const isNearCurrent =
                  cur &&
                  timestamp !== undefined &&
                  Math.abs(timestamp - cur.timestamp) < 1000;

                const displayValue = isNearCurrent ? cur.value : value;
                setCursorValue(displayValue ?? null);
              } else {
                setCursorValue(null);
              }
            },
          ],
        },
        scales: {
          x: {
            time: true,
            auto: true,
            min: initialMin,
            max: initialMax,
            distr: 1,
          },
          y: {
            auto: false,
            min: initialYMin,
            max: initialYMax,
          },
        },

        axes: [
          {
            stroke: colors.axis,
            labelSize: 12,
            labelFont: "Inter, system-ui, sans-serif",
            grid: { stroke: colors.grid, width: 1 },
            space: 60,
            values: (u, ticks) => {
              const xScale = u.scales.x;
              if (
                !xScale ||
                xScale.min === undefined ||
                xScale.max === undefined
              ) {
                return ticks.map((ts) => {
                  const date = new Date(ts);
                  return date.toLocaleTimeString("en-GB", {
                    hour12: false,
                    hour: "2-digit",
                    minute: "2-digit",
                    second: "2-digit",
                  });
                });
              }

              const timeRange = xScale.max - xScale.min;

              return ticks.map((ts) => {
                const date = new Date(ts);

                if (timeRange <= 30 * 1000) {
                  return date.toLocaleTimeString("en-GB", {
                    hour12: false,
                    hour: "2-digit",
                    minute: "2-digit",
                    second: "2-digit",
                  });
                } else if (timeRange <= 5 * 60 * 1000) {
                  return date.toLocaleTimeString("en-GB", {
                    hour12: false,
                    hour: "2-digit",
                    minute: "2-digit",
                    second: "2-digit",
                  });
                } else if (timeRange <= 60 * 60 * 1000) {
                  return date.toLocaleTimeString("en-GB", {
                    hour12: false,
                    hour: "2-digit",
                    minute: "2-digit",
                  });
                } else {
                  return date.toLocaleTimeString("en-GB", {
                    hour12: false,
                    hour: "2-digit",
                    minute: "2-digit",
                  });
                }
              });
            },

            splits: (
              _u,
              _axisIdx,
              scaleMin,
              scaleMax,
              _foundIncr,
              _foundSpace,
            ) => {
              const timeRange = scaleMax - scaleMin;
              const ticks: number[] = [];

              if (timeRange <= 10 * 1000) {
                const startTime = Math.ceil(scaleMin / 1000) * 1000;
                for (let t = startTime; t <= scaleMax; t += 1000) {
                  if (t >= scaleMin) ticks.push(t);
                }
              } else if (timeRange <= 30 * 1000) {
                const startTime = Math.ceil(scaleMin / 5000) * 5000;
                for (let t = startTime; t <= scaleMax; t += 5000) {
                  if (t >= scaleMin) ticks.push(t);
                }
              } else if (timeRange <= 60 * 1000) {
                const startTime = Math.ceil(scaleMin / 10000) * 10000;
                for (let t = startTime; t <= scaleMax; t += 10000) {
                  if (t >= scaleMin) ticks.push(t);
                }
              } else if (timeRange <= 5 * 60 * 1000) {
                const startTime = Math.ceil(scaleMin / 30000) * 30000;
                for (let t = startTime; t <= scaleMax; t += 30000) {
                  if (t >= scaleMin) ticks.push(t);
                }
              } else if (timeRange <= 10 * 60 * 1000) {
                const startTime = Math.ceil(scaleMin / 60000) * 60000;
                for (let t = startTime; t <= scaleMax; t += 60000) {
                  if (t >= scaleMin) ticks.push(t);
                }
              } else if (timeRange <= 30 * 60 * 1000) {
                const startTime = Math.ceil(scaleMin / 120000) * 120000;
                for (let t = startTime; t <= scaleMax; t += 120000) {
                  if (t >= scaleMin) ticks.push(t);
                }
              } else if (timeRange <= 60 * 60 * 1000) {
                const startTime = Math.ceil(scaleMin / 300000) * 300000;
                for (let t = startTime; t <= scaleMax; t += 300000) {
                  if (t >= scaleMin) ticks.push(t);
                }
              } else if (timeRange <= 6 * 60 * 60 * 1000) {
                const startTime = Math.ceil(scaleMin / 1800000) * 1800000;
                for (let t = startTime; t <= scaleMax; t += 1800000) {
                  if (t >= scaleMin) ticks.push(t);
                }
              } else {
                const startTime = Math.ceil(scaleMin / 3600000) * 3600000;
                for (let t = startTime; t <= scaleMax; t += 3600000) {
                  if (t >= scaleMin) ticks.push(t);
                }
              }

              return ticks;
            },
          },

          {
            stroke: colors.axis,
            labelSize: 12,
            labelFont: "Inter, system-ui, sans-serif",
            grid: { stroke: colors.grid, width: 1 },
            side: 1,
            space: 60,
            values: (_u, ticks) => {
              if (renderValue) {
                const renderedValues = ticks.map(renderValue);
                const uniqueValues = new Set(renderedValues);

                if (uniqueValues.size === renderedValues.length) {
                  return renderedValues;
                }
              }

              const precision = 0;
              const maxPrecision = 4;

              for (let p = precision; p <= maxPrecision; p++) {
                const formattedValues = ticks.map((v) => v.toFixed(p));
                const uniqueFormatted = new Set(formattedValues);

                if (uniqueFormatted.size === formattedValues.length) {
                  return formattedValues;
                }
              }

              return ticks.map((v) => v.toFixed(maxPrecision));
            },
          },
        ],

        series: seriesConfig,
      },
      uData,
      containerRef.current,
    );

    const handleTouchStart = (e: TouchEvent) => {
      const touch = e.touches[0];
      touchStartRef.current = {
        x: touch.clientX,
        y: touch.clientY,
        time: Date.now(),
      };
      touchDirectionRef.current = "unknown";

      if (e.touches.length === 2) {
        isPinchingRef.current = true;
        isDraggingRef.current = false;
        touchDirectionRef.current = "horizontal";

        const touch1 = e.touches[0];
        const touch2 = e.touches[1];

        const distance = Math.sqrt(
          Math.pow(touch2.clientX - touch1.clientX, 2) +
            Math.pow(touch2.clientY - touch1.clientY, 2),
        );
        lastPinchDistanceRef.current = distance;

        pinchCenterRef.current = {
          x: (touch1.clientX + touch2.clientX) / 2,
          y: (touch1.clientY + touch2.clientY) / 2,
        };

        e.preventDefault();
      }
    };

    const handleTouchMove = (e: TouchEvent) => {
      if (!touchStartRef.current) return;

      if (e.touches.length === 1) {
        const touch = e.touches[0];
        const deltaX = Math.abs(touch.clientX - touchStartRef.current.x);
        const deltaY = Math.abs(touch.clientY - touchStartRef.current.y);
        const timeDelta = Date.now() - touchStartRef.current.time;

        if (
          touchDirectionRef.current === "unknown" &&
          deltaX > 20 &&
          deltaY < 10 &&
          deltaX > deltaY * 4 &&
          timeDelta < 500
        ) {
          touchDirectionRef.current = "horizontal";
          isDraggingRef.current = true;
          lastDragXRef.current = touch.clientX;
          e.preventDefault();
        } else if (touchDirectionRef.current === "unknown") {
          return;
        }

        if (
          touchDirectionRef.current === "horizontal" &&
          isDraggingRef.current
        ) {
          e.preventDefault();

          const currentX = touch.clientX;
          const dragDelta = currentX - (lastDragXRef.current || 0);
          lastDragXRef.current = currentX;

          if (uplotRef.current && Math.abs(dragDelta) > 2) {
            const xScale = uplotRef.current.scales.x;
            if (
              xScale &&
              xScale.min !== undefined &&
              xScale.max !== undefined
            ) {
              const pixelToTime = (xScale.max - xScale.min) / width;
              const timeDelta = -dragDelta * pixelToTime;

              const newMin = xScale.min + timeDelta;
              const newMax = xScale.max + timeDelta;

              uplotRef.current.setScale("x", { min: newMin, max: newMax });
              const [timestamps, values] = seriesToUPlotData(newData.long);
              updateYAxisScale(timestamps, values, newMin, newMax);

              manualScaleRef.current = {
                x: { min: newMin, max: newMax },
                y: {
                  min: uplotRef.current.scales.y?.min ?? 0,
                  max: uplotRef.current.scales.y?.max ?? 1,
                },
              };

              setViewMode("manual");
              setIsLiveMode(false);

              // Prop-based sync
              if (syncGraph?.onZoomChange) {
                syncGraph.onZoomChange(graphId, {
                  min: newMin,
                  max: newMax,
                });
              }

              // Legacy context sync
              if (graphSync) {
                graphSync.syncZoom(graphId, {
                  x: { min: newMin, max: newMax },
                });
              }
            }
          }
        }
      } else if (e.touches.length === 2 && isPinchingRef.current) {
        e.preventDefault();

        const touch1 = e.touches[0];
        const touch2 = e.touches[1];

        const newDistance = Math.sqrt(
          Math.pow(touch2.clientX - touch1.clientX, 2) +
            Math.pow(touch2.clientY - touch1.clientY, 2),
        );

        if (lastPinchDistanceRef.current && uplotRef.current) {
          const scaleFactor = newDistance / lastPinchDistanceRef.current;
          const xScale = uplotRef.current.scales.x;

          if (
            xScale &&
            xScale.min !== undefined &&
            xScale.max !== undefined &&
            pinchCenterRef.current
          ) {
            const rect = containerRef.current?.getBoundingClientRect();
            if (rect) {
              const touchXRelative =
                (pinchCenterRef.current.x - rect.left) / rect.width;
              const centerTime =
                xScale.min + (xScale.max - xScale.min) * touchXRelative;

              const currentRange = xScale.max - xScale.min;
              const newRange = currentRange / scaleFactor;

              const leftRatio = (centerTime - xScale.min) / currentRange;
              const rightRatio = (xScale.max - centerTime) / currentRange;

              const newMin = centerTime - newRange * leftRatio;
              const newMax = centerTime + newRange * rightRatio;

              uplotRef.current.setScale("x", { min: newMin, max: newMax });
              const [timestamps, values] = seriesToUPlotData(newData.long);
              updateYAxisScale(timestamps, values, newMin, newMax);

              manualScaleRef.current = {
                x: { min: newMin, max: newMax },
                y: {
                  min: uplotRef.current.scales.y?.min ?? 0,
                  max: uplotRef.current.scales.y?.max ?? 1,
                },
              };
              setViewMode("manual");
              setIsLiveMode(false);

              // Prop-based sync
              if (syncGraph?.onZoomChange) {
                syncGraph.onZoomChange(graphId, {
                  min: newMin,
                  max: newMax,
                });
              }

              // Legacy context sync
              if (graphSync) {
                graphSync.syncZoom(graphId, {
                  x: { min: newMin, max: newMax },
                });
              }
            }
          }
        }

        lastPinchDistanceRef.current = newDistance;
      }
    };

    const handleTouchEnd = (e: TouchEvent) => {
      if (e.touches.length === 0) {
        isDraggingRef.current = false;
        isPinchingRef.current = false;
        lastDragXRef.current = null;
        lastPinchDistanceRef.current = null;
        pinchCenterRef.current = null;
        touchStartRef.current = null;
        touchDirectionRef.current = "unknown";
      } else if (e.touches.length === 1 && isPinchingRef.current) {
        isPinchingRef.current = false;
        lastPinchDistanceRef.current = null;
        pinchCenterRef.current = null;

        const touch = e.touches[0];
        touchStartRef.current = {
          x: touch.clientX,
          y: touch.clientY,
          time: Date.now(),
        };
        touchDirectionRef.current = "unknown";
        isDraggingRef.current = false;
      }

      if (touchDirectionRef.current === "horizontal" && isDraggingRef.current) {
        e.preventDefault();
      }
    };

    const handleMouseDown = (e: MouseEvent) => {
      if (e.button === 0) {
        isUserZoomingRef.current = true;
      }
    };

    const handleWheel = (e: WheelEvent) => {
      e.preventDefault();
    };

    if (containerRef.current && uplotRef.current) {
      containerRef.current.addEventListener("touchstart", handleTouchStart, {
        passive: false,
      });
      containerRef.current.addEventListener("touchmove", handleTouchMove, {
        passive: false,
      });
      containerRef.current.addEventListener("touchend", handleTouchEnd, {
        passive: false,
      });

      containerRef.current.addEventListener("mousedown", handleMouseDown);

      containerRef.current.addEventListener("wheel", handleWheel, {
        passive: false,
      });
    }

    chartCreatedRef.current = true;

    lastRenderedDataRef.current = {
      timestamps: [...timestamps],
      values: [...values],
    };
  };

  const isSwitchingModeRef = useRef(false);
  const isUserTimeWindowChangeRef = useRef(false);

  const handleTimeWindowChangeInternal = (
    newTimeWindow: number | "all",
    isSync: boolean = false,
  ) => {
    if (isSwitchingModeRef.current || (!isSync && isSyncingRef.current)) {
      return;
    }

    if (!isSync) {
      isUserTimeWindowChangeRef.current = true;
    }

    stopAnimations();
    setSelectedTimeWindow(newTimeWindow);

    if (!uplotRef.current || !newData?.long) {
      if (!isSync) {
        isUserTimeWindowChangeRef.current = false;
      }
      return;
    }

    const [timestamps, values] = seriesToUPlotData(newData.long);
    if (timestamps.length === 0) {
      if (!isSync) {
        isUserTimeWindowChangeRef.current = false;
      }
      return;
    }

    if (newTimeWindow === "all") {
      setViewMode("all");
      if (isLiveMode) {
        const fullStart = startTimeRef.current ?? timestamps[0];
        const fullEnd = timestamps[timestamps.length - 1];
        uplotRef.current.setScale("x", { min: fullStart, max: fullEnd });
        updateYAxisScale(timestamps, values, fullStart, fullEnd);
      }
      manualScaleRef.current = null;
    } else {
      if (isLiveMode) {
        setViewMode("default");
        const latestTimestamp = timestamps[timestamps.length - 1];
        const viewStart = latestTimestamp - newTimeWindow;
        uplotRef.current.setScale("x", {
          min: viewStart,
          max: latestTimestamp,
        });
        updateYAxisScale(timestamps, values, viewStart, latestTimestamp);
        manualScaleRef.current = null;
      } else {
        const rightmostTimestamp = getRightmostVisibleTimestamp();
        if (rightmostTimestamp) {
          const newViewStart = rightmostTimestamp - newTimeWindow;
          uplotRef.current.setScale("x", {
            min: newViewStart,
            max: rightmostTimestamp,
          });
          updateYAxisScale(
            timestamps,
            values,
            newViewStart,
            rightmostTimestamp,
          );

          manualScaleRef.current = {
            x: { min: newViewStart, max: rightmostTimestamp },
            y: manualScaleRef.current?.y ?? {
              min: Math.min(...values),
              max: Math.max(...values),
            },
          };
        }
      }
    }

    lastRenderedDataRef.current = { timestamps, values };

    // Prop-based sync
    if (!isSync && syncGraph?.onTimeWindowChange) {
      syncGraph.onTimeWindowChange(graphId, newTimeWindow);
    }

    // Legacy context sync
    if (graphSync && !isSync && !isSyncingRef.current) {
      graphSync.syncTimeWindow(graphId, newTimeWindow);
    }

    if (!isSync) {
      setTimeout(() => {
        isUserTimeWindowChangeRef.current = false;
      }, 50);
    }
  };

  // Chart creation effect
  useEffect(() => {
    if (!containerRef.current || !newData?.long) {
      chartCreatedRef.current = false;
      return;
    }

    const [timestamps] = seriesToUPlotData(newData.long);
    if (timestamps.length === 0) {
      chartCreatedRef.current = false;
      return;
    }

    createChart();

    return () => {
      if (uplotRef.current) {
        uplotRef.current.destroy();
        uplotRef.current = null;
      }
      stopAnimations();
      chartCreatedRef.current = false;
    };
  }, [newData?.long, containerRef.current]);

  useEffect(() => {
    if (
      !uplotRef.current ||
      !newData?.long ||
      !chartCreatedRef.current ||
      !isLiveMode
    )
      return;

    const [timestamps, values] = seriesToUPlotData(newData.long);
    if (timestamps.length === 0) return;

    const currentData = lastRenderedDataRef.current;
    const targetData = { timestamps, values };

    if (targetData.timestamps.length > currentData.timestamps.length) {
      const maxAnimatableLength = Math.min(
        targetData.timestamps.length,
        currentData.timestamps.length + 1,
      );

      const limitedTargetData = {
        timestamps: targetData.timestamps.slice(0, maxAnimatableLength),
        values: targetData.values.slice(0, maxAnimatableLength),
      };

      animateNewPoint(currentData, limitedTargetData);
    } else if (targetData.timestamps.length === currentData.timestamps.length) {
      let hasChanges = false;
      for (let i = 0; i < targetData.values.length; i++) {
        if (Math.abs(targetData.values[i] - currentData.values[i]) > 0.001) {
          hasChanges = true;
          break;
        }
      }

      if (hasChanges) {
        const uData = buildUPlotData(timestamps, values);
        uplotRef.current.setData(uData);
        lastRenderedDataRef.current = { timestamps, values };

        const lastTimestamp = timestamps[timestamps.length - 1] ?? 0;
        if (viewMode === "default") {
          let xMin: number | undefined, xMax: number | undefined;

          if (selectedTimeWindow === "all") {
            const fullStart = startTimeRef.current ?? timestamps[0];
            xMin = fullStart;
            xMax = lastTimestamp;
          } else {
            xMin = lastTimestamp - (selectedTimeWindow as number);
            xMax = lastTimestamp;
          }

          uplotRef.current.setScale("x", { min: xMin, max: xMax });
          updateYAxisScale(timestamps, values, xMin, xMax);
        } else if (viewMode === "all") {
          const fullStart = startTimeRef.current ?? timestamps[0];
          uplotRef.current.setScale("x", {
            min: fullStart,
            max: lastTimestamp,
          });
          updateYAxisScale(timestamps, values, fullStart, lastTimestamp);
        }
      }
    }
  }, [
    newData?.long?.validCount,
    newData?.long?.lastTimestamp,
    viewMode,
    selectedTimeWindow,
    isLiveMode,
  ]);

  useEffect(() => {
    if (
      !uplotRef.current ||
      !newData?.current ||
      !isLiveMode ||
      !chartCreatedRef.current ||
      animationStateRef.current.isAnimating
    )
      return;

    const updateLiveData = () => {
      if (!newData?.long || !newData?.current || !uplotRef.current) return;

      const [timestamps, values] = seriesToUPlotData(newData.long);
      const cur = newData.current;
      const liveTimestamps = [...timestamps];
      const liveValues = [...values];

      liveTimestamps.push(cur.timestamp);
      liveValues.push(cur.value);

      const liveData = buildUPlotData(liveTimestamps, liveValues);
      uplotRef.current.setData(liveData);

      const latestTimestamp = liveTimestamps[liveTimestamps.length - 1];

      if (viewMode === "default") {
        let xMin, xMax;

        if (selectedTimeWindow === "all") {
          const fullStart = startTimeRef.current ?? liveTimestamps[0];
          xMin = fullStart;
          xMax = latestTimestamp;
        } else {
          xMin = latestTimestamp - (selectedTimeWindow as number);
          xMax = latestTimestamp;
        }

        uplotRef.current.batch(() => {
          uplotRef.current!.setScale("x", { min: xMin, max: xMax });
          updateYAxisScale(liveTimestamps, liveValues, xMin, xMax);
        });
      } else if (viewMode === "all") {
        const fullStart = startTimeRef.current ?? liveTimestamps[0];
        uplotRef.current.batch(() => {
          uplotRef.current!.setScale("x", {
            min: fullStart,
            max: latestTimestamp,
          });
          updateYAxisScale(
            liveTimestamps,
            liveValues,
            fullStart,
            latestTimestamp,
          );
        });
      }

      if (startTimeRef.current === null && liveTimestamps.length > 0) {
        startTimeRef.current = liveTimestamps[0];
      }
    };

    updateLiveData();
  }, [
    newData?.current?.timestamp,
    viewMode,
    selectedTimeWindow,
    config.lines,
    isLiveMode,
  ]);

  const displayValue =
    cursorValue !== null ? cursorValue : newData?.current?.value;

  return (
    <div className="h-[50vh] w-full">
      <div className="flex h-full w-full flex-col overflow-hidden rounded-3xl border border-gray-200 bg-white shadow">
        <div className="flex items-center justify-between pt-4 pr-5 pb-6 pl-6">
          <div className="mt-1 flex items-center gap-4">
            <Icon
              name={unit ? getUnitIcon(unit) : "lu:TrendingUp"}
              className="size-6 text-gray-600"
            />

            <h2 className="text-2xl leading-none font-bold text-gray-900">
              {config.title}
            </h2>

            <div className="flex items-center gap-2 text-base text-gray-600">
              <span className="font-mono leading-none font-bold text-gray-900">
                {displayValue !== undefined && displayValue !== null
                  ? renderValue
                    ? renderValue(displayValue)
                    : displayValue.toFixed(3)
                  : "N/A"}
              </span>
              <span className="leading-none text-gray-500">
                {renderUnitSymbol(unit)}
              </span>
            </div>
          </div>
        </div>

        <div className="-mt-2 px-6">
          <div className="h-px bg-gray-200"></div>
        </div>

        <div className="flex-1 overflow-hidden rounded-b-3xl pt-4">
          <div
            ref={containerRef}
            className="h-full w-full overflow-hidden"
            style={{ backgroundColor: colors.background }}
          />
        </div>
      </div>
    </div>
  );
}

// Convenience wrapper for diameter graphs
export function DiameterGraph({
  newData,
  threshold1,
  threshold2,
  target,
  unit,
  renderValue,
  syncState,
  syncCallbacks,
}: {
  newData: TimeSeries | null;
  threshold1: number;
  threshold2: number;
  target: number;
  unit?: Unit;
  renderValue?: (value: number) => string;
  syncState?: PropSyncState;
  syncCallbacks?: PropSyncCallbacks;
}) {
  const config: GraphConfig = {
    title: "Diameter",
    lines: [
      {
        type: "threshold",
        value: threshold1,
        label: "Upper Threshold",
        color: "#ef4444",
        dash: [5, 5],
      },
      {
        type: "threshold",
        value: threshold2,
        label: "Lower Threshold",
        color: "#f97316",
        dash: [5, 5],
      },
      {
        type: "target",
        value: target,
        label: "Target",
        color: "#6b7280",
      },
    ],
    colors: {
      primary: "#3b82f6",
      grid: "#e2e8f0",
      axis: "#64748b",
      background: "#ffffff",
    },
    exportFilename: "diameter_data",
  };

  return (
    <BigGraph
      newData={newData}
      unit={unit}
      renderValue={renderValue}
      config={config}
      graphId="diameter-main"
      syncState={syncState}
      syncCallbacks={syncCallbacks}
    />
  );
}
