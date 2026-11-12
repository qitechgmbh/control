import { Chart, type ChartDataset, type Plugin, type Scale } from "chart.js";
import type { Marker } from "@/stores/markerStore";
import type { MachineIdentificationUnique } from "@/machines/types";
import type { TimeSeries, Series } from "@/lib/timeseries";
import { HISTORY_RETENTION_MS, type HistorySeries } from "@/lib/history";
import {
  HistoryLoader,
  lowerBound,
  type ViewRange,
} from "../history/HistoryLoader";
import { registerChartJs } from "./register";
import { TouchGestures } from "./touchGestures";
import { buildTimeTicks, formatTimeTick } from "./timeAxis";
import {
  hLinesRangePlugin,
  overlayPlugin,
  publishCursor,
  setOverlayState,
  subscribeCursor,
  type HLine,
  type OverlayState,
} from "./plugins";

type Point = { x: number; y: number | null };

/** Live redraw rate. */
const FRAME_INTERVAL_MS = 1000 / 30;
/** How often a pan/zoom gesture pushes its range to the other graphs. */
const SYNC_INTERVAL_MS = 80;

export type EngineSeries = {
  newData: TimeSeries | null;
  history?: string;
  historyScale?: number;
  color: string;
  visible: boolean;
};

export type EngineTarget = {
  seriesIndex: number;
  color: string;
  value: number;
  history?: string;
  targetSeries?: TimeSeries;
  dash?: number[];
  width?: number;
  visible: boolean;
};

export type EngineView = {
  live: boolean;
  /** Window length in ms ("all" = database retention, fitted to the data). */
  windowMs: number;
  all: boolean;
  /** Manual range from pan/zoom (historical). */
  range?: ViewRange;
  showFrom?: number | null;
  freeze?: number | null;
};

export type EngineInputs = {
  series: EngineSeries[];
  targets: EngineTarget[];
  hLines: HLine[];
  markers: Marker[];
  view: EngineView;
  renderValue?: (value: number) => string;
  colors: { grid: string; axis: string };
};

export type EngineCallbacks = {
  /** Cursor value per series, or null when the cursor left all synced charts. */
  onCursor: (values: (number | null)[] | null) => void;
  /** User panned/zoomed this chart. */
  onUserRange: (range: ViewRange) => void;
  onStatus: (status: { loading: boolean; error: string | null }) => void;
};

/**
 * Owns one Chart.js line chart: loads history from the database, appends live
 * points from the in-memory buffers, and handles pan/zoom, cursor sync and overlays.
 */
export class GraphEngine {
  private chart: Chart<"line", Point[]>;
  private inputs: EngineInputs | null = null;
  private loader: HistoryLoader | null = null;
  private loaderKey = "";

  private dirty = true;
  private rafId: number | null = null;
  private retryTimer: ReturnType<typeof setTimeout> | null = null;
  private lastFrameAt = 0;
  private visible = true;
  private observer: IntersectionObserver | null = null;

  private interacting = false;
  private userRange: ViewRange | null = null;
  private lastSyncAt = 0;

  private touch: TouchGestures;
  private gestureStart: ViewRange | null = null;
  private crosshairX: number | null = null;
  private unsubscribeCursor: (() => void) | null = null;

  /** Cached {x,y} arrays of the loaded history, rebuilt only when the loader changes. */
  private dbPointCache = new Map<
    string,
    { version: number; scale: number; points: Point[]; lastT: number }
  >();
  private seriesCount = 0;
  private disposed = false;

  constructor(
    canvas: HTMLCanvasElement,
    private readonly graphId: string,
    private readonly machine: MachineIdentificationUnique | null,
    private readonly cursorGroup: string,
    private readonly callbacks: EngineCallbacks,
  ) {
    registerChartJs();
    this.chart = new Chart<"line", Point[]>(canvas, {
      type: "line",
      data: { datasets: [] },
      options: this.buildOptions(),
      plugins: [hLinesRangePlugin, overlayPlugin, this.cursorPlugin()],
    });

    // Touch pan/pinch is ours (direction-locked, page scroll stays native);
    // the zoom plugin only handles mouse drag and Ctrl+wheel / trackpad pinch.
    this.touch = new TouchGestures(canvas, {
      getPlot: () => {
        const area = this.chart.chartArea;
        if (!area) return null;
        return {
          left: area.left,
          width: area.width,
          range: this.userRange ?? {
            min: this.chart.scales.x.min,
            max: this.chart.scales.x.max,
          },
        };
      },
      onStart: () => {
        this.interacting = true;
      },
      onMove: (range) => this.applyGestureRange(range, false),
      onEnd: (changed) => {
        if (changed && this.userRange) {
          this.applyGestureRange(this.userRange, true);
        } else {
          this.interacting = false;
        }
      },
    });

    this.unsubscribeCursor = subscribeCursor(cursorGroup, (x) => {
      this.crosshairX = x;
      this.emitCursorValues();
      this.requestFrame();
    });

    if (typeof IntersectionObserver !== "undefined") {
      this.observer = new IntersectionObserver((entries) => {
        this.visible = entries.some((e) => e.isIntersecting);
        if (this.visible) this.requestFrame();
      });
      this.observer.observe(canvas);
    }
  }

  get instance(): Chart {
    return this.chart as unknown as Chart;
  }

  dispose() {
    this.disposed = true;
    if (this.rafId !== null) cancelAnimationFrame(this.rafId);
    if (this.retryTimer !== null) clearTimeout(this.retryTimer);
    this.observer?.disconnect();
    this.touch.dispose();
    this.unsubscribeCursor?.();
    this.loader?.dispose();
    this.chart.destroy();
  }

  setInputs(inputs: EngineInputs) {
    this.inputs = inputs;
    this.seriesCount = inputs.series.length;

    // A range from outside (control panel, other graph) or live mode ends our local override.
    if (
      !this.interacting &&
      (inputs.view.live || inputs.view.range || inputs.view.showFrom)
    ) {
      this.userRange = null;
    }

    const measurements = inputs.series.map((s) => s.history).filter(isString);
    const targets = inputs.targets.map((t) => t.history).filter(isString);
    const key = JSON.stringify([this.machine, measurements, targets]);
    if (key !== this.loaderKey) {
      this.loaderKey = key;
      this.loader?.dispose();
      this.dbPointCache.clear();
      this.loader =
        this.machine && measurements.length + targets.length > 0
          ? new HistoryLoader(this.machine, { measurements, targets }, () => {
              this.callbacks.onStatus({
                loading: this.loader?.loading ?? false,
                error: this.loader?.error ?? null,
              });
              this.dirty = true;
              this.requestFrame();
            })
          : null;
    }

    this.dirty = true;
    this.requestFrame();
  }

  // ---------------------------------------------------------------- export

  /** Visible points of a series (history + live), for the Excel export. */
  exportSeries(seriesIndex: number): HistorySeries {
    return this.datasetPoints(seriesIndex);
  }

  exportTarget(targetIndex: number): HistorySeries {
    return this.datasetPoints(this.seriesCount + targetIndex);
  }

  private datasetPoints(datasetIndex: number): HistorySeries {
    const data = this.chart.data.datasets[datasetIndex]?.data ?? [];
    const { min, max } = this.chart.scales.x ?? {
      min: -Infinity,
      max: Infinity,
    };
    const t: number[] = [];
    const v: number[] = [];
    for (const p of data) {
      if (p.y === null || p.x < min || p.x > max) continue;
      t.push(p.x);
      v.push(p.y);
    }
    return { t, v };
  }

  // ---------------------------------------------------------------- frame loop

  private requestFrame() {
    if (this.rafId !== null || this.disposed) return;
    this.rafId = requestAnimationFrame((ts) => {
      this.rafId = null;
      this.frame(ts);
    });
  }

  private frame(ts: number) {
    const inputs = this.inputs;
    if (!inputs || this.disposed) return;

    // Off-screen charts do nothing; the IntersectionObserver restarts them.
    if (!this.visible) return;
    const live = inputs.view.live && !this.userRange;
    if (live && ts - this.lastFrameAt < FRAME_INTERVAL_MS) {
      this.requestFrame();
      return;
    }
    this.lastFrameAt = ts;

    const now = Date.now();
    const view = this.currentView(now);
    const plotWidth = this.chart.chartArea?.width || this.chart.width || 800;
    const retryInMs = this.loader?.ensure(view, live, plotWidth, now) ?? null;
    if (retryInMs !== null && !live) this.scheduleRetry(retryInMs);

    if (this.dirty || live) {
      this.rebuildDatasets(view, now);
      this.dirty = false;
    }

    const xOptions = this.chart.options.scales!.x!;
    if (!this.interacting) {
      xOptions.min = view.min;
      xOptions.max = view.max;
    }
    const limits = this.chart.options.plugins!.zoom!.limits!.x!;
    limits.min = now - HISTORY_RETENTION_MS;
    limits.max = now;

    this.updateOverlay();
    this.chart.update("none");
    this.touch.enforceTouchAction();

    if (live) this.requestFrame();
  }

  /** A deferred load must run even if nothing else triggers a frame (historical view, no live data). */
  private scheduleRetry(ms: number) {
    if (this.retryTimer !== null) return;
    this.retryTimer = setTimeout(() => {
      this.retryTimer = null;
      this.requestFrame();
    }, ms);
  }

  private currentView(now: number): ViewRange {
    const inputs = this.inputs!;
    if (this.interacting || this.userRange) {
      const range = this.userRange ?? {
        min: this.chart.scales.x.min,
        max: this.chart.scales.x.max,
      };
      return range;
    }
    const { view } = inputs;
    if (!view.live && view.range) return view.range;

    const end = view.live ? now : (view.freeze ?? now);
    if (view.showFrom)
      return { min: view.showFrom, max: Math.max(view.showFrom + 1000, end) };

    let min = end - view.windowMs;
    if (view.all) {
      const earliest = this.loader?.earliest() ?? this.earliestBuffered();
      if (earliest !== null && earliest > min) min = earliest;
    }
    return { min, max: end };
  }

  private earliestBuffered(): number | null {
    let earliest: number | null = null;
    for (const s of this.inputs?.series ?? []) {
      const long = s.newData?.long;
      if (!long || long.validCount === 0) continue;
      const start = long.validCount < long.size ? 0 : long.index;
      const first = long.values[start];
      if (
        first &&
        first.timestamp > 0 &&
        (earliest === null || first.timestamp < earliest)
      ) {
        earliest = first.timestamp;
      }
    }
    return earliest;
  }

  // ---------------------------------------------------------------- datasets

  private rebuildDatasets(view: ViewRange, now: number) {
    const inputs = this.inputs!;
    const span = view.max - view.min;
    const bucketMs = this.loader?.bucketMs ?? 0;
    const gapMs = Math.max(10_000, bucketMs * 5);
    const datasets: ChartDataset<"line", Point[]>[] = [];

    inputs.series.forEach((series) => {
      const db = series.history
        ? this.loader?.measurement(series.history)
        : undefined;
      const cached = db
        ? this.dbPoints(series.history!, db, gapMs, series.historyScale ?? 1)
        : null;
      const after = cached ? cached.lastT : view.min - span;
      const tail = series.newData ? collectTail(series.newData, after) : [];

      let data: Point[];
      if (cached && cached.points.length) {
        data = tail.length
          ? cached.points.concat(
              gapBetween(cached.lastT, tail[0]?.x, gapMs),
              tail,
            )
          : cached.points;
      } else {
        data = tail;
      }

      datasets.push({
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
      } as ChartDataset<"line", Point[]>);
    });

    for (const target of inputs.targets) {
      const steps = target.history
        ? this.loader?.target(target.history)
        : undefined;
      const points: Point[] = [];
      if (steps && steps.t.length) {
        for (let k = 0; k < steps.t.length; k++)
          points.push({ x: steps.t[k], y: steps.v[k] });
      } else if (target.targetSeries) {
        for (const p of bufferPoints(target.targetSeries.long)) points.push(p);
      }
      const end = inputs.view.live ? now : view.max;
      if (points.length === 0) {
        points.push({ x: view.min, y: target.value });
      } else if (
        inputs.view.live &&
        points[points.length - 1].y !== target.value
      ) {
        // Current setpoint not in the database yet.
        points.push({ x: Math.min(end, now), y: target.value });
      }
      points.push({
        x: Math.max(end, points[points.length - 1].x),
        y: points[points.length - 1].y,
      });

      datasets.push({
        data: points,
        borderColor: target.color,
        borderWidth: target.width ?? 1.5,
        borderDash: target.dash ?? [5, 5],
        pointRadius: 0,
        pointHitRadius: 0,
        stepped: "after",
        hidden: !target.visible,
        parsing: false,
      } as ChartDataset<"line", Point[]>);
    }

    this.chart.data.datasets = datasets;
  }

  private dbPoints(
    name: string,
    db: HistorySeries,
    gapMs: number,
    scale: number,
  ) {
    const version = this.loader!.version;
    const hit = this.dbPointCache.get(name);
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
    this.dbPointCache.set(name, entry);
    return entry;
  }

  // ---------------------------------------------------------------- overlay & cursor

  private updateOverlay() {
    const inputs = this.inputs!;
    const state: OverlayState = {
      hLines: inputs.hLines,
      markers: inputs.markers,
      crosshairX: this.crosshairX,
      markerValue: (timestamp) => {
        const index = inputs.series.findIndex((s) => s.visible);
        return index < 0 ? null : this.valueAt(index, timestamp);
      },
    };
    setOverlayState(this.chart as unknown as Chart, state);
  }

  /** Value of a series at `x`: the nearest point within the bucket/gap tolerance. */
  private valueAt(datasetIndex: number, x: number): number | null {
    const data = this.chart.data.datasets[datasetIndex]?.data;
    if (!data || data.length === 0) return null;
    let i = lowerBoundX(data, x);
    if (i >= data.length) i = data.length - 1;
    if (i > 0 && Math.abs(data[i - 1].x - x) < Math.abs(data[i].x - x)) i--;
    return data[i].y;
  }

  private emitCursorValues() {
    if (this.crosshairX === null) {
      this.callbacks.onCursor(null);
      return;
    }
    const values: (number | null)[] = [];
    for (let i = 0; i < this.seriesCount; i++)
      values.push(this.valueAt(i, this.crosshairX));
    this.callbacks.onCursor(values);
  }

  private cursorPlugin(): Plugin<"line"> {
    return {
      id: "qitechCursor",
      afterEvent: (chart, args) => {
        const e = args.event;
        const nativeType = (e.native as Event | null)?.type;
        // Touch shows the crosshair while the finger is down only.
        if (
          e.type === "mouseout" ||
          nativeType === "touchend" ||
          nativeType === "touchcancel"
        ) {
          publishCursor(this.cursorGroup, null, this.graphId);
          return;
        }
        // Chart.js maps touchstart/touchmove to mousedown/mousemove.
        if (e.type !== "mousemove" && e.type !== "mousedown") return;
        const area = chart.chartArea;
        if (
          e.x === null ||
          e.y === null ||
          e.x < area.left ||
          e.x > area.right ||
          e.y < area.top ||
          e.y > area.bottom
        ) {
          publishCursor(this.cursorGroup, null, this.graphId);
          return;
        }
        publishCursor(
          this.cursorGroup,
          chart.scales.x.getValueForPixel(e.x) ?? null,
          this.graphId,
        );
      },
    };
  }

  // ---------------------------------------------------------------- pan / zoom

  private onGesture(chart: Chart, done: boolean) {
    const range = { min: chart.scales.x.min, max: chart.scales.x.max };
    const start = this.gestureStart;
    // A drag that did not move the time axis (e.g. vertical) must not leave live mode.
    if (
      !this.userRange &&
      start &&
      Math.abs(range.min - start.min) < 1 &&
      Math.abs(range.max - start.max) < 1
    ) {
      if (done) this.interacting = false;
      return;
    }
    this.applyGestureRange(range, done);
  }

  /** Applies a pan/zoom range from a gesture and syncs it to the other graphs. */
  private applyGestureRange(requested: ViewRange, done: boolean) {
    const now = Date.now();
    const range = clampRange(requested, now - HISTORY_RETENTION_MS, now);
    const xOptions = this.chart.options.scales!.x!;
    xOptions.min = range.min;
    xOptions.max = range.max;
    this.userRange = range;
    if (done) this.interacting = false;
    if (done || now - this.lastSyncAt >= SYNC_INTERVAL_MS) {
      this.lastSyncAt = now;
      this.callbacks.onUserRange(range);
    }
    this.dirty = true;
    this.requestFrame();
  }

  private buildOptions() {
    const start = () => {
      this.interacting = true;
      this.gestureStart = {
        min: this.chart.scales.x.min,
        max: this.chart.scales.x.max,
      };
      return true;
    };
    return {
      animation: false as const,
      responsive: true,
      maintainAspectRatio: false,
      parsing: false as const,
      normalized: true,
      events: [
        "mousemove",
        "mouseout",
        "touchstart",
        "touchmove",
        "touchend",
        "touchcancel",
      ] as (keyof HTMLElementEventMap)[],
      interaction: { mode: "x" as const, intersect: false },
      layout: { padding: { right: 12, left: 4 } },
      scales: {
        x: {
          type: "linear" as const,
          afterBuildTicks: (scale: Scale) => {
            const maxTicks = Math.max(2, Math.floor(scale.width / 110));
            const { ticks, step } = buildTimeTicks(
              scale.min,
              scale.max,
              maxTicks,
            );
            scale.ticks = ticks;
            (scale as Scale & { qtStep?: number }).qtStep = step;
          },
          ticks: {
            autoSkip: false,
            maxRotation: 0,
            color: () => this.inputs?.colors.axis ?? "#64748b",
            callback: function (
              this: Scale & { qtStep?: number },
              value: string | number,
            ) {
              return formatTimeTick(
                Number(value),
                this.qtStep ?? 1000,
                this.max - this.min,
              );
            },
          },
          grid: { color: () => this.inputs?.colors.grid ?? "#e2e8f0" },
        },
        y: {
          type: "linear" as const,
          grace: "5%",
          ticks: {
            color: () => this.inputs?.colors.axis ?? "#64748b",
            callback: (value: string | number) => {
              const render = this.inputs?.renderValue;
              return render ? render(Number(value)) : String(value);
            },
          },
          grid: { color: () => this.inputs?.colors.grid ?? "#e2e8f0" },
        },
      },
      plugins: {
        legend: { display: false },
        tooltip: { enabled: false },
        zoom: {
          limits: {
            x: {
              min: Date.now() - HISTORY_RETENTION_MS,
              max: Date.now(),
              minRange: 1000,
            },
          },
          pan: {
            enabled: true,
            mode: "x" as const,
            threshold: 5,
            // Mouse/pen drag only; touch is handled by TouchGestures.
            onPanStart: ({ event }: { event: { pointerType?: string } }) =>
              event.pointerType === "touch" ? false : start(),
            onPan: ({ chart }: { chart: Chart }) =>
              this.onGesture(chart, false),
            onPanComplete: ({ chart }: { chart: Chart }) =>
              this.onGesture(chart, true),
          },
          zoom: {
            // Plain wheel scrolls the page; Ctrl+wheel (and trackpad pinch, which
            // Chromium reports as Ctrl+wheel) zooms.
            wheel: { enabled: true, speed: 0.1, modifierKey: "ctrl" as const },
            pinch: { enabled: false },
            mode: "x" as const,
            onZoomStart: start,
            onZoom: ({ chart }: { chart: Chart }) =>
              this.onGesture(chart, false),
            onZoomComplete: ({ chart }: { chart: Chart }) =>
              this.onGesture(chart, true),
          },
        },
      },
    };
  }
}

// ---------------------------------------------------------------- helpers

/** Keeps the span (min 1 s, max retention) and shifts the range inside [lo, hi]. */
function clampRange(range: ViewRange, lo: number, hi: number): ViewRange {
  const span = Math.min(hi - lo, Math.max(1000, range.max - range.min));
  let min = range.min;
  if (range.max - range.min !== span) {
    const mid = (range.min + range.max) / 2;
    min = mid - span / 2;
  }
  min = Math.max(lo, Math.min(min, hi - span));
  return { min, max: min + span };
}

function isString(v: string | undefined): v is string {
  return typeof v === "string" && v.length > 0;
}

function gapBetween(a: number, b: number | undefined, gapMs: number): Point[] {
  return b !== undefined && b - a > gapMs ? [{ x: a + 1, y: null }] : [];
}

function lowerBoundX(data: Point[], x: number): number {
  let lo = 0;
  let hi = data.length;
  while (lo < hi) {
    const mid = (lo + hi) >>> 1;
    if (data[mid].x < x) lo = mid + 1;
    else hi = mid;
  }
  return lo;
}

/** Ring-buffer entries in chronological order. */
function bufferPoints(series: Series): Point[] {
  const out: Point[] = [];
  const { values, index, size, validCount } = series;
  const start = validCount < size ? 0 : index;
  for (let i = 0; i < validCount; i++) {
    const v = values[(start + i) % size];
    if (v && v.timestamp > 0) out.push({ x: v.timestamp, y: v.value });
  }
  return out;
}

/** Newest-first walk of a ring buffer, stopping at `after`. */
function collectNewer(
  series: Series,
  after: number,
  before: number,
  out: Point[],
) {
  const { values, index, size, validCount } = series;
  for (let i = 1; i <= validCount; i++) {
    const v = values[(index - i + size) % size];
    if (!v || v.timestamp <= 0) continue;
    if (v.timestamp <= after) break;
    if (v.timestamp < before) out.push({ x: v.timestamp, y: v.value });
  }
}

/**
 * Live points newer than `after`: the 1 s buffer up to where the 20 ms buffer starts,
 * then the 20 ms buffer, in chronological order.
 */
export function collectTail(ts: TimeSeries, after: number): Point[] {
  const short: Point[] = [];
  collectNewer(ts.short, after, Infinity, short);
  const shortStart = short.length ? short[short.length - 1].x : Infinity;
  const long: Point[] = [];
  collectNewer(ts.long, after, shortStart, long);
  long.reverse();
  short.reverse();
  return long.concat(short);
}

export { lowerBound };
