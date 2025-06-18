import { useEffect, RefObject, useRef, useState } from "react";
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
  containerRef,
  uplotRef,
  startTimeRef,
  manualScaleRef,
  lastProcessedCountRef,
  animationRefs,
  handlerRefs,

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

  const localmanualScale = useRef(manualScaleRef.current);
  const localProcessedCount = useRef(lastProcessedCountRef.current);
  const localuplotRef = useRef<uPlot | null>(uplotRef.current);

  // Register/unregister this graph for Excel export functionality
  useEffect(() => {
    if (onRegisterForExport) {
      const getExportData = (): GraphExportData | null => {
        if (!newData) return null;

        return {
          config,
          data: newData,
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
        historicalMode.switchToHistoricalMode();
      } else if (!isLiveMode && newIsLiveMode) {
        historicalMode.switchToLiveMode();
      }

      setIsLiveMode(newIsLiveMode);
      hasChanges = true;
    }

    if (syncGraph.xRange && uplotRef.current) {
      uplotRef.current.batch(() => {
        uplotRef.current!.setScale("x", {
          min: syncGraph.xRange?.min ?? 0,
          max: syncGraph.xRange?.max ?? 0,
        });

        if (newData?.long) {
          const [timestamps, values] = seriesToUPlotData(newData.long);
          updateYAxisScale(
            timestamps,
            values,
            syncGraph.xRange?.min ?? 0,
            syncGraph.xRange?.max ?? 0,
          );
        }
      });
      localmanualScale.current = {
        x: syncGraph.xRange,
        y: manualScaleRef.current?.y ?? { min: 0, max: 1 },
      };
      hasChanges = true;
    }

    if (hasChanges && syncGraph.viewMode === "manual") {
      setViewMode("manual");
      setIsLiveMode(false);
      stopAnimations(animationRefs);
      localProcessedCount.current = 0;
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
      setIsChartCreated(false);
      return;
    }

    const [timestamps] = seriesToUPlotData(newData.long);
    if (timestamps.length === 0) {
      setIsChartCreated(false);
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

    if (isLiveMode) {
      localProcessedCount.current = timestamps.length;
    }

    return () => {
      if (cleanup) cleanup();
      if (uplotRef.current) {
        uplotRef.current.destroy();
        localuplotRef.current = null;
      }
      stopAnimations(animationRefs);
      setIsChartCreated(false);
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
      !isChartCreated ||
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
    isChartCreated,
    liveMode.updateLiveData,
  ]);
}
