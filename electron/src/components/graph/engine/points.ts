import type { Series, TimeSeries } from "@/lib/timeseries";

/** A chart point in Unix ms. `y: null` breaks the line (a gap in the data). */
export type Point = { x: number; y: number | null };

/** Index of the first point with `x >= value` (binary search; `data` sorted by x). */
export function lowerBoundX(data: Point[], x: number): number {
  let lo = 0;
  let hi = data.length;
  while (lo < hi) {
    const mid = (lo + hi) >>> 1;
    if (data[mid].x < x) lo = mid + 1;
    else hi = mid;
  }
  return lo;
}

/** A line break between `a` and `b` if they are further apart than `gapMs`. */
export function gapBetween(
  a: number,
  b: number | undefined,
  gapMs: number,
): Point[] {
  return b !== undefined && b - a > gapMs ? [{ x: a + 1, y: null }] : [];
}

// --- live ring buffers (`lib/timeseries`) ---

/** Ring-buffer entries in chronological order. */
export function bufferPoints(series: Series): Point[] {
  const out: Point[] = [];
  const { values, index, size, validCount } = series;
  const start = validCount < size ? 0 : index;
  for (let i = 0; i < validCount; i++) {
    const v = values[(start + i) % size];
    if (v && v.timestamp > 0) out.push({ x: v.timestamp, y: v.value });
  }
  return out;
}

/** Timestamp of the oldest entry in the long (1 s) buffer, or null if empty. */
export function oldestBufferedTimestamp(ts: TimeSeries): number | null {
  const long = ts.long;
  if (long.validCount === 0) return null;
  const start = long.validCount < long.size ? 0 : long.index;
  const first = long.values[start];
  return first && first.timestamp > 0 ? first.timestamp : null;
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
