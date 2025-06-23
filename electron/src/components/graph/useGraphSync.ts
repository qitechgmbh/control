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

  // ADDED: Track historical freeze timestamp at sync level
  const historicalFreezeTimestampRef = useRef<number | null>(null);

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

  // FIXED: Sync callbacks with historical freeze timestamp logic
  const handleTimeWindowChange = useCallback(
    (graphId: string, newTimeWindow: number | "all") => {
      if (lastChangeSourceRef.current === graphId) return;

      lastChangeSourceRef.current = graphId;
      setTimeWindow(newTimeWindow);

      if (newTimeWindow === "all") {
        setViewMode("all");
        setIsLiveMode(true);
        // Clear historical freeze when switching to "all"
        historicalFreezeTimestampRef.current = null;
      } else {
        setViewMode("default");
        // Don't change isLiveMode - preserve current mode
        // If in historical mode, keep the frozen timestamp
      }

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

      // ADDED: Clear historical freeze when switching to live mode
      if (newIsLiveMode) {
        historicalFreezeTimestampRef.current = null;
      }

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

      // ADDED: Capture freeze timestamp when manually zooming (entering historical mode)
      if (historicalFreezeTimestampRef.current === null) {
        historicalFreezeTimestampRef.current = Date.now();
      }

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
    // ADDED: Clear historical freeze when switching to live
    historicalFreezeTimestampRef.current = null;
  }, [timeWindow]);

  const handleSwitchToHistorical = useCallback(() => {
    setIsLiveMode(false);
    setViewMode("manual");
    // ADDED: Capture freeze timestamp when switching to historical
    historicalFreezeTimestampRef.current = Date.now();
  }, []);

  const handleControlTimeWindowChange = useCallback(
    (newTimeWindow: number | "all") => {
      setTimeWindow(newTimeWindow);

      if (newTimeWindow === "all") {
        setViewMode("all");
        setIsLiveMode(true);
        // Clear historical freeze when switching to "all"
        historicalFreezeTimestampRef.current = null;
      } else {
        setViewMode("default");
        // Don't automatically switch to live mode for specific time windows
        // Preserve current mode (live or historical)
      }

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
    // ADDED: Pass historical freeze timestamp to graphs
    historicalFreezeTimestamp: historicalFreezeTimestampRef.current,
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
    // ADDED: Expose historical freeze timestamp
    historicalFreezeTimestamp: historicalFreezeTimestampRef.current,
  };
}
