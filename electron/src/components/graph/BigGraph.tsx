import React, {
  useRef,
  useState,
  useCallback,
  useEffect,
  useMemo,
} from "react";
import type { Unit } from "@/control/units";
import { Icon } from "@/components/icons/Icon";
import { HISTORY_RETENTION_MS } from "@/lib/history";
import type { DataSeries, GraphConfig, GraphLine, SeriesData } from "./types";
import type { GraphSync } from "./useGraphSync";
import { DEFAULT_COLORS } from "./constants";
import { BigGraphHeader } from "./BigGraphHeader";
import { useGraphHistoryMachine } from "./history/GraphHistoryContext";
import { useMarkers } from "./markers/useMarkers";
import { GraphEngine } from "./engine/GraphEngine";
import type { EngineInputs, EngineTarget } from "./engine/types";
import type { HLine } from "./engine/plugins/thresholds";

export type BigGraphProps = {
  /** Shared view and export registry of the page (`useGraphSync`). */
  syncHook: GraphSync;
  newData: DataSeries;
  unit?: Unit;
  renderValue?: (value: number) => string;
  config: GraphConfig;
  graphId: string;
};

const DEFAULT_SERIES_COLORS = [
  "#3b82f6",
  "#ef4444",
  "#10b981",
  "#f59e0b",
  "#8b5cf6",
];

function sameValues(
  a: (number | null)[] | null,
  b: (number | null)[] | null,
): boolean {
  if (a === b) return true;
  if (!a || !b || a.length !== b.length) return false;
  return a.every((v, i) => v === b[i]);
}

type TargetWithOrigin = {
  line: GraphLine & { type: "target" };
  seriesIndex: number;
};

/** Target lines of all series (in order), each with the series it belongs to. */
function collectTargets(series: SeriesData[]): TargetWithOrigin[] {
  const out: TargetWithOrigin[] = [];
  series.forEach((s, seriesIndex) => {
    for (const line of s.lines ?? []) {
      if (line.type === "target") out.push({ line, seriesIndex });
    }
  });
  return out;
}

/**
 * A full-width time-series graph: database history plus live data, the page's markers,
 * and the shared view of `syncHook`. Drawing is done by `GraphEngine` (Chart.js); this
 * component feeds it and renders the header.
 */
export function BigGraph({
  syncHook,
  newData,
  unit,
  renderValue,
  config,
  graphId,
}: BigGraphProps) {
  const canvasRef = useRef<HTMLCanvasElement | null>(null);
  const engineRef = useRef<GraphEngine | null>(null);
  const machine = useGraphHistoryMachine();
  const { markers } = useMarkers();
  const { view, setRange, registerExport, unregisterExport } = syncHook;

  const series = useMemo(
    () => (Array.isArray(newData) ? newData : [newData]),
    [newData],
  );
  const [visibleSeries, setVisibleSeries] = useState<boolean[]>(() =>
    new Array(series.length).fill(true),
  );
  const [cursorValues, setCursorValues] = useState<(number | null)[] | null>(
    null,
  );
  const [status, setStatus] = useState<{
    loading: boolean;
    error: string | null;
  }>({ loading: false, error: null });

  if (visibleSeries.length !== series.length) {
    setVisibleSeries(new Array(series.length).fill(true));
  }

  const colors = useMemo(
    () => ({
      grid: config.colors?.grid ?? DEFAULT_COLORS.grid,
      axis: config.colors?.axis ?? DEFAULT_COLORS.axis,
      background: config.colors?.background ?? DEFAULT_COLORS.background,
    }),
    [config.colors],
  );
  const seriesColors = useMemo(
    () =>
      series.map(
        (s, index) =>
          s.color ||
          (series.length === 1 ? config.colors?.primary : undefined) ||
          DEFAULT_SERIES_COLORS[index % DEFAULT_SERIES_COLORS.length],
      ),
    [series, config.colors],
  );

  // Latest props for callbacks that outlive a render (export, engine callbacks).
  const latestRef = useRef({ series, config, unit, renderValue, setRange });
  useEffect(() => {
    latestRef.current = { series, config, unit, renderValue, setRange };
  });

  // --- engine lifecycle (re-created only if the machine or cursor group changes) ---
  const cursorGroup = view.cursorGroup;
  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    const engine = new GraphEngine(canvas, machine, cursorGroup, {
      onCursor: (values) =>
        setCursorValues((prev) => (sameValues(prev, values) ? prev : values)),
      onUserRange: (range) => latestRef.current.setRange(range),
      onStatus: (next) =>
        setStatus((prev) =>
          prev.loading === next.loading && prev.error === next.error
            ? prev
            : next,
        ),
    });
    engineRef.current = engine;
    return () => {
      engine.dispose();
      engineRef.current = null;
    };
  }, [machine, cursorGroup]);

  // --- push inputs to the engine when they change ---
  // Not on every render: a cursor move re-renders this component (header values), and
  // new inputs make the engine rebuild every dataset.
  const targets = useMemo(() => collectTargets(series), [series]);

  useEffect(() => {
    const engine = engineRef.current;
    if (!engine) return;

    const timeWindow =
      view.timeWindow ?? config.defaultTimeWindow ?? 30 * 60 * 1000;
    const hLines: HLine[] = [];
    const addThreshold = (line: GraphLine, color: string, shown: boolean) => {
      if (line.type !== "threshold" || line.show === false || !shown) return;
      hLines.push({
        value: line.value,
        color: line.color || color,
        dash: line.dash,
        width: line.width,
      });
    };
    for (const line of config.lines ?? [])
      addThreshold(line, DEFAULT_COLORS.primary, true);
    series.forEach((s, i) => {
      for (const line of s.lines ?? [])
        addThreshold(line, seriesColors[i], visibleSeries[i] ?? true);
    });

    const engineTargets: EngineTarget[] = targets.map(
      ({ line, seriesIndex }) => ({
        color: line.color || seriesColors[seriesIndex],
        value: line.value,
        history: line.history,
        targetSeries: line.targetSeries,
        dash: line.dash,
        width: line.width,
        visible: line.show !== false && (visibleSeries[seriesIndex] ?? true),
      }),
    );

    const inputs: EngineInputs = {
      series: series.map((s, i) => ({
        newData: s.newData,
        history: s.history,
        historyScale: s.historyScale,
        color: seriesColors[i],
        visible: visibleSeries[i] ?? true,
      })),
      targets: engineTargets,
      hLines,
      markers,
      view: {
        live: view.isLiveMode,
        windowMs: timeWindow === "all" ? HISTORY_RETENTION_MS : timeWindow,
        all: timeWindow === "all",
        range: view.isLiveMode ? undefined : view.xRange,
        showFrom: view.showFromTimestamp,
        freeze: view.historicalFreezeTimestamp,
        revision: view.revision,
      },
      renderValue,
      colors,
    };
    engine.setInputs(inputs);
  }, [
    series,
    seriesColors,
    colors,
    targets,
    config,
    markers,
    view,
    visibleSeries,
    renderValue,
  ]);

  // --- export: what this graph contributes (data is loaded from the database on export) ---
  useEffect(() => {
    registerExport(graphId, () => {
      const current = latestRef.current;
      return {
        machine,
        config: current.config,
        series: current.series,
        unit: current.unit,
        renderValue: current.renderValue,
        visibleRange: () => engineRef.current?.visibleRange() ?? null,
      };
    });
    return () => unregisterExport(graphId);
  }, [registerExport, unregisterExport, graphId, machine]);

  const toggleSeries = useCallback((index: number) => {
    setVisibleSeries((prev) => {
      const visibleCount = prev.filter(Boolean).length;
      if (prev[index] && visibleCount === 1) return prev;
      const next = [...prev];
      next[index] = !next[index];
      return next;
    });
  }, []);

  // Under the cursor its value (null over a gap), otherwise the live value.
  const displayValues = series.map((s, index) =>
    cursorValues
      ? (cursorValues[index] ?? null)
      : (s.newData?.current?.value ?? null),
  );

  return (
    <div className="h-[50vh] w-full">
      <div className="flex h-full w-full flex-col overflow-hidden rounded-3xl border border-gray-200 bg-white shadow">
        <BigGraphHeader
          title={config.title}
          unit={unit}
          series={series}
          colors={seriesColors}
          visible={visibleSeries}
          values={displayValues}
          renderValue={renderValue}
          onToggleSeries={toggleSeries}
        />

        <div className="-mt-2 px-6">
          <div className="h-px bg-gray-200"></div>
        </div>

        <div className="flex-1 overflow-hidden rounded-b-3xl pt-4 pr-2 pb-2 pl-2">
          <div
            className="relative h-full w-full touch-pan-y overflow-hidden select-none"
            style={{ backgroundColor: colors.background }}
          >
            <canvas ref={canvasRef} />
            {(status.loading || status.error) && (
              <div className="pointer-events-none absolute top-1 right-3 flex items-center gap-1 rounded bg-white/80 px-2 py-0.5 text-xs text-gray-500">
                {status.loading ? (
                  <>
                    <Icon name="lu:Loader" className="size-3 animate-spin" />
                    Loading history…
                  </>
                ) : (
                  <span title={status.error ?? undefined}>
                    History unavailable — showing live data
                  </span>
                )}
              </div>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}
