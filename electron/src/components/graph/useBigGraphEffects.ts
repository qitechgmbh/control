/* eslint-disable react-compiler/react-compiler */
import { useEffect, RefObject, useRef, useState } from "react";
import uPlot from "uplot";
import { seriesToUPlotData } from "@/lib/timeseries";
import { BigGraphProps, SeriesData, AnimationRefs, HandlerRefs } from "./types";
import { GraphExportData } from "./excelExport";
import { getPrimarySeries, stopAnimations } from "./animation";
import { createChart } from "./createChart";
import { LiveModeHandlers } from "./liveMode";
import { HistoricalModeHandlers } from "./historicalMode";

interface UseBigGraphEffectsProps {
  // Refs
  containerRef: RefObject<HTMLDivElement | null>;
  uplotRef: RefObject<uPlot | null>;
  startTimeRef: RefObject<number | null>;
  manualScaleRef: RefObject<{
    x: { min: number; max: number };
    y: { min: number; max: number };
  } | null>;
  lastProcessedCountRef: RefObject<number>;
  animationRefs: AnimationRefs;
  handlerRefs: HandlerRefs;
  chartCreatedRef: RefObject<boolean>;

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
  visibleSeries: boolean[];
  showFromTimestamp?: number | null;

  // State setters
  setSelectedTimeWindow: (value: number | "all") => void;
  setViewMode: React.Dispatch<
    React.SetStateAction<"default" | "all" | "manual">
  >;
  setIsLiveMode: React.Dispatch<React.SetStateAction<boolean>>;
  setCursorValue: React.Dispatch<React.SetStateAction<number | null>>;
  setCursorValues: React.Dispatch<React.SetStateAction<(number | null)[]>>;

  // Handlers
  liveMode: LiveModeHandlers;
  historicalMode: HistoricalModeHandlers;
  colors: {
    primary: string;
    grid: string;
    axis: string;
    background: string;
  };
  updateYAxisScale: (xMin?: number, xMax?: number) => void;
  handleTimeWindowChangeInternal: (
    newTimeWindow: number | "all",
    isSync?: boolean,
  ) => void;
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
  visibleSeries,
  showFromTimestamp,

  setSelectedTimeWindow,
  setViewMode,
  setIsLiveMode,
  setCursorValue,
  setCursorValues,

  liveMode,
  historicalMode,
  colors,
  updateYAxisScale,
  handleTimeWindowChangeInternal,
}: UseBigGraphEffectsProps) {
  const [isChartCreated, setIsChartCreated] = useState(false);

  // Track the last synchronized state for comparison
  const lastSyncStateRef = useRef({
    timeWindow: selectedTimeWindow,
    viewMode,
    isLiveMode,
    xRange: syncGraph?.xRange,
  });

  // Get the primary series for compatibility with existing logic
  const primarySeries = getPrimarySeries(newData);

  // Register the graph for Excel export functionality
  useEffect(() => {
    if (onRegisterForExport && primarySeries?.newData) {
      const getExportData = (): GraphExportData | null => {
        if (!primarySeries?.newData) return null;

        return {
          config,
          data: {
            newData: primarySeries?.newData ?? null,
            title: primarySeries?.title,
            color: primarySeries?.color,
            lines: primarySeries?.lines,
          } as SeriesData,
          unit,
          renderValue,
        };
      };

      onRegisterForExport(graphId, getExportData);

      return () => {
        onUnregisterFromExport?.(graphId);
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

  // Synchronize graph state with external sync properties
  useEffect(() => {
    if (!syncGraph) return;

    const lastState = lastSyncStateRef.current;
    let hasChanges = false;

    if (syncGraph.timeWindow !== lastState.timeWindow) {
      setSelectedTimeWindow(syncGraph.timeWindow);
      handleTimeWindowChangeInternal(syncGraph.timeWindow, true);
      hasChanges = true;
    }

    if (syncGraph.viewMode !== lastState.viewMode) {
      setViewMode(syncGraph.viewMode);
      hasChanges = true;
    }

    if (syncGraph.isLiveMode !== lastState.isLiveMode) {
      const newIsLiveMode = syncGraph.isLiveMode;

      if (lastState.isLiveMode && !newIsLiveMode) {
        historicalMode.switchToHistoricalMode();
      } else if (!lastState.isLiveMode && newIsLiveMode) {
        historicalMode.switchToLiveMode();

        setTimeout(() => {
          liveMode.processNewHistoricalData?.();
        }, 200);
      }

      setIsLiveMode(newIsLiveMode);
      hasChanges = true;
    }

    const xRangeChanged =
      syncGraph.xRange &&
      (!lastState.xRange ||
        syncGraph.xRange.min !== lastState.xRange.min ||
        syncGraph.xRange.max !== lastState.xRange.max);

    if (xRangeChanged && uplotRef.current) {
      uplotRef.current.batch(() => {
        uplotRef.current!.setScale("x", {
          min: syncGraph.xRange?.min ?? 0,
          max: syncGraph.xRange?.max ?? 0,
        });

        updateYAxisScale(
          syncGraph.xRange?.min ?? 0,
          syncGraph.xRange?.max ?? 0,
        );
      });
      manualScaleRef.current = {
        x: syncGraph.xRange ?? { min: 0, max: 1 },
        y: manualScaleRef.current?.y ?? { min: 0, max: 1 },
      };
      hasChanges = true;
    }

    if (hasChanges) {
      if (syncGraph.viewMode === "manual") {
        setViewMode("manual");
        setIsLiveMode(false);
        stopAnimations(animationRefs);
        lastProcessedCountRef.current = 0;
      }

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
    updateYAxisScale,
  ]);

  // Create and initialize the chart when data becomes available
  useEffect(() => {
    if (!containerRef.current || !primarySeries?.newData?.long) {
      setIsChartCreated(false);
      chartCreatedRef.current = false;
      return;
    }

    const [timestamps] = seriesToUPlotData(primarySeries.newData.long);
    if (timestamps.length === 0) {
      setIsChartCreated(false);
      chartCreatedRef.current = false;
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
      setCursorValues,
      visibleSeries,
      showFromTimestamp,
    });

    setIsChartCreated(true);
    chartCreatedRef.current = true;

    if (isLiveMode) {
      lastProcessedCountRef.current = timestamps.length;
    }

    return () => {
      cleanup?.();
      uplotRef.current?.destroy();
      uplotRef.current = null;
      stopAnimations(animationRefs);
      setIsChartCreated(false);
      chartCreatedRef.current = false;
    };
  }, [primarySeries?.newData?.long, containerRef.current]);

  // Process new historical data in live mode
  useEffect(() => {
    if (!isLiveMode || viewMode === "manual" || !isChartCreated) return;

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

  // Update chart with real-time data in live mode
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
  ]);

  // Recalculate Y-axis scale when series visibility changes
  useEffect(() => {
    if (!uplotRef.current || !isChartCreated) return;

    const currentScale = uplotRef.current.scales.x;
    if (currentScale?.min !== undefined && currentScale?.max !== undefined) {
      updateYAxisScale(currentScale.min, currentScale.max);
    } else {
      updateYAxisScale();
    }
  }, [visibleSeries, isChartCreated, updateYAxisScale]);
}
