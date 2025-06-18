import { useEffect, RefObject } from "react";
import uPlot from "uplot";
import { seriesToUPlotData } from "@/lib/timeseries";
import { BigGraphProps } from "./types";
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
  chartCreatedRef: RefObject<boolean>;
  startTimeRef: RefObject<number | null>;
  manualScaleRef: RefObject<{
    x: { min: number; max: number };
    y: { min: number; max: number };
  } | null>;
  lastProcessedCountRef: RefObject<number>;
  animationRefs: AnimationRefs;
  handlerRefs: HandlerRefs;

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

export function useBigGraphEffects({
  // Refs
  containerRef,
  uplotRef,
  chartCreatedRef,
  startTimeRef,
  manualScaleRef,
  lastProcessedCountRef,
  animationRefs,
  handlerRefs,

  // Props
  newData,
  unit,
  renderValue,
  config,
  graphId,
  syncGraph,
  onRegisterForExport,
  onUnregisterFromExport,

  // State
  viewMode,
  isLiveMode,
  selectedTimeWindow,

  // State setters
  setSelectedTimeWindow,
  setViewMode,
  setIsLiveMode,
  setCursorValue,

  // Handlers
  liveMode,
  historicalMode,
  colors,
  updateYAxisScale,
  handleTimeWindowChangeInternal,
}: UseBigGraphEffectsProps) {
  // Register/unregister this graph for Excel export functionality
  useEffect(() => {
    if (onRegisterForExport) {
      const getExportData = (): GraphExportData | null => {
        if (!newData) return null;

        const exportData: GraphExportData = {
          config,
          data: newData,
          unit,
          renderValue,
        };

        return exportData;
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
    newData,
    config,
    unit,
    renderValue,
    onRegisterForExport,
    onUnregisterFromExport,
  ]);

  // Sync graph state with external sync props
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
      const newIsLiveMode = syncGraph.isLiveMode;

      if (isLiveMode && !newIsLiveMode) {
        // Switching live → historical
        historicalMode.switchToHistoricalMode();
      } else if (!isLiveMode && newIsLiveMode) {
        // Switching historical → live
        historicalMode.switchToLiveMode();
      }

      setIsLiveMode(newIsLiveMode);
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
      stopAnimations(animationRefs);
      lastProcessedCountRef.current = 0;
    }
  }, [
    syncGraph?.timeWindow,
    syncGraph?.viewMode,
    syncGraph?.isLiveMode,
    syncGraph?.xRange?.min,
    syncGraph?.xRange?.max,
    selectedTimeWindow,
    viewMode,
    isLiveMode,
    historicalMode.switchToHistoricalMode,
    historicalMode.switchToLiveMode,
  ]);

  // Create and initialize the uPlot chart when data becomes available
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

    chartCreatedRef.current = true;

    if (isLiveMode) {
      lastProcessedCountRef.current = timestamps.length;
    }

    return () => {
      if (cleanup) {
        cleanup();
      }
      if (uplotRef.current) {
        uplotRef.current.destroy();
        uplotRef.current = null;
      }
      stopAnimations(animationRefs);
      chartCreatedRef.current = false;
    };
  }, [newData?.long, containerRef.current]);

  // Update chart data when new historical data arrives (live mode only)
  useEffect(() => {
    if (!isLiveMode || viewMode === "manual") return;

    liveMode.processNewHistoricalData();
  }, [
    newData?.long?.validCount,
    newData?.long?.lastTimestamp,
    viewMode,
    selectedTimeWindow,
    isLiveMode,
    liveMode.processNewHistoricalData,
  ]);

  // Update chart with real-time current data point (live mode only)
  useEffect(() => {
    if (
      !uplotRef.current ||
      !newData?.current ||
      !isLiveMode ||
      !chartCreatedRef.current ||
      animationRefs.animationState.current.isAnimating ||
      viewMode === "manual"
    )
      return;

    liveMode.updateLiveData();
  }, [
    newData?.current?.timestamp,
    viewMode,
    selectedTimeWindow,
    config.lines,
    isLiveMode,
    liveMode.updateLiveData,
  ]);
}
