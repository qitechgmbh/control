import { useState, useCallback, useRef } from "react";
import { PropGraphSync } from "./types";
import { GraphExportData, exportGraphsToExcel } from "./excelExport";

export function useGraphSync(
  defaultTimeWindow: number | "all" = 30 * 60 * 1000,
  exportGroupId?: string,
) {
  const [timeWindow, setTimeWindow] = useState<number | "all">(
    defaultTimeWindow,
  );
  const [viewMode, setViewMode] = useState<"default" | "all" | "manual">(
    "default",
  );
  const [isLiveMode, setIsLiveMode] = useState(true);
  const [xRange, setXRange] = useState<
    { min: number; max: number } | undefined
  >();

  // Store graph data for export using your GraphExportData type
  const graphDataRef = useRef<Map<string, () => GraphExportData | null>>(
    new Map(),
  );

  // Register graph for export
  const registerGraphForExport = useCallback(
    (graphId: string, getDataFn: () => GraphExportData | null) => {
      graphDataRef.current.set(graphId, getDataFn);
    },
    [],
  );

  // Unregister graph from export
  const unregisterGraphFromExport = useCallback((graphId: string) => {
    graphDataRef.current.delete(graphId);
  }, []);

  // Export all graphs using your existing function
  const handleExport = useCallback(() => {
    if (graphDataRef.current.size === 0) {
      console.warn("No graphs registered for export");
      return;
    }

    exportGraphsToExcel(graphDataRef.current, exportGroupId || "synced-graphs");
  }, [exportGroupId]);

  // Sync callbacks
  const handleTimeWindowChange = useCallback(
    (graphId: string, newTimeWindow: number | "all") => {
      setTimeWindow(newTimeWindow);
      setViewMode(newTimeWindow === "all" ? "all" : "default");
      setXRange(undefined);
    },
    [],
  );

  const handleViewModeChange = useCallback(
    (
      graphId: string,
      newViewMode: "default" | "all" | "manual",
      newIsLiveMode: boolean,
    ) => {
      setViewMode(newViewMode);
      setIsLiveMode(newIsLiveMode);
    },
    [],
  );

  const handleZoomChange = useCallback(
    (graphId: string, newXRange: { min: number; max: number }) => {
      setXRange(newXRange);
      setViewMode("manual");
      setIsLiveMode(false);
    },
    [],
  );

  // Control handlers
  const handleSwitchToLive = useCallback(() => {
    setIsLiveMode(true);
    setViewMode(timeWindow === "all" ? "all" : "default");
    setXRange(undefined);
  }, [timeWindow]);

  const handleSwitchToHistorical = useCallback(() => {
    setIsLiveMode(false);
    setViewMode("manual");
  }, []);

  const handleControlTimeWindowChange = useCallback(
    (newTimeWindow: number | "all") => {
      setTimeWindow(newTimeWindow);
      setViewMode(newTimeWindow === "all" ? "all" : "default");
      setXRange(undefined);
    },
    [],
  );

  // Create sync object for graphs
  const syncGraph: PropGraphSync = {
    timeWindow,
    viewMode,
    isLiveMode,
    xRange,
    onTimeWindowChange: handleTimeWindowChange,
    onViewModeChange: handleViewModeChange,
    onZoomChange: handleZoomChange,
  };

  // Control props object with export
  const controlProps = {
    timeWindow,
    isLiveMode,
    onTimeWindowChange: handleControlTimeWindowChange,
    onSwitchToLive: handleSwitchToLive,
    onSwitchToHistorical: handleSwitchToHistorical,
    onExport: handleExport,
  };

  return {
    syncGraph,
    controlProps,
    registerGraphForExport,
    unregisterGraphFromExport,
    handleExport,
    // Individual state values if needed
    timeWindow,
    viewMode,
    isLiveMode,
    xRange,
  };
}
