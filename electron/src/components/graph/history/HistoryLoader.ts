import {
  fetchHistory,
  HistorySeries,
  niceBucketMs,
  HISTORY_RETENTION_MS,
} from "@/lib/history";
import { MachineIdentificationUnique } from "@/machines/types";

/** Points requested per pixel of plot width (one bucket ≈ one pixel, min+max per bucket). */
const BUCKETS_PER_PX = 1;
/** Extra range fetched on each side of the view, as a fraction of the view span. */
const PAD_FRACTION = 0.5;
/** Rows reach ClickHouse in batches (inserter period 2.5 s), so live increments overlap this much. */
const DB_LAG_MS = 5_000;
/** Minimum time between two live increments. */
const LIVE_REFRESH_MIN_MS = 1_000;
/** Minimum time between two full (range) loads, so pan/zoom gestures don't flood the backend. */
const FULL_LOAD_MIN_INTERVAL_MS = 120;
/** Back-off after a failed request. */
const ERROR_BACKOFF_MS = 5_000;

export type ViewRange = { min: number; max: number };

export type HistoryNames = {
  measurements: string[];
  targets: string[];
};

type Loaded = {
  from: number;
  to: number;
  bucketMs: number;
  measurements: Map<string, HistorySeries>;
  targets: Map<string, HistorySeries>;
};

/**
 * Keeps the ClickHouse history for one graph: the visible range plus padding, downsampled
 * to about one bucket per pixel. Full loads happen when the view leaves the loaded range or
 * the zoom level changes the bucket width; in live mode only the newest buckets are fetched
 * and spliced on. At most one request is in flight.
 */
export class HistoryLoader {
  private loaded: Loaded | null = null;
  private inflight: AbortController | null = null;
  private lastFullLoadAt = 0;
  private lastLiveLoadAt = 0;
  private errorUntil = 0;

  /** Incremented whenever the loaded data changes. */
  version = 0;
  loading = false;
  error: string | null = null;

  constructor(
    private readonly machine: MachineIdentificationUnique,
    private readonly names: HistoryNames,
    private readonly onChange: () => void,
  ) {}

  dispose() {
    this.inflight?.abort();
    this.inflight = null;
  }

  measurement(name: string): HistorySeries | undefined {
    return this.loaded?.measurements.get(name);
  }

  target(name: string): HistorySeries | undefined {
    return this.loaded?.targets.get(name);
  }

  get bucketMs(): number | null {
    return this.loaded?.bucketMs ?? null;
  }

  /** Earliest loaded measurement timestamp, used to fit "Show All" to the data. */
  earliest(): number | null {
    let earliest: number | null = null;
    this.loaded?.measurements.forEach((s) => {
      if (s.t.length && (earliest === null || s.t[0] < earliest)) {
        earliest = s.t[0];
      }
    });
    return earliest;
  }

  /**
   * Called every frame (cheap when nothing needs loading). Returns the delay in ms after
   * which a load that is due but held back (throttle, error back-off) can run, else null.
   * A request in flight calls `onChange` when done, so it needs no retry.
   */
  ensure(
    view: ViewRange,
    live: boolean,
    widthPx: number,
    now: number,
  ): number | null {
    if (this.inflight) return null;
    if (this.names.measurements.length + this.names.targets.length === 0) {
      return null;
    }

    const span = Math.max(1, view.max - view.min);
    const bucketMs = niceBucketMs(
      span,
      Math.max(100, widthPx * BUCKETS_PER_PX),
    );
    const loaded = this.loaded;

    const needsFull =
      !loaded ||
      loaded.bucketMs !== bucketMs ||
      view.min < loaded.from ||
      (!live && view.max > loaded.to && loaded.to < now - DB_LAG_MS);

    if (needsFull) {
      if (now < this.errorUntil) return this.errorUntil - now;
      const wait = this.lastFullLoadAt + FULL_LOAD_MIN_INTERVAL_MS - now;
      if (wait > 0) return wait;
      const oldest = now - HISTORY_RETENTION_MS;
      const from = Math.max(oldest, view.min - span * PAD_FRACTION);
      const to = live ? now : Math.min(now, view.max + span * PAD_FRACTION);
      this.lastFullLoadAt = now;
      this.lastLiveLoadAt = now;
      void this.load(from, to, bucketMs, "full");
      return null;
    }

    if (
      live &&
      now >= this.errorUntil &&
      now - this.lastLiveLoadAt >= Math.max(LIVE_REFRESH_MIN_MS, bucketMs)
    ) {
      this.lastLiveLoadAt = now;
      const from = Math.floor((loaded.to - DB_LAG_MS) / bucketMs) * bucketMs;
      void this.load(
        Math.max(from, loaded.from),
        now,
        bucketMs,
        "append",
        view.min - span * PAD_FRACTION,
      );
    }
    return null;
  }

  private async load(
    from: number,
    to: number,
    bucketMs: number,
    mode: "full" | "append",
    trimBefore?: number,
  ) {
    const controller = new AbortController();
    this.inflight = controller;
    this.loading = mode === "full";
    if (mode === "full") this.onChange();

    try {
      const response = await fetchHistory(
        {
          machine: this.machine,
          measurements: this.names.measurements,
          targets: this.names.targets,
          from,
          to,
          bucketMs,
        },
        controller.signal,
      );
      if (controller.signal.aborted) return;

      const toMap = (rec: Record<string, HistorySeries>) =>
        new Map(Object.entries(rec ?? {}));

      if (mode === "full" || !this.loaded) {
        this.loaded = {
          from,
          to,
          bucketMs: response.bucket_ms,
          measurements: toMap(response.measurements),
          targets: toMap(response.targets),
        };
      } else {
        const loaded = this.loaded;
        for (const [name, fresh] of Object.entries(response.measurements)) {
          const merged = spliceFrom(loaded.measurements.get(name), fresh, from);
          loaded.measurements.set(
            name,
            trimBefore !== undefined ? trimFront(merged, trimBefore) : merged,
          );
        }
        for (const [name, fresh] of Object.entries(response.targets)) {
          loaded.targets.set(
            name,
            mergeSteps(loaded.targets.get(name), fresh, from),
          );
        }
        loaded.to = to;
        if (trimBefore !== undefined) {
          loaded.from = Math.max(loaded.from, trimBefore);
        }
      }

      this.error = null;
      this.version++;
    } catch (e) {
      if (controller.signal.aborted) return;
      this.error = e instanceof Error ? e.message : String(e);
      this.errorUntil = Date.now() + ERROR_BACKOFF_MS;
    } finally {
      if (this.inflight === controller) {
        this.inflight = null;
        this.loading = false;
        this.onChange();
      }
    }
  }
}

/** Replace everything at or after `from` in `old` with `fresh`. */
function spliceFrom(
  old: HistorySeries | undefined,
  fresh: HistorySeries,
  from: number,
): HistorySeries {
  if (!old || old.t.length === 0) return fresh;
  const cut = lowerBound(old.t, from);
  return {
    t: old.t.slice(0, cut).concat(fresh.t),
    v: old.v.slice(0, cut).concat(fresh.v),
  };
}

/** Steps: the fresh response starts with the value in effect at `from`, so splice the same way. */
function mergeSteps(
  old: HistorySeries | undefined,
  fresh: HistorySeries,
  from: number,
): HistorySeries {
  const merged = spliceFrom(old, fresh, from);
  // Drop consecutive duplicates introduced by the "value at from" point.
  const t: number[] = [];
  const v: number[] = [];
  for (let i = 0; i < merged.t.length; i++) {
    if (v.length && v[v.length - 1] === merged.v[i]) continue;
    t.push(merged.t[i]);
    v.push(merged.v[i]);
  }
  return { t, v };
}

function trimFront(series: HistorySeries, before: number): HistorySeries {
  const cut = lowerBound(series.t, before);
  if (cut === 0) return series;
  return { t: series.t.slice(cut), v: series.v.slice(cut) };
}

/** First index with `arr[i] >= value` (arr sorted ascending). */
export function lowerBound(arr: ArrayLike<number>, value: number): number {
  let lo = 0;
  let hi = arr.length;
  while (lo < hi) {
    const mid = (lo + hi) >>> 1;
    if (arr[mid] < value) lo = mid + 1;
    else hi = mid;
  }
  return lo;
}
