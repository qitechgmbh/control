import React, {
  useRef,
  useState,
  useCallback,
  useEffect,
  useMemo,
} from "react";
import type { Chart } from "chart.js";
import { renderUnitSymbol, getUnitIcon } from "@/control/units";
import { Icon } from "@/components/Icon";
import { BigGraphProps, GraphLine, SeriesData } from "./types";
import { GraphExportData } from "./excelExport";
import { DEFAULT_COLORS } from "./constants";
import { TouchButton } from "../touch/TouchButton";
import { ControlCard } from "@/control/ControlCard";
import { timeSeriesFromPoints } from "@/lib/timeseries";
import { HISTORY_RETENTION_MS } from "@/lib/history";
import { useGraphHistoryMachine } from "./history/GraphHistoryContext";
import {
  GraphEngine,
  type EngineInputs,
  type EngineTarget,
} from "./chartjs/GraphEngine";
import type { HLine } from "./chartjs/plugins";
import type { Marker } from "@/stores/markerStore";

const DEFAULT_SERIES_COLORS = [
  "#3b82f6",
  "#ef4444",
  "#10b981",
  "#f59e0b",
  "#8b5cf6",
];
const EMPTY_MARKERS: Marker[] = [];

export function normalizeDataSeries(
  data: BigGraphProps["newData"],
): SeriesData[] {
  return Array.isArray(data) ? data : [data];
}

function formatDisplayValue(
  value: number | null | undefined,
  renderValue?: (value: number) => string,
) {
  if (value === null || value === undefined || !Number.isFinite(value)) {
    return "N/A";
  }
  return renderValue ? renderValue(value) : value.toFixed(3);
}

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

export function BigGraph({
  newData,
  unit,
  renderValue,
  config,
  graphId,
  syncGraph,
  onChart,
  markers,
  onRegisterForExport,
  onUnregisterFromExport,
}: BigGraphProps & {
  onRegisterForExport?: (
    graphId: string,
    getDataFn: () => GraphExportData | null,
  ) => void;
  onUnregisterFromExport?: (graphId: string) => void;
}) {
  const canvasRef = useRef<HTMLCanvasElement | null>(null);
  const engineRef = useRef<GraphEngine | null>(null);
  const machine = useGraphHistoryMachine();

  const normalizedSeries = normalizeDataSeries(newData);
  const [visibleSeries, setVisibleSeries] = useState<boolean[]>(() =>
    new Array(normalizedSeries.length).fill(true),
  );
  const [cursorValues, setCursorValues] = useState<(number | null)[] | null>(
    null,
  );
  const [status, setStatus] = useState<{
    loading: boolean;
    error: string | null;
  }>({ loading: false, error: null });

  if (visibleSeries.length !== normalizedSeries.length) {
    setVisibleSeries(new Array(normalizedSeries.length).fill(true));
  }

  const colors = {
    grid: config.colors?.grid ?? DEFAULT_COLORS.grid,
    axis: config.colors?.axis ?? DEFAULT_COLORS.axis,
    background: config.colors?.background ?? DEFAULT_COLORS.background,
  };

  const seriesColor = (series: SeriesData, index: number) =>
    series.color ||
    (normalizedSeries.length === 1 ? config.colors?.primary : undefined) ||
    DEFAULT_SERIES_COLORS[index % DEFAULT_SERIES_COLORS.length];

  // Latest props for callbacks that outlive a render (export, engine callbacks).
  const latestRef = useRef({
    normalizedSeries,
    config,
    unit,
    renderValue,
    syncGraph,
    onChart,
  });
  useEffect(() => {
    latestRef.current = {
      normalizedSeries,
      config,
      unit,
      renderValue,
      syncGraph,
      onChart,
    };
  });

  // --- engine lifecycle (re-created only if the machine or cursor group changes) ---
  const cursorGroup = syncGraph?.cursorGroup ?? graphId;
  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    const engine = new GraphEngine(canvas, graphId, machine, cursorGroup, {
      onCursor: (values) =>
        setCursorValues((prev) => (sameValues(prev, values) ? prev : values)),
      onUserRange: (range) =>
        latestRef.current.syncGraph?.onZoomChange?.(graphId, range),
      onStatus: (next) =>
        setStatus((prev) =>
          prev.loading === next.loading && prev.error === next.error
            ? prev
            : next,
        ),
    });
    engineRef.current = engine;
    latestRef.current.onChart?.(engine.instance as Chart);
    return () => {
      engine.dispose();
      engineRef.current = null;
      latestRef.current.onChart?.(null);
    };
  }, [graphId, machine, cursorGroup]);

  // --- push inputs to the engine on every render ---
  const targets = useMemo(
    () => collectTargets(normalizedSeries),
    [normalizedSeries],
  );

  useEffect(() => {
    const engine = engineRef.current;
    if (!engine) return;

    const timeWindow =
      syncGraph?.timeWindow ?? config.defaultTimeWindow ?? 30 * 60 * 1000;
    const hLines: HLine[] = [];
    const markerList: Marker[] = [...(markers ?? EMPTY_MARKERS)];
    const addLine = (
      line: GraphLine,
      color: string,
      seriesVisible: boolean,
    ) => {
      if (line.show === false || !seriesVisible) return;
      if (line.type === "threshold") {
        hLines.push({
          value: line.value,
          color: line.color || color,
          dash: line.dash,
          width: line.width,
        });
      } else if (line.type === "user_marker") {
        markerList.push({
          timestamp: line.markerTimestamp,
          name: line.label,
          color: line.color,
        });
      }
    };
    for (const line of config.lines ?? [])
      addLine(line, DEFAULT_COLORS.primary, true);
    normalizedSeries.forEach((s, i) => {
      for (const line of s.lines ?? [])
        addLine(line, seriesColor(s, i), visibleSeries[i] ?? true);
    });

    const engineTargets: EngineTarget[] = targets.map(
      ({ line, seriesIndex }) => ({
        seriesIndex,
        color:
          line.color || seriesColor(normalizedSeries[seriesIndex], seriesIndex),
        value: line.value,
        history: line.history,
        targetSeries: line.targetSeries,
        dash: line.dash,
        width: line.width,
        visible: line.show !== false && (visibleSeries[seriesIndex] ?? true),
      }),
    );

    const inputs: EngineInputs = {
      series: normalizedSeries.map((s, i) => ({
        newData: s.newData,
        history: s.history,
        historyScale: s.historyScale,
        color: seriesColor(s, i),
        visible: visibleSeries[i] ?? true,
      })),
      targets: engineTargets,
      hLines,
      markers: markerList,
      view: {
        live: syncGraph?.isLiveMode ?? true,
        windowMs: timeWindow === "all" ? HISTORY_RETENTION_MS : timeWindow,
        all: timeWindow === "all",
        range: syncGraph?.isLiveMode ? undefined : syncGraph?.xRange,
        showFrom: syncGraph?.showFromTimestamp ?? null,
        freeze: syncGraph?.historicalFreezeTimestamp ?? null,
      },
      renderValue,
      colors,
    };
    engine.setInputs(inputs);
  });

  // --- export: one entry per series, with the history + live data in view ---
  useEffect(() => {
    if (!onRegisterForExport) return;
    const count = normalizedSeries.length;
    for (let index = 0; index < count; index++) {
      const seriesId = `${graphId}-series-${index}`;
      onRegisterForExport(seriesId, () => {
        const current = latestRef.current;
        const series = current.normalizedSeries[index];
        const engine = engineRef.current;
        if (!series?.newData && !series?.history) return null;
        if (!engine) return null;

        const points = engine.exportSeries(index);
        const allTargets = collectTargets(current.normalizedSeries);
        const lines = (series.lines ?? []).map((line) => {
          if (line.type !== "target") return line;
          const targetIndex = allTargets.findIndex((t) => t.line === line);
          const steps =
            targetIndex >= 0 ? engine.exportTarget(targetIndex) : null;
          return steps && steps.t.length
            ? { ...line, targetSeries: timeSeriesFromPoints(steps.t, steps.v) }
            : line;
        });

        return {
          config: {
            ...current.config,
            lines: [...(current.config.lines ?? []), ...lines],
          },
          data: {
            newData: timeSeriesFromPoints(points.t, points.v),
            title: series.title || `Series ${index + 1}`,
            color: series.color,
            lines,
          },
          unit: current.unit,
          renderValue: current.renderValue,
        };
      });
    }
    return () => {
      for (let index = 0; index < count; index++) {
        onUnregisterFromExport?.(`${graphId}-series-${index}`);
      }
    };
  }, [
    onRegisterForExport,
    onUnregisterFromExport,
    graphId,
    normalizedSeries.length,
  ]);

  const toggleSeries = useCallback((index: number) => {
    setVisibleSeries((prev) => {
      const visibleCount = prev.filter(Boolean).length;
      if (prev[index] && visibleCount === 1) return prev;
      const next = [...prev];
      next[index] = !next[index];
      return next;
    });
  }, []);

  const getSeriesDisplayValue = (seriesIndex: number) => {
    const cursor = cursorValues?.[seriesIndex];
    if (cursor !== undefined && cursor !== null) return cursor;
    return normalizedSeries[seriesIndex]?.newData?.current?.value ?? null;
  };

  return (
    <div className="h-[50vh] w-full">
      <div className="flex h-full w-full flex-col overflow-hidden rounded-3xl border border-gray-200 bg-white shadow">
        <div className="flex shrink-0 items-center justify-between gap-4 pt-4 pr-5 pb-6 pl-6">
          <div className="mt-1 flex min-w-0 flex-1 items-center gap-4">
            <Icon
              name={unit ? getUnitIcon(unit) : "lu:TrendingUp"}
              className="size-6 text-gray-600"
            />

            <h2 className="text-2xl leading-none font-bold text-gray-900">
              {config.title}
            </h2>
          </div>

          <div className="flex shrink-0 items-center gap-4">
            {normalizedSeries.length === 1 && (
              <ControlCard className="rounded-lg border-gray-200/80 bg-gray-50/80 px-4 py-3 shadow-sm">
                <div className="flex items-center gap-2 text-base text-gray-600">
                  <span className="font-mono leading-none font-bold text-gray-900">
                    {formatDisplayValue(getSeriesDisplayValue(0), renderValue)}
                  </span>
                  <span className="leading-none text-gray-500">
                    {renderUnitSymbol(unit)}
                  </span>
                </div>
              </ControlCard>
            )}

            {normalizedSeries.length > 1 && (
              <div className="flex items-center gap-1">
                {normalizedSeries.map((series, index) => {
                  const color = seriesColor(series, index);
                  const formattedValue = formatDisplayValue(
                    getSeriesDisplayValue(index),
                    renderValue,
                  );
                  const isLastVisible =
                    visibleSeries[index] &&
                    visibleSeries.filter(Boolean).length === 1;
                  const seriesLines = series.lines || [];

                  return (
                    <TouchButton
                      key={index}
                      onClick={() => {
                        if (!isLastVisible) toggleSeries(index);
                      }}
                      className={`rounded-md px-4 py-1.5 text-sm transition-all ${
                        visibleSeries[index]
                          ? "text-white shadow-md"
                          : "bg-gray-100 text-gray-500 hover:bg-gray-200"
                      } ${isLastVisible ? "cursor-not-allowed" : ""}`}
                      style={{
                        backgroundColor: visibleSeries[index]
                          ? color
                          : undefined,
                      }}
                      title={`${series.title || `Series ${index + 1}`}: ${formattedValue} ${renderUnitSymbol(unit) || ""}${
                        seriesLines.length > 0
                          ? `\nLines: ${seriesLines.map((l) => l.label || `${l.value}`).join(", ")}`
                          : ""
                      }`}
                    >
                      <div className="flex flex-col items-center">
                        <span className="text-xs font-medium">
                          {series.title || `S${index + 1}`}
                        </span>
                        <span
                          className={`font-mono leading-none font-bold ${
                            visibleSeries[index]
                              ? "text-white"
                              : "text-gray-500"
                          }`}
                        >
                          {formattedValue} {renderUnitSymbol(unit)}
                        </span>
                      </div>
                    </TouchButton>
                  );
                })}
              </div>
            )}
          </div>
        </div>

        <div className="-mt-2 px-6">
          <div className="h-px bg-gray-200"></div>
        </div>

        <div className="flex-1 overflow-hidden rounded-b-3xl pt-4 pr-2 pb-2 pl-2">
          <div
            className="relative h-full w-full touch-none overflow-hidden select-none"
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
