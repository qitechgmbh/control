import { Chart } from "chart.js";
import type { MachineIdentificationUnique } from "@/machines/types";
import { HISTORY_RETENTION_MS } from "@/lib/history";
import { HistoryLoader } from "../history/HistoryLoader";
import type { ViewRange } from "../types";
import { registerChartJs } from "./registerChartJs";
import { createChartOptions } from "./chartOptions";
import { DatasetBuilder } from "./DatasetBuilder";
import { TouchGestures } from "./TouchGestures";
import { Viewport } from "./Viewport";
import { lowerBoundX, oldestBufferedTimestamp, type Point } from "./points";
import { bandPlugin } from "./plugins/band";
import { thresholdsPlugin } from "./plugins/thresholds";
import { markersPlugin } from "./plugins/markers";
import { crosshairPlugin, subscribeCursor } from "./plugins/crosshair";
import type { EngineCallbacks, EngineInputs } from "./types";

/** Live redraw rate. */
const FRAME_INTERVAL_MS = 1000 / 30;

/**
 * Owns one Chart.js line chart, without React: loads history from the database, appends
 * live points from the in-memory buffers, and handles pan/zoom, the synced crosshair
 * and the overlays (thresholds, markers).
 *
 * - `Viewport` decides the visible range (shared view or this chart's gesture range).
 * - `DatasetBuilder` turns history + live buffers + setpoints into datasets.
 * - `HistoryLoader` fetches what the view needs from the database.
 */
export class GraphEngine {
  private readonly chart: Chart<"line", Point[]>;
  private readonly viewport = new Viewport();
  private readonly datasets = new DatasetBuilder();
  private readonly touch: TouchGestures;
  private readonly unsubscribeCursor: () => void;
  private readonly observer: IntersectionObserver | null = null;

  private inputs: EngineInputs | null = null;
  private loader: HistoryLoader | null = null;
  private loaderKey = "";
  private crosshairX: number | null = null;

  private dirty = true;
  private rafId: number | null = null;
  private retryTimer: ReturnType<typeof setTimeout> | null = null;
  private lastFrameAt = 0;
  private visible = true;
  private disposed = false;

  constructor(
    canvas: HTMLCanvasElement,
    graphId: string,
    private readonly machine: MachineIdentificationUnique | null,
    cursorGroup: string,
    private readonly callbacks: EngineCallbacks,
  ) {
    registerChartJs();
    this.chart = new Chart<"line", Point[]>(canvas, {
      type: "line",
      data: { datasets: [] },
      options: createChartOptions({
        colors: () => this.inputs?.colors,
        renderValue: () => this.inputs?.renderValue,
        onGestureStart: (chart) => this.viewport.beginGesture(axisRange(chart)),
        onGesture: (chart, done) => this.onMouseGesture(chart, done),
      }),
      plugins: [
        bandPlugin(),
        thresholdsPlugin(() => this.inputs?.hLines ?? []),
        markersPlugin(() => ({
          markers: this.inputs?.markers ?? [],
          valueAt: (timestamp) => this.firstVisibleValueAt(timestamp),
        })),
        crosshairPlugin({
          group: cursorGroup,
          sourceId: graphId,
          getX: () => this.crosshairX,
        }),
      ],
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
          range: this.viewport.gestureBase(axisRange(this.chart)),
        };
      },
      onStart: () => this.viewport.beginGesture(null),
      onMove: (range) => this.applyGestureRange(range, false),
      onEnd: (changed) => {
        const pending = this.viewport.pendingRange;
        if (changed && pending) this.applyGestureRange(pending, true);
        else this.viewport.endGesture();
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

  dispose() {
    this.disposed = true;
    if (this.rafId !== null) cancelAnimationFrame(this.rafId);
    if (this.retryTimer !== null) clearTimeout(this.retryTimer);
    this.observer?.disconnect();
    this.touch.dispose();
    this.unsubscribeCursor();
    this.loader?.dispose();
    this.chart.destroy();
  }

  setInputs(inputs: EngineInputs) {
    this.inputs = inputs;
    this.viewport.follow(inputs.view);
    this.updateLoader(inputs);
    this.dirty = true;
    this.requestFrame();
  }

  // ---------------------------------------------------------------- export

  /** The time range on screen, the default range of an export. */
  visibleRange(): ViewRange | null {
    const scale = this.chart.scales.x;
    if (!scale || !Number.isFinite(scale.min) || !Number.isFinite(scale.max)) {
      return null;
    }
    return { min: scale.min, max: scale.max };
  }

  // ---------------------------------------------------------------- history

  /** Replaces the loader when the machine or the requested names change. */
  private updateLoader(inputs: EngineInputs) {
    const measurements = inputs.series.map((s) => s.history).filter(isName);
    const targets = inputs.targets.map((t) => t.history).filter(isName);
    const key = JSON.stringify([this.machine, measurements, targets]);
    if (key === this.loaderKey) return;

    this.loaderKey = key;
    this.loader?.dispose();
    this.datasets.reset();
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
    const live = this.viewport.isLive(inputs.view);
    if (live && ts - this.lastFrameAt < FRAME_INTERVAL_MS) {
      this.requestFrame();
      return;
    }
    this.lastFrameAt = ts;

    const now = Date.now();
    const view = this.viewport.resolve(
      inputs.view,
      axisRange(this.chart),
      now,
      () => this.loader?.earliest() ?? this.oldestBuffered(),
    );
    const plotWidth = this.chart.chartArea?.width || this.chart.width || 800;
    const retryInMs = this.loader?.ensure(view, live, plotWidth, now) ?? null;
    if (retryInMs !== null && !live) this.scheduleRetry(retryInMs);

    if (this.dirty || live) {
      this.chart.data.datasets = this.datasets.build(
        inputs,
        this.loader,
        view,
        now,
      );
      this.dirty = false;
    }

    if (!this.viewport.isInteracting) this.setAxisRange(view);
    const limits = this.chart.options.plugins!.zoom!.limits!.x!;
    limits.min = now - HISTORY_RETENTION_MS;
    limits.max = now;

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

  private oldestBuffered(): number | null {
    let oldest: number | null = null;
    for (const s of this.inputs?.series ?? []) {
      const t = s.newData ? oldestBufferedTimestamp(s.newData) : null;
      if (t !== null && (oldest === null || t < oldest)) oldest = t;
    }
    return oldest;
  }

  private setAxisRange(range: ViewRange) {
    const x = this.chart.options.scales!.x!;
    x.min = range.min;
    x.max = range.max;
  }

  // ---------------------------------------------------------------- pan / zoom

  private onMouseGesture(chart: Chart, done: boolean) {
    const range = axisRange(chart);
    if (this.viewport.isNoop(range)) {
      if (done) this.viewport.endGesture();
      return;
    }
    this.applyGestureRange(range, done);
  }

  /** Shows a pan/zoom range on this chart and syncs it to the other graphs. */
  private applyGestureRange(requested: ViewRange, done: boolean) {
    const { range, sync } = this.viewport.applyGesture(
      requested,
      done,
      Date.now(),
    );
    this.setAxisRange(range);
    if (sync) this.callbacks.onUserRange(range);
    this.dirty = true;
    this.requestFrame();
  }

  // ---------------------------------------------------------------- cursor

  private get seriesCount(): number {
    return this.inputs?.series.length ?? 0;
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

  private firstVisibleValueAt(timestamp: number): number | null {
    const index = this.inputs?.series.findIndex((s) => s.visible) ?? -1;
    return index < 0 ? null : this.valueAt(index, timestamp);
  }

  /** Value of a dataset at `x`: the nearest point. */
  private valueAt(datasetIndex: number, x: number): number | null {
    const data = this.chart.data.datasets[datasetIndex]?.data;
    if (!data || data.length === 0) return null;
    let i = lowerBoundX(data, x);
    if (i >= data.length) i = data.length - 1;
    if (i > 0 && Math.abs(data[i - 1].x - x) < Math.abs(data[i].x - x)) i--;
    return data[i].y;
  }
}

function axisRange(chart: Chart): ViewRange {
  return { min: chart.scales.x.min, max: chart.scales.x.max };
}

function isName(v: string | undefined): v is string {
  return typeof v === "string" && v.length > 0;
}
