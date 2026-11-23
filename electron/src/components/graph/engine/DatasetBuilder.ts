import type { ChartDataset } from "chart.js";
import type { BucketSeries } from "@/lib/history";
import type { HistoryLoader } from "../history/HistoryLoader";
import type { ViewRange } from "../types";
import type { EngineInputs, EngineSeries, EngineTarget } from "./types";
import type { BandDataset } from "./plugins/band";
import {
  bucketPoints,
  bufferPoints,
  collectTail,
  gapBetween,
  type BandPoint,
  type Point,
} from "./points";

/** Gaps longer than this (or 5 buckets) break the line. */
const MIN_GAP_MS = 10_000;

type LineDataset = ChartDataset<"line", Point[]> & BandDataset;

type CachedHistory = {
  version: number;
  scale: number;
  /** Bucket means (the line). */
  points: Point[];
  /** Bucket min/max (the band). Empty if the backend sent none. */
  band: BandPoint[];
  lastT: number;
  /** Live points newer than this continue the history: the end of its last bucket. */
  tailAfter: number;
};

/**
 * Builds the Chart.js datasets of one chart: one per series (database history followed
 * by the live tail) and one step line per target, in that order. A series line is the
 * mean of each bucket; its min/max ride along as `band` for the band plugin. The live
 * tail is bucketed the same way, so it is no noisier than the history before it.
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
        this.seriesDataset(series, loader, view, gapMs, bucketMs),
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
    bucketMs: number,
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
            bucketMs,
            series.historyScale ?? 1,
          )
        : null;
    const after = history
      ? history.tailAfter
      : view.min - (view.max - view.min);
    let tail = series.newData ? collectTail(series.newData, after) : [];
    let tailBand: BandPoint[] = [];
    if (bucketMs > 0 && tail.length) {
      ({ points: tail, band: tailBand } = bucketPoints(tail, bucketMs));
    }

    let data: Point[];
    let band: BandPoint[];
    if (history && history.points.length) {
      if (tail.length) {
        const gap = gapBetween(history.lastT, tail[0].x, gapMs);
        data = history.points.concat(gap, tail);
        band = history.band.length
          ? history.band.concat(
              gap.map((p) => ({ x: p.x, lo: null, hi: null })),
              tailBand,
            )
          : [];
      } else {
        data = history.points;
        band = history.band;
      }
    } else {
      data = tail;
      band = tailBand;
    }

    return {
      data,
      band,
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
    db: BucketSeries,
    gapMs: number,
    bucketMs: number,
    scale: number,
  ): CachedHistory {
    const version = loader.version;
    const hit = this.historyCache.get(name);
    if (hit && hit.version === version && hit.scale === scale) return hit;

    const hasBand =
      db.min?.length === db.t.length && db.max?.length === db.t.length;
    const points: Point[] = [];
    const band: BandPoint[] = [];
    let prev = -Infinity;
    for (let k = 0; k < db.t.length; k++) {
      const x = db.t[k];
      if (x - prev > gapMs && points.length) {
        points.push({ x: prev + 1, y: null });
        if (hasBand) band.push({ x: prev + 1, lo: null, hi: null });
      }
      points.push({ x, y: db.v[k] * scale });
      if (hasBand)
        band.push({ x, lo: db.min[k] * scale, hi: db.max[k] * scale });
      prev = x;
    }
    const lastT = db.t.length ? db.t[db.t.length - 1] : -Infinity;
    const entry = {
      version,
      scale,
      points,
      band,
      lastT,
      // `lastT` is the mean time of the last bucket; the tail takes over at its end.
      tailAfter:
        bucketMs > 0 && Number.isFinite(lastT)
          ? (Math.floor(lastT / bucketMs) + 1) * bucketMs - 1
          : lastT,
    };
    this.historyCache.set(name, entry);
    return entry;
  }
}
