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
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { GraphExportData, exportGraphsToExcel } from "./excel_helpers";
// Sync Context for synchronized zooming
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
};
const GraphSyncContext = createContext<GraphSyncContextType | null>(null);

export function GraphSyncProvider({
  children,
  groupId,
}: {
  children: ReactNode;
  groupId: string;
}) {
  const graphsRef = useRef<Map<string, (state: SyncState) => void>>(new Map());
  const graphDataRef = useRef<Map<string, () => GraphExportData | null>>(
    new Map(),
  );

  // Add this state to track when graphs are registered
  const [hasRegisteredGraphs, setHasRegisteredGraphs] = useState(false);

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
    // Update state when graphs are registered
    setHasRegisteredGraphs(graphDataRef.current.size > 0);
  };

  const unregisterGraphData = (id: string) => {
    graphDataRef.current.delete(id);
    // Update state when graphs are unregistered
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
    graphsRef.current.forEach((updateFn, id) => {
      if (id !== fromId) {
        updateFn({ x: { min: 0, max: 0 }, viewMode, isLiveMode });
      }
    });
  };

  const exportAllGraphs = () => {
    exportGraphsToExcel(graphDataRef.current, groupId);
  };

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

export function FloatingExportButton({ groupId }: { groupId: string }) {
  const graphSync = useGraphSync();

  // Use the state-based approach instead of the function call
  if (!graphSync || !graphSync.hasRegisteredGraphs) return null;

  return (
    <div className="fixed right-10 bottom-6 z-50">
      <TouchButton
        onClick={graphSync.exportAllGraphs}
        variant="outline"
        className="bg-green-600 px-3 py-2 text-base font-medium text-white transition-colors hover:bg-green-100"
      >
        Export
      </TouchButton>
    </div>
  );
}

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
  description?: string;
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
  syncGroupId?: string;
  graphId: string;
};

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

export function BigGraph({
  newData,
  unit,
  renderValue,
  config,
  syncGroupId,
  graphId,
}: BigGraphProps) {
  const containerRef = useRef<HTMLDivElement | null>(null);
  const uplotRef = useRef<uPlot | null>(null);
  const chartCreatedRef = useRef(false);

  const [viewMode, setViewMode] = useState<"default" | "all" | "manual">(
    "default",
  );
  const [isLiveMode, setIsLiveMode] = useState(true);
  const [selectedTimeWindow, setSelectedTimeWindow] = useState<number | "all">(
    config.defaultTimeWindow ?? 30 * 60 * 1000,
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

  // Sync context
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

  // Animation constants
  const POINT_ANIMATION_DURATION = 1000;
  const timeWindowOptions = config.timeWindows ?? DEFAULT_TIME_WINDOW_OPTIONS;
  const colors = {
    primary: config.colors?.primary ?? "#3b82f6",
    grid: config.colors?.grid ?? "#e2e8f0",
    axis: config.colors?.axis ?? "#64748b",
    background: config.colors?.background ?? "#ffffff",
  };

  const handleSyncUpdate = useCallback(
    (state: SyncState) => {
      if (!uplotRef.current) return;

      // Temporarily disable user zoom detection during sync
      const wasUserZooming = isUserZoomingRef.current;
      isUserZoomingRef.current = false;

      if (state.viewMode !== undefined && state.isLiveMode !== undefined) {
        // Sync view mode
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
        // Sync time window
        setSelectedTimeWindow(state.timeWindow);
        handleTimeWindowChangeInternal(state.timeWindow, true);
      } else {
        // Sync zoom - this is the key fix
        setViewMode("manual");
        setIsLiveMode(false);

        // Use batch to ensure atomic update
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

      // Restore the original user zoom state
      setTimeout(() => {
        isUserZoomingRef.current = wasUserZooming;
      }, 10);
    },
    [newData, selectedTimeWindow],
  );

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

  // Register with sync context
  useEffect(() => {
    if (graphSync) {
      graphSync.registerGraph(graphId, handleSyncUpdate);
      return () => graphSync.unregisterGraph(graphId);
    }
  }, [graphSync, graphId, handleSyncUpdate]);

  // Linear interpolation function
  const lerp = (start: number, end: number, t: number): number => {
    return start + (end - start) * t;
  };

  // Animate new point addition
  const animateNewPoint = (
    currentData: { timestamps: number[]; values: number[] },
    targetData: { timestamps: number[]; values: number[] },
  ) => {
    if (targetData.timestamps.length <= currentData.timestamps.length) {
      return;
    }

    // Only animate to the exact next point - no prediction
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

      // Use current data as base, don't modify it
      const animatedTimestamps = [...currentData.timestamps];
      const animatedValues = [...currentData.values];

      if (progress < 1) {
        // Animate only to the exact target point
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
        // Animation complete - add exactly the target point
        animatedTimestamps.push(animationStateRef.current.toTimestamp);
        animatedValues.push(animationStateRef.current.toValue);

        // Update rendered data to this exact point
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

      // Update view if in live mode
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
        // Check if there are more points to animate (only immediate next point)
        if (targetData.timestamps.length > animatedTimestamps.length) {
          // Animate the next single point
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
      // Get values that are actually visible in the time window
      for (let i = 0; i < timestamps.length; i++) {
        if (timestamps[i] >= xMin && timestamps[i] <= xMax) {
          visibleValues.push(values[i]);
        }
      }
    } else {
      // Use all values if no time window specified
      visibleValues = [...values];
    }

    // Include line values for proper scaling
    config.lines?.forEach((line) => {
      if (line.show !== false) {
        visibleValues.push(line.value);
      }
    });

    if (visibleValues.length === 0) {
      // Fallback if no visible values
      visibleValues = values;
    }

    const minY = Math.min(...visibleValues);
    const maxY = Math.max(...visibleValues);
    const range = maxY - minY || Math.abs(maxY) * 0.1 || 1;

    // Add 10% padding above and below
    const yRange = {
      min: minY - range * 0.1,
      max: maxY + range * 0.1,
    };

    // Force Y-axis to update with new range - use batch update
    uplotRef.current.batch(() => {
      uplotRef.current!.setScale("y", yRange);
    });

    // Update manual scale reference if in manual mode
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

    // Calculate initial Y range based on VISIBLE data, not all data
    let initialYMin: number, initialYMax: number;

    // Get values that are visible in the initial time window
    const initialVisibleValues: number[] = [];
    for (let i = 0; i < timestamps.length; i++) {
      if (timestamps[i] >= initialMin && timestamps[i] <= initialMax) {
        initialVisibleValues.push(values[i]);
      }
    }

    // Include line values
    config.lines?.forEach((line) => {
      if (line.show !== false) {
        initialVisibleValues.push(line.value);
      }
    });

    // If no visible values, fallback to all values
    if (initialVisibleValues.length === 0) {
      initialVisibleValues.push(...values);
      config.lines?.forEach((line) => {
        if (line.show !== false) {
          initialVisibleValues.push(line.value);
        }
      });
    }

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
            // This ensures linear time scaling - equal intervals = equal distance
            distr: 1, // Linear distribution
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

                // For very short ranges (≤ 30s), show full time with seconds
                if (timeRange <= 30 * 1000) {
                  return date.toLocaleTimeString("en-GB", {
                    hour12: false,
                    hour: "2-digit",
                    minute: "2-digit",
                    second: "2-digit",
                  });
                }
                // For short ranges (≤ 5min), show time with seconds
                else if (timeRange <= 5 * 60 * 1000) {
                  return date.toLocaleTimeString("en-GB", {
                    hour12: false,
                    hour: "2-digit",
                    minute: "2-digit",
                    second: "2-digit",
                  });
                }
                // For medium ranges (≤ 1hour), show HH:MM
                else if (timeRange <= 60 * 60 * 1000) {
                  return date.toLocaleTimeString("en-GB", {
                    hour12: false,
                    hour: "2-digit",
                    minute: "2-digit",
                  });
                }
                // For longer ranges, show HH:MM
                else {
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

              // For ≤ 10 seconds: every 1 second
              if (timeRange <= 10 * 1000) {
                const startTime = Math.ceil(scaleMin / 1000) * 1000;
                for (let t = startTime; t <= scaleMax; t += 1000) {
                  if (t >= scaleMin) ticks.push(t);
                }
              }
              // For ≤ 30 seconds: every 5 seconds
              else if (timeRange <= 30 * 1000) {
                const startTime = Math.ceil(scaleMin / 5000) * 5000;
                for (let t = startTime; t <= scaleMax; t += 5000) {
                  if (t >= scaleMin) ticks.push(t);
                }
              }
              // For ≤ 1 minute: every 10 seconds
              else if (timeRange <= 60 * 1000) {
                const startTime = Math.ceil(scaleMin / 10000) * 10000;
                for (let t = startTime; t <= scaleMax; t += 10000) {
                  if (t >= scaleMin) ticks.push(t);
                }
              }
              // For ≤ 5 minutes: every 30 seconds
              else if (timeRange <= 5 * 60 * 1000) {
                const startTime = Math.ceil(scaleMin / 30000) * 30000;
                for (let t = startTime; t <= scaleMax; t += 30000) {
                  if (t >= scaleMin) ticks.push(t);
                }
              }
              // For ≤ 10 minutes: every 1 minute
              else if (timeRange <= 10 * 60 * 1000) {
                const startTime = Math.ceil(scaleMin / 60000) * 60000;
                for (let t = startTime; t <= scaleMax; t += 60000) {
                  if (t >= scaleMin) ticks.push(t);
                }
              }
              // For ≤ 30 minutes: every 2 minutes
              else if (timeRange <= 30 * 60 * 1000) {
                const startTime = Math.ceil(scaleMin / 120000) * 120000;
                for (let t = startTime; t <= scaleMax; t += 120000) {
                  if (t >= scaleMin) ticks.push(t);
                }
              }
              // For ≤ 1 hour: every 5 minutes
              else if (timeRange <= 60 * 60 * 1000) {
                const startTime = Math.ceil(scaleMin / 300000) * 300000;
                for (let t = startTime; t <= scaleMax; t += 300000) {
                  if (t >= scaleMin) ticks.push(t);
                }
              }
              // For ≤ 6 hours: every 30 minutes
              else if (timeRange <= 6 * 60 * 60 * 1000) {
                const startTime = Math.ceil(scaleMin / 1800000) * 1800000;
                for (let t = startTime; t <= scaleMax; t += 1800000) {
                  if (t >= scaleMin) ticks.push(t);
                }
              }
              // For longer ranges: every 1 hour
              else {
                const startTime = Math.ceil(scaleMin / 3600000) * 3600000;
                for (let t = startTime; t <= scaleMax; t += 3600000) {
                  if (t >= scaleMin) ticks.push(t);
                }
              }

              return ticks;
            },
          },

          // Y-axis remains the same
          {
            stroke: colors.axis,
            labelSize: 12,
            labelFont: "Inter, system-ui, sans-serif",
            grid: { stroke: colors.grid, width: 1 },
            side: 1,
            space: 60,
            values: (_u, ticks) => {
              // First try with renderValue function
              if (renderValue) {
                const renderedValues = ticks.map(renderValue);
                const uniqueValues = new Set(renderedValues);

                // If no duplicates, use renderValue
                if (uniqueValues.size === renderedValues.length) {
                  return renderedValues;
                }
              }

              // If renderValue creates duplicates or doesn't exist, find optimal precision
              const precision = 0;
              const maxPrecision = 4;

              for (let p = precision; p <= maxPrecision; p++) {
                const formattedValues = ticks.map((v) => v.toFixed(p));
                const uniqueFormatted = new Set(formattedValues);

                // If all values are unique at this precision, use it
                if (uniqueFormatted.size === formattedValues.length) {
                  return formattedValues;
                }
              }

              // Fallback: use maximum precision
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

      // Only handle multi-touch gestures - let single touch scroll normally
      if (e.touches.length === 2) {
        // Two fingers - pinch to zoom
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

        e.preventDefault(); // Only prevent for 2+ finger gestures
      }
      // For single finger, don't prevent default - allow normal scrolling
    };

    const handleTouchMove = (e: TouchEvent) => {
      if (!touchStartRef.current) return;

      if (e.touches.length === 1) {
        // Single finger - ONLY interfere if it's a very clear horizontal drag attempt
        const touch = e.touches[0];
        const deltaX = Math.abs(touch.clientX - touchStartRef.current.x);
        const deltaY = Math.abs(touch.clientY - touchStartRef.current.y);
        const timeDelta = Date.now() - touchStartRef.current.time;

        // Much more strict criteria for chart interaction
        if (
          touchDirectionRef.current === "unknown" &&
          deltaX > 20 && // Higher threshold
          deltaY < 10 && // Very low vertical tolerance
          deltaX > deltaY * 4 && // Much stricter ratio
          timeDelta < 500 // Must be quick movement
        ) {
          touchDirectionRef.current = "horizontal";
          isDraggingRef.current = true;
          lastDragXRef.current = touch.clientX;
          e.preventDefault(); // Only now prevent default
        } else if (touchDirectionRef.current === "unknown") {
          // Any other movement - allow normal page scrolling
          return;
        }

        // Handle chart dragging only if we've confirmed it's horizontal
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

              if (graphSync) {
                graphSync.syncZoom(graphId, {
                  x: { min: newMin, max: newMax },
                });
              }
            }
          }
        }
      } else if (e.touches.length === 2 && isPinchingRef.current) {
        // Two finger pinch zoom
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
      // For all other cases, don't prevent default - allow normal scrolling
    };

    const handleTouchEnd = (e: TouchEvent) => {
      if (e.touches.length === 0) {
        // All fingers lifted - reset everything
        isDraggingRef.current = false;
        isPinchingRef.current = false;
        lastDragXRef.current = null;
        lastPinchDistanceRef.current = null;
        pinchCenterRef.current = null;
        touchStartRef.current = null;
        touchDirectionRef.current = "unknown";
      } else if (e.touches.length === 1 && isPinchingRef.current) {
        // Went from pinch to single finger
        isPinchingRef.current = false;
        lastPinchDistanceRef.current = null;
        pinchCenterRef.current = null;

        // Reset for potential new interaction
        const touch = e.touches[0];
        touchStartRef.current = {
          x: touch.clientX,
          y: touch.clientY,
          time: Date.now(),
        };
        touchDirectionRef.current = "unknown";
        isDraggingRef.current = false;
      }

      // Only prevent default if we actually intercepted the gesture
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
      // Touch events with more permissive settings
      containerRef.current.addEventListener("touchstart", handleTouchStart, {
        passive: false,
      });
      containerRef.current.addEventListener("touchmove", handleTouchMove, {
        passive: false,
      });
      containerRef.current.addEventListener("touchend", handleTouchEnd, {
        passive: false,
      });

      // Mouse events
      containerRef.current.addEventListener("mousedown", handleMouseDown);

      // Remove or modify wheel handler to be less aggressive
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

  const switchToLiveMode = () => {
    isSwitchingModeRef.current = true;
    stopAnimations();

    const currentTimeWindow = selectedTimeWindow;

    setIsLiveMode(true);

    if (currentTimeWindow === "all") {
      setViewMode("all");
    } else {
      setViewMode("default");
    }

    // Broadcast mode change to other graphs
    if (graphSync && !isSyncingRef.current) {
      graphSync.syncViewMode(
        graphId,
        currentTimeWindow === "all" ? "all" : "default",
        true,
      );
    }

    if (uplotRef.current && newData?.long) {
      const [timestamps, values] = seriesToUPlotData(newData.long);
      const fullData = buildUPlotData(timestamps, values);
      uplotRef.current.setData(fullData);

      if (timestamps.length > 0) {
        const latestTimestamp = timestamps[timestamps.length - 1];

        if (currentTimeWindow === "all") {
          const fullStart = startTimeRef.current ?? timestamps[0];
          uplotRef.current.setScale("x", {
            min: fullStart,
            max: latestTimestamp,
          });
          updateYAxisScale(timestamps, values, fullStart, latestTimestamp);
        } else {
          const viewStart = latestTimestamp - (currentTimeWindow as number);
          uplotRef.current.setScale("x", {
            min: viewStart,
            max: latestTimestamp,
          });
          updateYAxisScale(timestamps, values, viewStart, latestTimestamp);
        }

        manualScaleRef.current = null;
      }

      lastRenderedDataRef.current = { timestamps, values };
    }

    setTimeout(() => {
      isSwitchingModeRef.current = false;
    }, 100);
  };

  const switchToHistoricalMode = () => {
    isSwitchingModeRef.current = true;
    stopAnimations();

    setIsLiveMode(false);
    setViewMode("manual");

    // Broadcast mode change to other graphs
    if (graphSync && !isSyncingRef.current) {
      graphSync.syncViewMode(graphId, "manual", false);
    }

    if (uplotRef.current && uplotRef.current.scales) {
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
          x: { min: xScale.min, max: xScale.max },
          y: { min: yScale.min, max: yScale.max },
        };
      }
    }

    if (uplotRef.current && newData?.long) {
      const [timestamps, values] = seriesToUPlotData(newData.long);
      const fullData = buildUPlotData(timestamps, values);
      uplotRef.current.setData(fullData);
      lastRenderedDataRef.current = { timestamps, values };
    }

    setTimeout(() => {
      isSwitchingModeRef.current = false;
    }, 100);
  };

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
        // Historical mode - this is the key fix to match your simple version
        const rightmostTimestamp = getRightmostVisibleTimestamp();
        if (rightmostTimestamp) {
          const newViewStart = rightmostTimestamp - newTimeWindow;
          uplotRef.current.setScale("x", {
            min: newViewStart,
            max: rightmostTimestamp,
          });
          // This is the crucial call - auto-scale Y-axis for the new time window
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

    // Broadcast time window change to other graphs
    if (graphSync && !isSync && !isSyncingRef.current) {
      graphSync.syncTimeWindow(graphId, newTimeWindow);
    }

    if (!isSync) {
      setTimeout(() => {
        isUserTimeWindowChangeRef.current = false;
      }, 50);
    }
  };

  // Public time window change function
  const handleTimeWindowChange = (newTimeWindow: number | "all") => {
    handleTimeWindowChangeInternal(newTimeWindow, false);
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

    // Only animate if we have new points, and only animate to existing data
    if (targetData.timestamps.length > currentData.timestamps.length) {
      // Ensure we don't animate beyond actual data
      const maxAnimatableLength = Math.min(
        targetData.timestamps.length,
        currentData.timestamps.length + 1, // Only one point ahead at a time
      );

      const limitedTargetData = {
        timestamps: targetData.timestamps.slice(0, maxAnimatableLength),
        values: targetData.values.slice(0, maxAnimatableLength),
      };

      animateNewPoint(currentData, limitedTargetData);
    } else if (targetData.timestamps.length === currentData.timestamps.length) {
      // Handle value updates without animation
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

        // Update view scales
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

  // Live data updates effect
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

      // Always add the current point for live updates
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

        // Force immediate scale update to prevent time gaps
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

  const getSelectedTimeWindowLabel = () => {
    const option = timeWindowOptions.find(
      (opt) => opt.value === selectedTimeWindow,
    );
    return option ? option.label : "1m";
  };

  return (
    <div className="h-[50vh] w-full">
      <div className="flex h-full w-full flex-col overflow-hidden rounded-3xl border border-gray-200 bg-white shadow">
        {/* Header */}
        <div className="flex items-center justify-between pt-4 pr-5 pb-6 pl-6">
          {/* Left side - Icon, Title, Current value */}
          <div className="-mt-1 flex items-center gap-4">
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
          {/* Right side - Time window dropdown, View buttons */}
          <div className="flex items-center gap-4">
            {/* Time window dropdown */}
            <DropdownMenu>
              <DropdownMenuTrigger asChild>
                <TouchButton
                  variant="outline"
                  className="border-gray-300 px-3 py-2 text-base font-medium text-gray-900 hover:bg-gray-50"
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
                    onClick={() => handleTimeWindowChange(option.value)}
                    className={`min-h-[48px] px-4 py-3 text-base ${
                      selectedTimeWindow === option.value ? "bg-blue-50" : ""
                    }`}
                  >
                    {option.label}
                  </DropdownMenuItem>
                ))}
              </DropdownMenuContent>
            </DropdownMenu>

            {/* View Buttons */}
            <div className="flex items-center gap-4">
              <TouchButton
                onClick={switchToHistoricalMode}
                variant="outline"
                className={`px-3 py-2 text-base font-medium transition-colors ${
                  !isLiveMode
                    ? "bg-black text-white shadow-sm"
                    : "border-gray-300 text-gray-700 hover:bg-gray-100"
                }`}
              >
                Historical
              </TouchButton>
              <TouchButton
                onClick={switchToLiveMode}
                variant="outline"
                className={`px-3 py-2 text-base font-medium transition-colors ${
                  isLiveMode
                    ? "bg-black text-white shadow-sm"
                    : "border-gray-300 text-gray-700 hover:bg-gray-100"
                }`}
              >
                Live
              </TouchButton>
            </div>
          </div>
        </div>

        {/* Separator line with padding */}
        <div className="-mt-2 px-6">
          <div className="h-px bg-gray-200"></div>
        </div>

        {/* Graph Container - full width with only vertical padding */}
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
}: {
  newData: TimeSeries | null;
  threshold1: number;
  threshold2: number;
  target: number;
  unit?: Unit;
  renderValue?: (value: number) => string;
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
      syncGroupId="diameter-group"
      graphId="diameter-main"
    />
  );
}
