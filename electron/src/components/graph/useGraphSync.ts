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

  // Store graph data for export
  const graphDataRef = useRef<Map<string, () => GraphExportData | null>>(
    new Map(),
  );

  // Track which graph initiated the change to prevent loops
  const lastChangeSourceRef = useRef<string | null>(null);

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

  // Export all graphs
  const handleExport = useCallback(() => {
    if (graphDataRef.current.size === 0) {
      console.warn("No graphs registered for export");
      return;
    }

    exportGraphsToExcel(graphDataRef.current, exportGroupId || "synced-graphs");
  }, [exportGroupId]);

  // Sync callbacks - prevent infinite loops
  const handleTimeWindowChange = useCallback(
    (graphId: string, newTimeWindow: number | "all") => {
      if (lastChangeSourceRef.current === graphId) return;

      lastChangeSourceRef.current = graphId;
      setTimeWindow(newTimeWindow);
      setViewMode(newTimeWindow === "all" ? "all" : "default");
      setXRange(undefined);

      // Clear the source after a brief delay
      setTimeout(() => {
        lastChangeSourceRef.current = null;
      }, 100);
    },
    [],
  );

  const handleViewModeChange = useCallback(
    (
      graphId: string,
      newViewMode: "default" | "all" | "manual",
      newIsLiveMode: boolean,
    ) => {
      if (lastChangeSourceRef.current === graphId) return;

      lastChangeSourceRef.current = graphId;
      setViewMode(newViewMode);
      setIsLiveMode(newIsLiveMode);

      setTimeout(() => {
        lastChangeSourceRef.current = null;
      }, 100);
    },
    [],
  );

  const handleZoomChange = useCallback(
    (graphId: string, newXRange: { min: number; max: number }) => {
      if (lastChangeSourceRef.current === graphId) return;

      lastChangeSourceRef.current = graphId;
      setXRange(newXRange);
      setViewMode("manual");
      setIsLiveMode(false);

      setTimeout(() => {
        lastChangeSourceRef.current = null;
      }, 100);
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
      setIsLiveMode(true); // Reset to live mode when changing time window
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
