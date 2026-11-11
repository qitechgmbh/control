import { useState, useCallback, useRef, useId } from "react";
import { PropGraphSync, SwitchOrigin } from "./types";
import { GraphExportData, exportGraphsToExcel } from "./excelExport";
import { useLogsStore } from "@/stores/logsStore";
import { useGraphSettingsStore } from "@/stores/graphSettingsStore";
import { useExportDialog } from "@/hooks/useExportDialog";

type Range = { min: number; max: number };

/**
 * Shared view state for all graphs on a page: time window, live/historical mode, the
 * manual (pan/zoom) range and the "show from" timestamp. Every graph renders from this
 * state, so changing it in one graph moves all of them.
 */
export function useGraphSync(exportGroupId?: string) {
  const timeWindow = useGraphSettingsStore((s) => s.timeframe);
  const setTimeframe = useGraphSettingsStore((s) => s.setTimeframe);
  const cursorGroup = useId();

  const [viewMode, setViewMode] = useState<"default" | "all" | "manual">(
    timeWindow === "all" ? "all" : "default",
  );
  const [isLiveMode, setIsLiveMode] = useState(true);
  const [xRange, setXRange] = useState<Range | undefined>();
  const [showFromTimestamp, setShowFromTimestamp] = useState<number | null>(
    null,
  );
  const [historicalFreezeTimestamp, setHistoricalFreezeTimestamp] = useState<
    number | null
  >(null);
  const [historicalSwitchOrigin, setHistoricalSwitchOrigin] =
    useState<SwitchOrigin>("gesture");

  const graphDataRef = useRef<Map<string, () => GraphExportData | null>>(
    new Map(),
  );

  const registerGraphForExport = useCallback(
    (graphId: string, getDataFn: () => GraphExportData | null) => {
      graphDataRef.current.set(graphId, getDataFn);
    },
    [],
  );

  const unregisterGraphFromExport = useCallback((graphId: string) => {
    graphDataRef.current.delete(graphId);
  }, []);

  const { notifyResult, dialogProps: exportDialogProps } = useExportDialog();

  const handleExport = useCallback(() => {
    if (graphDataRef.current.size === 0) {
      console.warn("No graphs registered for export");
      return Promise.resolve();
    }
    const logs = useLogsStore.getState().entries;
    return exportGraphsToExcel(
      graphDataRef.current,
      exportGroupId || "synced-graphs",
      logs,
    )
      .then((result) => {
        if (result) notifyResult(result);
      })
      .catch((error) => {
        console.error("Failed to export graphs:", error);
      });
  }, [exportGroupId, notifyResult]);

  const changeTimeWindow = useCallback(
    (newTimeWindow: number | "all") => {
      setTimeframe(newTimeWindow);
      setXRange(undefined);
      setShowFromTimestamp(null);
      if (newTimeWindow === "all") {
        setViewMode("all");
        setIsLiveMode(true);
        setHistoricalFreezeTimestamp(null);
      } else {
        setViewMode(isLiveMode ? "default" : "manual");
      }
    },
    [setTimeframe, isLiveMode],
  );

  const handleTimeWindowChange = useCallback(
    (_graphId: string, newTimeWindow: number | "all") =>
      changeTimeWindow(newTimeWindow),
    [changeTimeWindow],
  );

  const handleViewModeChange = useCallback(
    (
      _graphId: string,
      newViewMode: "default" | "all" | "manual",
      newIsLiveMode: boolean,
    ) => {
      setViewMode(newViewMode);
      setIsLiveMode(newIsLiveMode);
      if (newIsLiveMode) {
        setHistoricalFreezeTimestamp(null);
        setXRange(undefined);
      }
    },
    [],
  );

  const handleZoomChange = useCallback((_graphId: string, newXRange: Range) => {
    setXRange(newXRange);
    setViewMode("manual");
    setIsLiveMode(false);
    setHistoricalFreezeTimestamp((ts) => ts ?? Date.now());
    setHistoricalSwitchOrigin("gesture");
  }, []);

  const handleSwitchToLive = useCallback(() => {
    setIsLiveMode(true);
    setViewMode(timeWindow === "all" ? "all" : "default");
    setXRange(undefined);
    setHistoricalFreezeTimestamp(null);
    setShowFromTimestamp(null);
  }, [timeWindow]);

  const handleSwitchToHistorical = useCallback(() => {
    setIsLiveMode(false);
    setViewMode("manual");
    setHistoricalFreezeTimestamp(Date.now());
    setHistoricalSwitchOrigin("button");
  }, []);

  const handleShowFromChange = useCallback((timestamp: number | null) => {
    setShowFromTimestamp(timestamp);
    setXRange(undefined);
  }, []);

  const syncGraph: PropGraphSync = {
    timeWindow,
    viewMode,
    isLiveMode,
    xRange,
    historicalFreezeTimestamp,
    showFromTimestamp,
    historicalSwitchOrigin,
    cursorGroup,
    onTimeWindowChange: handleTimeWindowChange,
    onViewModeChange: handleViewModeChange,
    onZoomChange: handleZoomChange,
  };

  const controlProps = {
    timeWindow,
    isLiveMode,
    onTimeWindowChange: changeTimeWindow,
    onSwitchToLive: handleSwitchToLive,
    onSwitchToHistorical: handleSwitchToHistorical,
    onExport: handleExport,
    exportDialogProps,
    showFromTimestamp,
    onShowFromChange: handleShowFromChange,
  };

  return {
    syncGraph,
    controlProps,
    registerGraphForExport,
    unregisterGraphFromExport,
    handleExport,
    timeWindow,
    viewMode,
    isLiveMode,
    xRange,
    historicalFreezeTimestamp,
    showFromTimestamp,
  };
}
