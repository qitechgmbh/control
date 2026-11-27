import { useState, useCallback, useRef, useId } from "react";
import type { GraphView, TimeWindow, ViewRange } from "./types";
import {
  type ExportOptions,
  type GraphExportSource,
  currentExportRange,
  runExport as runExportFiles,
} from "./export";
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

  const exportSources = useRef(
    new Map<string, () => GraphExportSource | null>(),
  );

  const registerExport = useCallback(
    (id: string, getSource: () => GraphExportSource | null) => {
      exportSources.current.set(id, getSource);
    },
    [],
  );

  const unregisterExport = useCallback((id: string) => {
    exportSources.current.delete(id);
  }, []);

  const collectSources = useCallback(
    () =>
      [...exportSources.current.values()]
        .map((getSource) => getSource())
        .filter((s): s is GraphExportSource => s !== null),
    [],
  );

  /** The range the graphs show right now: the export dialog's default. */
  const exportRange = useCallback(
    () => currentExportRange(collectSources()),
    [collectSources],
  );

  const { notifyResult, dialogProps: exportDialogProps } = useExportDialog();

  const runExport = useCallback(
    async (options: Omit<ExportOptions, "groupId">) => {
      const result = await runExportFiles(collectSources(), {
        ...options,
        groupId: exportGroupId || "graphs",
      });
      notifyResult(result);
      return result;
    },
    [collectSources, exportGroupId, notifyResult],
  );

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
    exportRange,
    runExport,
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
