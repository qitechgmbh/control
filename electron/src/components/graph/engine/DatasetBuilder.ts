import type { ChartDataset } from "chart.js";
import type { HistorySeries } from "@/lib/history";
import type { HistoryLoader } from "../history/HistoryLoader";
import type { ViewRange } from "../types";
import type { EngineInputs, EngineSeries, EngineTarget } from "./types";
import { bufferPoints, collectTail, gapBetween, type Point } from "./points";

/** Gaps longer than this (or 5 buckets) break the line. */
const MIN_GAP_MS = 10_000;

type LineDataset = ChartDataset<"line", Point[]>;

type CachedHistory = {
  version: number;
  scale: number;
  points: Point[];
  lastT: number;
};

/**
 * Builds the Chart.js datasets of one chart: one per series (database history followed
 * by the live tail) and one step line per target, in that order.
 */
export class DatasetBuilder {
  /** History converted to points, rebuilt only when the loader has new data. */
  private historyCache = new Map<string, CachedHistory>();

  /** Forget cached history (the loader was replaced). */
  reset() {
    this.historyCache.clear();
  }

  build(
    inputs: EngineInputs,
    loader: HistoryLoader | null,
    view: ViewRange,
    now: number,
  ): LineDataset[] {
    const bucketMs = loader?.bucketMs ?? 0;
    const gapMs = Math.max(MIN_GAP_MS, bucketMs * 5);
    return [
      ...inputs.series.map((series) =>
        this.seriesDataset(series, loader, view, gapMs),
      ),
      ...inputs.targets.map((target) =>
        this.targetDataset(target, loader, inputs.view.live, view, now),
      ),
    ];
  }

  private seriesDataset(
    series: EngineSeries,
    loader: HistoryLoader | null,
    view: ViewRange,
    gapMs: number,
  ): LineDataset {
    const db =
      series.history && loader ? loader.measurement(series.history) : undefined;
    const history =
      db && loader
        ? this.historyPoints(
            loader,
            series.history!,
            db,
            gapMs,
            series.historyScale ?? 1,
          )
        : null;
    const after = history ? history.lastT : view.min - (view.max - view.min);
    const tail = series.newData ? collectTail(series.newData, after) : [];

    let data: Point[];
    if (history && history.points.length) {
      data = tail.length
        ? history.points.concat(
            gapBetween(history.lastT, tail[0]?.x, gapMs),
            tail,
          )
        : history.points;
    } else {
      data = tail;
    }

    return {
      data,
      borderColor: series.color,
      backgroundColor: series.color,
      borderWidth: 2,
      pointRadius: 0,
      pointHitRadius: 0,
      hidden: !series.visible,
      spanGaps: false,
      normalized: true,
      parsing: false,
    } as LineDataset;
  }

  private targetDataset(
    target: EngineTarget,
    loader: HistoryLoader | null,
    live: boolean,
    view: ViewRange,
    now: number,
  ): LineDataset {
    const steps =
      target.history && loader ? loader.target(target.history) : undefined;
    const points: Point[] = [];
    if (steps && steps.t.length) {
      for (let k = 0; k < steps.t.length; k++)
        points.push({ x: steps.t[k], y: steps.v[k] });
    } else if (target.targetSeries) {
      for (const p of bufferPoints(target.targetSeries.long)) points.push(p);
    }

    const end = live ? now : view.max;
    if (points.length === 0) {
      points.push({ x: view.min, y: target.value });
    } else if (live && points[points.length - 1].y !== target.value) {
      // Current setpoint not in the database yet.
      points.push({ x: Math.min(end, now), y: target.value });
    }
    // Extend the last step to the right edge.
    points.push({
      x: Math.max(end, points[points.length - 1].x),
      y: points[points.length - 1].y,
    });

    return {
      data: points,
      borderColor: target.color,
      borderWidth: target.width ?? 1.5,
      borderDash: target.dash ?? [5, 5],
      pointRadius: 0,
      pointHitRadius: 0,
      stepped: "after",
      hidden: !target.visible,
      parsing: false,
    } as LineDataset;
  }

  private historyPoints(
    loader: HistoryLoader,
    name: string,
    db: HistorySeries,
    gapMs: number,
    scale: number,
  ): CachedHistory {
    const version = loader.version;
    const hit = this.historyCache.get(name);
    if (hit && hit.version === version && hit.scale === scale) return hit;

    const points: Point[] = [];
    let prev = -Infinity;
    for (let k = 0; k < db.t.length; k++) {
      const x = db.t[k];
      if (x - prev > gapMs && points.length)
        points.push({ x: prev + 1, y: null });
      points.push({ x, y: db.v[k] * scale });
      prev = x;
    }
    const entry = {
      version,
      scale,
      points,
      lastT: db.t.length ? db.t[db.t.length - 1] : -Infinity,
    };
    this.historyCache.set(name, entry);
    return entry;
  }
}
