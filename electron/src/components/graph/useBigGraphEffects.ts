import { useEffect, RefObject, useRef, useState } from "react";
import uPlot from "uplot";
import { seriesToUPlotData, TimeSeries } from "@/lib/timeseries";
import { BigGraphProps, DataSeries, SeriesData } from "./types";
import { GraphExportData } from "./excelExport";
import { AnimationRefs, stopAnimations } from "./animation";
import { HandlerRefs } from "./handlers";
import { createChart } from "./createChart";
import { LiveModeHandlers } from "./liveMode";
import { HistoricalModeHandlers } from "./historicalMode";

interface UseBigGraphEffectsProps {
  // Refs
  containerRef: RefObject<HTMLDivElement | null>;
  uplotRef: RefObject<uPlot | null>;
  startTimeRef: RefObject<number | null>;
  manualScaleRef: React.MutableRefObject<{
    x: { min: number; max: number };
    y: { min: number; max: number };
  } | null>;
  lastProcessedCountRef: React.MutableRefObject<number>;
  animationRefs: AnimationRefs;
  handlerRefs: HandlerRefs;
  chartCreatedRef: React.MutableRefObject<boolean>;

  // Props
  newData: BigGraphProps["newData"];
  unit: BigGraphProps["unit"];
  renderValue: BigGraphProps["renderValue"];
  config: BigGraphProps["config"];
  graphId: BigGraphProps["graphId"];
  syncGraph: BigGraphProps["syncGraph"];
  onRegisterForExport?: (
    graphId: string,
    getDataFn: () => GraphExportData | null,
  ) => void;
  onUnregisterFromExport?: (graphId: string) => void;

  // State
  viewMode: "default" | "all" | "manual";
  isLiveMode: boolean;
  selectedTimeWindow: number | "all";

  // State setters
  setSelectedTimeWindow: (value: number | "all") => void;
  setViewMode: React.Dispatch<
    React.SetStateAction<"default" | "all" | "manual">
  >;
  setIsLiveMode: React.Dispatch<React.SetStateAction<boolean>>;
  setCursorValue: React.Dispatch<React.SetStateAction<number | null>>;

  // Handlers
  liveMode: LiveModeHandlers;
  historicalMode: HistoricalModeHandlers;
  colors: {
    primary: string;
    grid: string;
    axis: string;
    background: string;
  };
  updateYAxisScale: (
    timestamps: number[],
    values: number[],
    xMin?: number,
    xMax?: number,
  ) => void;
  handleTimeWindowChangeInternal: (
    newTimeWindow: number | "all",
    isSync?: boolean,
  ) => void;
}

// Helper function to normalize data to array format
function normalizeDataSeries(data: BigGraphProps["newData"]): SeriesData[] {
  if (Array.isArray(data)) {
    return data;
  }
  return [data];
}

// Helper function to get the primary series (first valid one)
function getPrimarySeries(data: BigGraphProps["newData"]): SeriesData | null {
  const normalized = normalizeDataSeries(data);
  return normalized.find((series) => series.newData !== null) || null;
}

export function useBigGraphEffects({
  containerRef,
  uplotRef,
  startTimeRef,
  manualScaleRef,
  lastProcessedCountRef,
  animationRefs,
  handlerRefs,
  chartCreatedRef,

  newData,
  unit,
  renderValue,
  config,
  graphId,
  syncGraph,
  onRegisterForExport,
  onUnregisterFromExport,

  viewMode,
  isLiveMode,
  selectedTimeWindow,

  setSelectedTimeWindow,
  setViewMode,
  setIsLiveMode,
  setCursorValue,

  liveMode,
  historicalMode,
  colors,
  updateYAxisScale,
  handleTimeWindowChangeInternal,
}: UseBigGraphEffectsProps) {
  const [isChartCreated, setIsChartCreated] = useState(false);
  const localManualScale = useRef(manualScaleRef.current);
  const localProcessedCount = useRef(lastProcessedCountRef.current);
  const localUplotRef = useRef<uPlot | null>(uplotRef.current);
  const localChart = useRef(chartCreatedRef.current);

  // Sync state tracking
  const lastSyncStateRef = useRef({
    timeWindow: selectedTimeWindow,
    viewMode,
    isLiveMode,
    xRange: syncGraph?.xRange,
  });

  // Get primary series for compatibility with existing logic
  const primarySeries = getPrimarySeries(newData);

  // Register/unregister this graph for Excel export functionality
  useEffect(() => {
    if (onRegisterForExport && primarySeries?.newData) {
      const getExportData = (): GraphExportData | null => {
        if (!primarySeries?.newData) return null;

        return {
          config,
          data: primarySeries as unknown as DataSeries,
          unit,
          renderValue,
        };
      };

      onRegisterForExport(graphId, getExportData);

      return () => {
        if (onUnregisterFromExport) {
          onUnregisterFromExport(graphId);
        }
      };
    }
  }, [
    graphId,
    primarySeries?.newData,
    config,
    unit,
    renderValue,
    onRegisterForExport,
    onUnregisterFromExport,
  ]);

  // Sync graph state with external sync props
  useEffect(() => {
    if (!syncGraph) return;

    const lastState = lastSyncStateRef.current;
    let hasChanges = false;

    // Check for time window changes
    if (syncGraph.timeWindow !== lastState.timeWindow) {
      setSelectedTimeWindow(syncGraph.timeWindow);
      handleTimeWindowChangeInternal(syncGraph.timeWindow, true);
      hasChanges = true;
    }

    // Check for view mode changes
    if (syncGraph.viewMode !== lastState.viewMode) {
      setViewMode(syncGraph.viewMode);
      hasChanges = true;
    }

    // Check for live mode changes
    if (syncGraph.isLiveMode !== lastState.isLiveMode) {
      const newIsLiveMode = syncGraph.isLiveMode;

      if (lastState.isLiveMode && !newIsLiveMode) {
        historicalMode.switchToHistoricalMode();
      } else if (!lastState.isLiveMode && newIsLiveMode) {
        historicalMode.switchToLiveMode();

        // Add a longer delay to ensure proper state transition
        setTimeout(() => {
          if (liveMode.processNewHistoricalData) {
            liveMode.processNewHistoricalData();
          }
        }, 200);
      }

      setIsLiveMode(newIsLiveMode);
      hasChanges = true;
    }

    // Check for zoom range changes
    const xRangeChanged =
      syncGraph.xRange &&
      (!lastState.xRange ||
        syncGraph.xRange.min !== lastState.xRange.min ||
        syncGraph.xRange.max !== lastState.xRange.max);

    if (xRangeChanged && uplotRef.current && primarySeries?.newData?.long) {
      uplotRef.current.batch(() => {
        uplotRef.current!.setScale("x", {
          min: syncGraph.xRange?.min ?? 0,
          max: syncGraph.xRange?.max ?? 0,
        });

        if (primarySeries.newData?.long) {
          const [timestamps, values] = seriesToUPlotData(
            primarySeries.newData.long,
          );
          updateYAxisScale(
            timestamps,
            values,
            syncGraph.xRange?.min ?? 0,
            syncGraph.xRange?.max ?? 0,
          );
        }
      });

      localManualScale.current = {
        x: syncGraph.xRange ?? { min: 0, max: 1 },
        y: localManualScale.current?.y ?? { min: 0, max: 1 },
      };
      hasChanges = true;
    }

    if (hasChanges) {
      if (syncGraph.viewMode === "manual") {
        setViewMode("manual");
        setIsLiveMode(false);
        stopAnimations(animationRefs);
        localProcessedCount.current = 0;
      }

      // Update last sync state
      lastSyncStateRef.current = {
        timeWindow: syncGraph.timeWindow,
        viewMode: syncGraph.viewMode,
        isLiveMode: syncGraph.isLiveMode,
        xRange: syncGraph.xRange,
      };
    }
  }, [
    syncGraph?.timeWindow,
    syncGraph?.viewMode,
    syncGraph?.isLiveMode,
    syncGraph?.xRange?.min,
    syncGraph?.xRange?.max,
    historicalMode.switchToHistoricalMode,
    historicalMode.switchToLiveMode,
    setSelectedTimeWindow,
    handleTimeWindowChangeInternal,
    setViewMode,
    setIsLiveMode,
    uplotRef,
    primarySeries?.newData?.long,
    updateYAxisScale,
    manualScaleRef,
    animationRefs,
    lastProcessedCountRef,
    liveMode.processNewHistoricalData,
  ]);

  // Create and initialize the uPlot chart when data becomes available
  useEffect(() => {
    if (!containerRef.current || !primarySeries?.newData?.long) {
      setIsChartCreated(false);
      localChart.current = false;
      return;
    }

    const [timestamps] = seriesToUPlotData(primarySeries.newData.long);
    if (timestamps.length === 0) {
      setIsChartCreated(false);
      localChart.current = false;
      return;
    }

    const cleanup = createChart({
      containerRef,
      uplotRef,
      newData,
      config,
      colors,
      renderValue,
      viewMode,
      selectedTimeWindow,
      isLiveMode,
      startTimeRef,
      manualScaleRef,
      animationRefs,
      handlerRefs,
      graphId,
      syncGraph,
      getHistoricalEndTimestamp: historicalMode.getHistoricalEndTimestamp,
      updateYAxisScale,
      setViewMode,
      setIsLiveMode,
      setCursorValue,
    });

    setIsChartCreated(true);
    localChart.current = true;

    if (isLiveMode) {
      localProcessedCount.current = timestamps.length;
    }

    return () => {
      if (cleanup) cleanup();
      if (localUplotRef.current) {
        localUplotRef.current.destroy();
        localUplotRef.current = null;
      }
      stopAnimations(animationRefs);
      setIsChartCreated(false);
      localChart.current = false;
    };
  }, [
    primarySeries?.newData?.long,
    containerRef,
    uplotRef,
    newData,
    config,
    colors,
    renderValue,
    viewMode,
    selectedTimeWindow,
    isLiveMode,
    startTimeRef,
    manualScaleRef,
    animationRefs,
    handlerRefs,
    graphId,
    syncGraph,
    historicalMode.getHistoricalEndTimestamp,
    updateYAxisScale,
    setViewMode,
    setIsLiveMode,
    setCursorValue,
    chartCreatedRef,
    lastProcessedCountRef,
  ]);

  // Update chart data when new historical data arrives (live mode only)
  useEffect(() => {
    if (!isLiveMode || viewMode === "manual" || !isChartCreated) return;

    // Add a delay to ensure chart is ready after mode switch
    const timeoutId = setTimeout(() => {
      liveMode.processNewHistoricalData();
    }, 100);

    return () => clearTimeout(timeoutId);
  }, [
    primarySeries?.newData?.long?.validCount,
    primarySeries?.newData?.long?.lastTimestamp,
    viewMode,
    selectedTimeWindow,
    isLiveMode,
    isChartCreated,
    liveMode.processNewHistoricalData,
  ]);

  // Update chart with real-time current data point (live mode only)
  useEffect(() => {
    if (
      !uplotRef.current ||
      !primarySeries?.newData?.current ||
      !isLiveMode ||
      !isChartCreated ||
      animationRefs.animationState.current.isAnimating ||
      viewMode === "manual"
    )
      return;

    liveMode.updateLiveData();
  }, [
    primarySeries?.newData?.current?.timestamp,
    viewMode,
    selectedTimeWindow,
    config.lines,
    isLiveMode,
    isChartCreated,
    liveMode.updateLiveData,
    uplotRef,
    primarySeries?.newData?.current,
    animationRefs.animationState,
  ]);

  // Suppress unused variable warnings for local refs
  void localManualScale;
  void localProcessedCount;
  void localUplotRef;
}
