/* eslint-disable react-compiler/react-compiler */
import {
  useEffect,
  MutableRefObject,
  RefObject,
  useRef,
  useState,
} from "react";
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
  uplotRefOut?: MutableRefObject<uPlot | null>;
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
  uplotRefOut,
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

  // Cleanup returned by createChart (detaches DOM event handlers). Stored in a
  // ref rather than returned from the creation effect below, since that effect's
  // dependencies tick on every data update — an effect-returned cleanup would
  // otherwise re-run (and tear down the chart) on every tick. Actual teardown
  // only happens on unmount (see the dedicated cleanup effect) or when data
  // disappears.
  const chartCleanupRef = useRef<(() => void) | null>(null);

  const destroyChart = () => {
    chartCleanupRef.current?.();
    chartCleanupRef.current = null;
    uplotRef.current?.destroy();
    uplotRef.current = null;
    if (uplotRefOut) uplotRefOut.current = null;
    stopAnimations(animationRefs);
    setIsChartCreated(false);
    chartCreatedRef.current = false;
  };

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
        historicalMode.switchToHistoricalMode(syncGraph.historicalSwitchOrigin);
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

  // Create the chart when data first becomes available.
  // With mutable TimeSeries (no immer), inserts mutate in place and never produce
  // a new `long` object reference, so we depend on validCount/lastTimestamp
  // (primitives that do change on every insert) instead. This effect deliberately
  // does NOT return a cleanup function: React would run that cleanup — and thus
  // destroy the chart — before every re-run, i.e. on every long-buffer tick,
  // which is exactly the periodic full rebuild that caused the graph to
  // pause/stutter. Once a chart exists, subsequent ticks are handled
  // incrementally by liveMode.processNewHistoricalData / updateLiveData below;
  // true teardown is handled by the dedicated unmount effect further down.
  useEffect(() => {
    if (!containerRef.current || !primarySeries?.newData?.long) {
      destroyChart();
      return;
    }

    if (chartCreatedRef.current && uplotRef.current) {
      return;
    }

    const [timestamps] = seriesToUPlotData(primarySeries.newData.long);
    if (timestamps.length === 0) {
      destroyChart();
      return;
    }

    chartCleanupRef.current =
      createChart({
        containerRef,
        uplotRef,
        uplotRefOut,
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
      }) ?? null;

    setIsChartCreated(true);
    chartCreatedRef.current = true;

    if (isLiveMode) {
      lastProcessedCountRef.current = timestamps.length;
    }
  }, [
    primarySeries?.newData?.long?.validCount,
    primarySeries?.newData?.long?.lastTimestamp,
    containerRef.current,
  ]);

  // Tear down the chart on unmount only. Split out from the creation effect
  // above so that the frequent validCount/lastTimestamp ticks there can't
  // trigger a destroy — see the comment on that effect for why.
  useEffect(() => {
    return () => {
      destroyChart();
    };
  }, []);

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
