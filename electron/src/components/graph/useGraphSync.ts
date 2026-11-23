import { useState, useCallback, useRef, useId } from "react";
import type { GraphView, TimeWindow, ViewRange } from "./types";
import { type GraphExportData, exportGraphsToExcel } from "./export";
import { useLogsStore } from "@/stores/logsStore";
import { useGraphSettingsStore } from "@/stores/graphSettingsStore";
import { useExportDialog } from "@/hooks/useExportDialog";

export type GraphSync = ReturnType<typeof useGraphSync>;

/**
 * Shared state of all graphs on a page: the view (time window, live/historical, the
 * pan/zoom range, "show from") and the export registry. Every graph renders from
 * `view`, so changing it from one graph or the control panel moves all of them.
 */
export function useGraphSync(exportGroupId?: string) {
  const timeWindow = useGraphSettingsStore((s) => s.timeframe);
  const setTimeframe = useGraphSettingsStore((s) => s.setTimeframe);
  const cursorGroup = useId();

  const [isLiveMode, setIsLiveMode] = useState(true);
  const [xRange, setXRange] = useState<ViewRange | undefined>();
  const [showFromTimestamp, setShowFromTimestamp] = useState<number | null>(
    null,
  );
  const [historicalFreezeTimestamp, setHistoricalFreezeTimestamp] = useState<
    number | null
  >(null);

  // --- view ---

  const setTimeWindow = useCallback(
    (next: TimeWindow) => {
      setTimeframe(next);
      setXRange(undefined);
      setShowFromTimestamp(null);
      if (next === "all") {
        setIsLiveMode(true);
        setHistoricalFreezeTimestamp(null);
      }
    },
    [setTimeframe],
  );

  /** A graph was panned/zoomed: show that range on all graphs (historical mode). */
  const setRange = useCallback((range: ViewRange) => {
    setXRange(range);
    setIsLiveMode(false);
    setHistoricalFreezeTimestamp((ts) => ts ?? Date.now());
  }, []);

  const switchToLive = useCallback(() => {
    setIsLiveMode(true);
    setXRange(undefined);
    setHistoricalFreezeTimestamp(null);
    setShowFromTimestamp(null);
  }, []);

  const switchToHistorical = useCallback(() => {
    setIsLiveMode(false);
    setHistoricalFreezeTimestamp(Date.now());
  }, []);

  const setShowFrom = useCallback((timestamp: number | null) => {
    setShowFromTimestamp(timestamp);
    setXRange(undefined);
  }, []);

  // --- export ---

  const exportSources = useRef(new Map<string, () => GraphExportData | null>());

  const registerExport = useCallback(
    (id: string, getData: () => GraphExportData | null) => {
      exportSources.current.set(id, getData);
    },
    [],
  );

  const unregisterExport = useCallback((id: string) => {
    exportSources.current.delete(id);
  }, []);

  const { notifyResult, dialogProps: exportDialogProps } = useExportDialog();

  const exportToExcel = useCallback(async () => {
    if (exportSources.current.size === 0) {
      console.warn("No graphs registered for export");
      return;
    }
    try {
      const logs = useLogsStore.getState().entries;
      const result = await exportGraphsToExcel(
        exportSources.current,
        exportGroupId || "synced-graphs",
        logs,
      );
      if (result) notifyResult(result);
    } catch (error) {
      console.error("Failed to export graphs:", error);
    }
  }, [exportGroupId, notifyResult]);

  const view: GraphView = {
    timeWindow,
    isLiveMode,
    xRange,
    historicalFreezeTimestamp,
    showFromTimestamp,
    cursorGroup,
  };

  return {
    view,
    setTimeWindow,
    setRange,
    switchToLive,
    switchToHistorical,
    setShowFrom,
    registerExport,
    unregisterExport,
    exportToExcel,
    exportDialogProps,
  };
}

/**
 * Right edge of the current view, or null in live mode (where "now" moves on).
 * Used as the default time of a new marker.
 */
export function viewEndTimestamp(view: GraphView): number | null {
  if (view.isLiveMode) return null;
  return view.xRange?.max ?? view.historicalFreezeTimestamp;
}
