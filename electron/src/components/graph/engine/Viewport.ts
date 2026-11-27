import { HISTORY_RETENTION_MS } from "@/lib/history";
import type { ViewRange } from "../types";
import type { EngineView } from "./types";

/** How often a pan/zoom gesture pushes its range to the other graphs. */
const SYNC_INTERVAL_MS = 80;
/** Smallest visible span. */
const MIN_SPAN_MS = 1000;

/**
 * Decides which time range one chart shows. Normally that is the shared page view;
 * while the user pans or zooms this chart it is the chart's own range, until the
 * shared view takes it over again.
 */
export class Viewport {
  private interacting = false;
  /** This chart's own range from a gesture, until the shared view replaces it. */
  private ownRange: ViewRange | null = null;
  private gestureStart: ViewRange | null = null;
  private lastSyncAt = 0;
  /** `EngineView.revision` last seen by `follow`. */
  private seenRevision: number | null = null;

  /** True during a gesture: the chart's axis must not be overwritten. */
  get isInteracting(): boolean {
    return this.interacting;
  }

  /** Live mode that is not overridden by a gesture range. */
  isLive(view: EngineView): boolean {
    return view.live && !this.ownRange;
  }

  /** Range a touch gesture starts from. */
  gestureBase(current: ViewRange): ViewRange {
    return this.ownRange ?? current;
  }

  /**
   * The shared view changed (panel, other graph, live, or this chart's own committed
   * gesture): drop our own range and show the view. Changes during a gesture are
   * ignored; the gesture commits its own range when it ends.
   */
  follow(view: EngineView) {
    if (view.revision === this.seenRevision) return;
    this.seenRevision = view.revision;
    if (!this.interacting) this.ownRange = null;
  }

  /** The range to show at `now`. `current` is the chart's axis, `earliest` the oldest data. */
  resolve(
    view: EngineView,
    current: ViewRange,
    now: number,
    earliest: () => number | null,
  ): ViewRange {
    if (this.interacting || this.ownRange) return this.ownRange ?? current;
    if (!view.live && view.range) return view.range;

    const end = view.live ? now : (view.freeze ?? now);
    if (view.showFrom) {
      return { min: view.showFrom, max: Math.max(view.showFrom + 1000, end) };
    }

    let min = end - view.windowMs;
    if (view.all) {
      const first = earliest();
      if (first !== null && first > min) min = first;
    }
    return { min, max: end };
  }

  // --- gestures ---

  beginGesture(start: ViewRange | null) {
    this.interacting = true;
    this.gestureStart = start;
  }

  endGesture() {
    this.interacting = false;
  }

  /** A drag that did not move the time axis (e.g. vertical) must not leave live mode. */
  isNoop(range: ViewRange): boolean {
    const start = this.gestureStart;
    return (
      !this.ownRange &&
      start !== null &&
      Math.abs(range.min - start.min) < 1 &&
      Math.abs(range.max - start.max) < 1
    );
  }

  /**
   * Takes a range from a gesture, clamped to the data retention. `sync` says whether to
   * push it to the other graphs now (throttled while the gesture runs, always at the end).
   */
  applyGesture(
    requested: ViewRange,
    done: boolean,
    now: number,
  ): { range: ViewRange; sync: boolean } {
    const range = clampRange(requested, now - HISTORY_RETENTION_MS, now);
    this.ownRange = range;
    if (done) this.interacting = false;
    const sync = done || now - this.lastSyncAt >= SYNC_INTERVAL_MS;
    if (sync) this.lastSyncAt = now;
    return { range, sync };
  }

  /** The last gesture range, to commit when a touch gesture ends. */
  get pendingRange(): ViewRange | null {
    return this.ownRange;
  }
}

/** Keeps the span (min 1 s, max `hi - lo`) and shifts the range inside [lo, hi]. */
export function clampRange(
  range: ViewRange,
  lo: number,
  hi: number,
): ViewRange {
  const span = Math.min(hi - lo, Math.max(MIN_SPAN_MS, range.max - range.min));
  let min = range.min;
  if (range.max - range.min !== span) {
    const mid = (range.min + range.max) / 2;
    min = mid - span / 2;
  }
  min = Math.max(lo, Math.min(min, hi - span));
  return { min, max: min + span };
}
