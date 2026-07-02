/* eslint-disable react-compiler/react-compiler */
import {
  useEffect,
  MutableRefObject,
  RefObject,
  useRef,
  useState,
} from "react";
import type { IChartApi } from "lightweight-charts";
import { seriesToUPlotData } from "@/lib/timeseries";
import { BigGraphProps, SeriesData, SeriesRefs } from "./types";
import { GraphExportData } from "./excelExport";
import { getPrimarySeries } from "./dataHelpers";
import {
  createChart,
  setVisibleRangeSilently,
  setYAutoScale,
} from "./createChart";
import { LiveModeHandlers } from "./liveMode";
import { HistoricalModeHandlers } from "./historicalMode";

interface UseBigGraphEffectsProps {
  // Refs
  containerRef: RefObject<HTMLDivElement | null>;
  chartRef: RefObject<IChartApi | null>;
  chartRefOut?: MutableRefObject<IChartApi | null>;
  containerRefOut?: MutableRefObject<HTMLDivElement | null>;
  seriesRefs: RefObject<SeriesRefs>;
  startTimeRef: RefObject<number | null>;
  manualScaleRef: RefObject<{ x: { min: number; max: number } } | null>;
  suppressRangeEventRef: RefObject<boolean>;
  lastProcessedCountRef: RefObject<number>;
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
  handleTimeWindowChangeInternal: (
    newTimeWindow: number | "all",
    isSync?: boolean,
  ) => void;
}

export function useBigGraphEffects({
  containerRef,
  chartRef,
  chartRefOut,
  containerRefOut,
  seriesRefs,
  startTimeRef,
  manualScaleRef,
  suppressRangeEventRef,
  lastProcessedCountRef,
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

    if (xRangeChanged && chartRef.current) {
      setVisibleRangeSilently(chartRef.current, suppressRangeEventRef, {
        min: syncGraph.xRange?.min ?? 0,
        max: syncGraph.xRange?.max ?? 0,
      });
      manualScaleRef.current = {
        x: syncGraph.xRange ?? { min: 0, max: 1 },
      };
      hasChanges = true;
    }

    if (hasChanges) {
      if (syncGraph.viewMode === "manual") {
        setViewMode("manual");
        setIsLiveMode(false);
        lastProcessedCountRef.current = 0;
        if (chartRef.current) {
          setYAutoScale(chartRef.current, false);
        }
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

    createChart({
      containerRef,
      chartRef,
      chartRefOut,
      seriesRefs,
      newData,
      config,
      colors,
      renderValue,
      viewMode,
      selectedTimeWindow,
      isLiveMode,
      startTimeRef,
      manualScaleRef,
      suppressRangeEventRef,
      graphId,
      syncGraph,
      getHistoricalEndTimestamp: historicalMode.getHistoricalEndTimestamp,
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
      chartRef.current?.remove();
      chartRef.current = null;
      if (chartRefOut) chartRefOut.current = null;
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
      !chartRef.current ||
      !primarySeries?.newData?.current ||
      !isLiveMode ||
      !isChartCreated ||
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

  // Keep the Y auto-scale freeze state consistent with the current mode
  // whenever series visibility (or chart creation) changes.
  useEffect(() => {
    if (!chartRef.current || !isChartCreated) return;
    setYAutoScale(chartRef.current, isLiveMode && viewMode !== "manual");
  }, [visibleSeries, isChartCreated, isLiveMode, viewMode]);

  // Expose the chart's mount element to the caller (e.g. marker overlay
  // positioning), mirroring how chartRefOut is populated above.
  useEffect(() => {
    if (containerRefOut) {
      containerRefOut.current = containerRef.current;
    }
  }, [containerRefOut, isChartCreated]);
}
