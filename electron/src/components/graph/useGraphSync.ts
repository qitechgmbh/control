import { useState, useCallback, useRef } from "react";
import { PropGraphSync } from "./types";
import { GraphExportData, exportGraphsToExcel } from "./excelExport";
import { useGraphSettingsStore } from "@/stores/graphSettingsStore";

export function useGraphSync(exportGroupId?: string) {
  const store = useGraphSettingsStore();
  // not reactive but works :)
  const timeWindow = store.getTimeframe();

  const [viewMode, setViewMode] = useState<"default" | "all" | "manual">(
    "default",
  );
  const [isLiveMode, setIsLiveMode] = useState(true);
  const [xRange, setXRange] = useState<
    { min: number; max: number } | undefined
  >();
  const [showFromTimestamp, setShowFromTimestamp] = useState<number | null>(
    null,
  );

  const historicalFreezeTimestampRef = useRef<number | null>(null);
  const graphDataRef = useRef<Map<string, () => GraphExportData | null>>(
    new Map(),
  );

  const syncStateRef = useRef({
    lastChangeSource: null as string | null,
    isProcessingChange: false,
    pendingChanges: new Set<string>(),
    currentRequestId: 0,
  });

  // IMPROVED: Separate timeouts for different operations
  const timeoutRefs = useRef({
    cleanup: null as NodeJS.Timeout | null,
    throttledZoom: null as NodeJS.Timeout | null,
  });

  // IMPROVED: Throttling state using ref instead of function property
  const throttleStateRef = useRef({
    lastCall: 0,
  });

  const clearChangeSource = useCallback(() => {
    if (timeoutRefs.current.cleanup) {
      clearTimeout(timeoutRefs.current.cleanup);
    }
    timeoutRefs.current.cleanup = setTimeout(() => {
      syncStateRef.current.lastChangeSource = null;
      syncStateRef.current.isProcessingChange = false;
      syncStateRef.current.pendingChanges.clear();
    }, 100);
  }, []);

  // IMPROVED: Atomic state updates with request ID tracking
  const updateSyncState = useCallback(
    (
      graphId: string,
      updates: {
        timeWindow?: number | "all";
        viewMode?: "default" | "all" | "manual";
        isLiveMode?: boolean;
        xRange?: { min: number; max: number } | undefined;
        showFromTimestamp?: number | null;
        clearHistoricalFreeze?: boolean;
        setHistoricalFreeze?: boolean;
      },
      requestId?: number,
    ) => {
      // Generate request ID if not provided
      const currentRequestId =
        requestId ?? ++syncStateRef.current.currentRequestId;

      // Prevent circular updates and stale requests
      if (syncStateRef.current.lastChangeSource === graphId) return;
      if (syncStateRef.current.isProcessingChange && !requestId) {
        syncStateRef.current.pendingChanges.add(graphId);
        return;
      }

      syncStateRef.current.isProcessingChange = true;
      syncStateRef.current.lastChangeSource = graphId;
      // Use requestAnimationFrame to ensure state updates happen in next frame
      requestAnimationFrame(() => {
        // Check if this request is still valid
        if (currentRequestId < syncStateRef.current.currentRequestId - 1) {
          return; // Skip stale request
        }

        // Batch all state updates together
        if (updates.timeWindow !== undefined)
          store.setTimeframe(updates.timeWindow);
        if (updates.viewMode !== undefined) setViewMode(updates.viewMode);
        if (updates.isLiveMode !== undefined) setIsLiveMode(updates.isLiveMode);
        if (updates.xRange !== undefined) setXRange(updates.xRange);
        if (updates.showFromTimestamp !== undefined)
          setShowFromTimestamp(updates.showFromTimestamp);

        // Handle historical freeze timestamp
        if (updates.clearHistoricalFreeze) {
          historicalFreezeTimestampRef.current = null;
        } else if (updates.setHistoricalFreeze) {
          historicalFreezeTimestampRef.current = Date.now();
        }

        clearChangeSource();
      });
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

  // IMPROVED: Better throttling with immediate first call using ref
  const handleZoomChangeThrottled = useCallback(
    (graphId: string, newXRange: { min: number; max: number }) => {
      // Clear existing throttled call
      if (timeoutRefs.current.throttledZoom) {
        clearTimeout(timeoutRefs.current.throttledZoom);
      }

      // For the first call or if enough time has passed, execute immediately
      const now = Date.now();
      const lastCall = throttleStateRef.current.lastCall;

      if (now - lastCall > 100) {
        handleZoomChange(graphId, newXRange);
        throttleStateRef.current.lastCall = now;
      } else {
        // Throttle subsequent calls
        timeoutRefs.current.throttledZoom = setTimeout(() => {
          handleZoomChange(graphId, newXRange);
          throttleStateRef.current.lastCall = Date.now();
        }, 50);
      }
    },
    [handleZoomChange],
  );

  const handleSwitchToLive = useCallback(() => {
    updateSyncState("control", {
      isLiveMode: true,
      viewMode: timeWindow === "all" ? "all" : "default",
      xRange: undefined,
      clearHistoricalFreeze: true,
      showFromTimestamp: null, // Clear the show from timestamp when switching to live
    });
  }, [updateSyncState, timeWindow]);

  const handleSwitchToHistorical = useCallback(() => {
    updateSyncState("control", {
      isLiveMode: false,
      viewMode: "manual",
      setHistoricalFreeze: true,
    });
  }, [updateSyncState]);

  const handleShowFromChange = useCallback(
    (timestamp: number | null) => {
      updateSyncState("control", {
        showFromTimestamp: timestamp,
      });
    },
    [updateSyncState],
  );

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
    showFromTimestamp,
    onTimeWindowChange: handleTimeWindowChange,
    onViewModeChange: handleViewModeChange,
    onZoomChange: handleZoomChangeThrottled,
  };

  const controlProps = {
    timeWindow,
    isLiveMode,
    onTimeWindowChange: handleControlTimeWindowChange,
    onSwitchToLive: handleSwitchToLive,
    onSwitchToHistorical: handleSwitchToHistorical,
    onExport: handleExport,
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
    historicalFreezeTimestamp: historicalFreezeTimestampRef.current,
    showFromTimestamp,
  };
}
