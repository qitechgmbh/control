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

  const historicalFreezeTimestampRef = useRef<number | null>(null);
  const graphDataRef = useRef<Map<string, () => GraphExportData | null>>(
    new Map(),
  );

  // IMPROVED: Better synchronization tracking
  const syncStateRef = useRef({
    lastChangeSource: null as string | null,
    isProcessingChange: false,
    pendingChanges: new Set<string>(),
  });

  // IMPROVED: Debounced cleanup for better performance
  const cleanupTimeoutRef = useRef<NodeJS.Timeout | null>(null);

  const clearChangeSource = useCallback(() => {
    if (cleanupTimeoutRef.current) {
      clearTimeout(cleanupTimeoutRef.current);
    }
    cleanupTimeoutRef.current = setTimeout(() => {
      syncStateRef.current.lastChangeSource = null;
      syncStateRef.current.isProcessingChange = false;
      syncStateRef.current.pendingChanges.clear();
    }, 150); // Slightly longer timeout for complex operations
  }, []);

  // IMPROVED: Atomic state updates to prevent race conditions
  const updateSyncState = useCallback(
    (
      graphId: string,
      updates: {
        timeWindow?: number | "all";
        viewMode?: "default" | "all" | "manual";
        isLiveMode?: boolean;
        xRange?: { min: number; max: number } | undefined;
        clearHistoricalFreeze?: boolean;
        setHistoricalFreeze?: boolean;
      },
    ) => {
      // Prevent circular updates
      if (syncStateRef.current.lastChangeSource === graphId) return;
      if (syncStateRef.current.isProcessingChange) {
        syncStateRef.current.pendingChanges.add(graphId);
        return;
      }

      syncStateRef.current.isProcessingChange = true;
      syncStateRef.current.lastChangeSource = graphId;

      // Batch all state updates together
      if (updates.timeWindow !== undefined) setTimeWindow(updates.timeWindow);
      if (updates.viewMode !== undefined) setViewMode(updates.viewMode);
      if (updates.isLiveMode !== undefined) setIsLiveMode(updates.isLiveMode);
      if (updates.xRange !== undefined) setXRange(updates.xRange);

      // Handle historical freeze timestamp
      if (updates.clearHistoricalFreeze) {
        historicalFreezeTimestampRef.current = null;
      } else if (updates.setHistoricalFreeze) {
        historicalFreezeTimestampRef.current = Date.now();
      }

      clearChangeSource();
    },
    [clearChangeSource],
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

  const handleExport = useCallback(() => {
    if (graphDataRef.current.size === 0) {
      console.warn("No graphs registered for export");
      return;
    }
    exportGraphsToExcel(graphDataRef.current, exportGroupId || "synced-graphs");
  }, [exportGroupId]);

  // IMPROVED: More robust handlers
  const handleTimeWindowChange = useCallback(
    (graphId: string, newTimeWindow: number | "all") => {
      updateSyncState(graphId, {
        timeWindow: newTimeWindow,
        viewMode: newTimeWindow === "all" ? "all" : "default",
        isLiveMode: newTimeWindow === "all" ? true : isLiveMode,
        xRange: undefined,
        clearHistoricalFreeze: newTimeWindow === "all",
      });
    },
    [updateSyncState, isLiveMode],
  );

  const handleViewModeChange = useCallback(
    (
      graphId: string,
      newViewMode: "default" | "all" | "manual",
      newIsLiveMode: boolean,
    ) => {
      updateSyncState(graphId, {
        viewMode: newViewMode,
        isLiveMode: newIsLiveMode,
        clearHistoricalFreeze: newIsLiveMode,
      });
    },
    [updateSyncState],
  );

  // IMPROVED: Better zoom handling for drag operations
  const handleZoomChange = useCallback(
    (graphId: string, newXRange: { min: number; max: number }) => {
      updateSyncState(graphId, {
        xRange: newXRange,
        viewMode: "manual",
        isLiveMode: false,
        setHistoricalFreeze: historicalFreezeTimestampRef.current === null,
      });
    },
    [updateSyncState],
  );

  // IMPROVED: Throttled zoom updates for better performance during dragging
  const throttledZoomRef = useRef<NodeJS.Timeout | null>(null);
  const handleZoomChangeThrottled = useCallback(
    (graphId: string, newXRange: { min: number; max: number }) => {
      if (throttledZoomRef.current) {
        clearTimeout(throttledZoomRef.current);
      }

      throttledZoomRef.current = setTimeout(() => {
        handleZoomChange(graphId, newXRange);
      }, 50); // Throttle zoom updates to every 50ms during rapid changes
    },
    [handleZoomChange],
  );

  const handleSwitchToLive = useCallback(() => {
    updateSyncState("control", {
      isLiveMode: true,
      viewMode: timeWindow === "all" ? "all" : "default",
      xRange: undefined,
      clearHistoricalFreeze: true,
    });
  }, [updateSyncState, timeWindow]);

  const handleSwitchToHistorical = useCallback(() => {
    updateSyncState("control", {
      isLiveMode: false,
      viewMode: "manual",
      setHistoricalFreeze: true,
    });
  }, [updateSyncState]);

  const handleControlTimeWindowChange = useCallback(
    (newTimeWindow: number | "all") => {
      updateSyncState("control", {
        timeWindow: newTimeWindow,
        viewMode: newTimeWindow === "all" ? "all" : "default",
        isLiveMode: newTimeWindow === "all" ? true : isLiveMode,
        xRange: undefined,
        clearHistoricalFreeze: newTimeWindow === "all",
      });
    },
    [updateSyncState, isLiveMode],
  );

  const syncGraph: PropGraphSync = {
    timeWindow,
    viewMode,
    isLiveMode,
    xRange,
    historicalFreezeTimestamp: historicalFreezeTimestampRef.current,
    onTimeWindowChange: handleTimeWindowChange,
    onViewModeChange: handleViewModeChange,
    onZoomChange: handleZoomChangeThrottled, // Use throttled version
  };

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
    timeWindow,
    viewMode,
    isLiveMode,
    xRange,
    historicalFreezeTimestamp: historicalFreezeTimestampRef.current,
  };
}
