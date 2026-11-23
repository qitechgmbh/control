import * as XLSX from "xlsx";
import type { ViewRange } from "../types";

/** Columnar points: `t[i]` (Unix ms) belongs to `v[i]`. */
export type Points = { t: number[]; v: number[] };

/** One line of a graph, as it goes into the workbook. */
export type ExcelSeries = {
  graphTitle: string;
  /** Series title, or the graph title for single-series graphs. */
  title: string;
  /** Unit symbol, `""` if none. */
  unit: string;
  points: Points;
  /** Setpoint steps: `v[i]` is in effect from `t[i]` on. */
  target?: Points;
};

export type ExcelReport = {
  title: string;
  range: ViewRange;
  /** Width of the averaged buckets in ms, 0 for raw samples. */
  bucketMs: number;
  series: ExcelSeries[];
  markers: { timestamp: number; name: string }[];
};

const SOFTWARE = "QiTech Control";
const DATE_FORMAT = "yyyy-mm-dd hh:mm:ss.000";
const NUMBER_FORMAT = "0.000";
const OVERVIEW_SHEET = "Overview";
const ALL_VALUES_SHEET = "All values";

type Cell = XLSX.CellObject | string | number | null;

/**
 * Builds the Excel report: an overview (range, statistics, markers), all values side by
 * side on one time axis, and one sheet per series with its target. Every value and time
 * is a real number/date cell, so Excel can chart and calculate with it.
 */
export function buildExcelWorkbook(report: ExcelReport): Uint8Array {
  const series = report.series.filter((s) => s.points.t.length > 0);
  if (series.length === 0) {
    throw new Error("No data in the selected time range");
  }

  const workbook = XLSX.utils.book_new();
  const sheetNames = new SheetNames([OVERVIEW_SHEET, ALL_VALUES_SHEET]);
  const perSeriesNames = series.map((s) => sheetNames.take(s.title));

  XLSX.utils.book_append_sheet(
    workbook,
    overviewSheet(report, series),
    OVERVIEW_SHEET,
  );
  XLSX.utils.book_append_sheet(
    workbook,
    allValuesSheet(report, series),
    ALL_VALUES_SHEET,
  );
  series.forEach((s, i) =>
    XLSX.utils.book_append_sheet(workbook, seriesSheet(s), perSeriesNames[i]),
  );

  return XLSX.write(workbook, { type: "buffer", bookType: "xlsx" });
}

// ------------------------------------------------------------------ sheets

function overviewSheet(
  report: ExcelReport,
  series: ExcelSeries[],
): XLSX.WorkSheet {
  const rows: Cell[][] = [
    [text(report.title)],
    [],
    ["From", date(report.range.min)],
    ["To", date(report.range.max)],
    ["Duration (h)", num((report.range.max - report.range.min) / 3_600_000)],
    [
      "Resolution",
      report.bucketMs > 0
        ? `Mean of ${formatMs(report.bucketMs)} intervals`
        : "Every recorded sample",
    ],
    ["Exported", date(Date.now())],
    ["Software", SOFTWARE],
    [],
    [
      "Graph",
      "Value",
      "Unit",
      "Samples",
      "Min",
      "Max",
      "Mean",
      "Std. dev.",
      "25 %",
      "Median",
      "75 %",
      "Target (end)",
    ],
  ];

  for (const s of series) {
    const stats = statistics(s.points.v);
    const lastTarget = s.target?.v.at(-1);
    rows.push([
      text(s.graphTitle),
      text(s.title),
      text(s.unit),
      s.points.v.length,
      num(stats.min),
      num(stats.max),
      num(stats.mean),
      num(stats.stdDev),
      num(stats.p25),
      num(stats.p50),
      num(stats.p75),
      lastTarget === undefined ? null : num(lastTarget),
    ]);
  }

  const markers = report.markers
    .filter(
      (m) => m.timestamp >= report.range.min && m.timestamp <= report.range.max,
    )
    .sort((a, b) => a.timestamp - b.timestamp);
  if (markers.length > 0) {
    rows.push([], ["Markers"], ["Time", "Name"]);
    for (const m of markers) rows.push([date(m.timestamp), text(m.name)]);
  }

  return sheet(rows, [22, 22, 8, 10, 12, 12, 12, 12, 12, 12, 12, 12]);
}

/** Every series on one shared time axis, plus elapsed seconds and markers. */
function allValuesSheet(
  report: ExcelReport,
  series: ExcelSeries[],
): XLSX.WorkSheet {
  const times = new Set<number>();
  for (const s of series) for (const t of s.points.t) times.add(t);
  const axis = Float64Array.from(times).sort();
  const start = axis[0];

  // One row index per timestamp, so each series is placed with a lookup instead of a merge.
  const rowOf = new Map<number, number>();
  axis.forEach((t, i) => rowOf.set(t, i));
  const columns: (number | null)[][] = series.map((s) => {
    const column: (number | null)[] = new Array(axis.length).fill(null);
    s.points.t.forEach((t, i) => (column[rowOf.get(t)!] = s.points.v[i]));
    return column;
  });

  // Each marker goes to the first row at or after it.
  const markerAt = new Map<number, string[]>();
  for (const m of report.markers) {
    const row = lowerBound(axis, m.timestamp);
    if (row >= axis.length || axis[row] - m.timestamp > 60_000) continue;
    markerAt.set(row, [...(markerAt.get(row) ?? []), m.name]);
  }

  const rows: Cell[][] = [
    [
      "Time",
      "Seconds",
      ...series.map((s) => text(withUnit(columnTitle(s, series), s.unit))),
      "Marker",
    ],
  ];
  axis.forEach((t, i) => {
    rows.push([
      date(t),
      num((t - start) / 1000),
      ...columns.map((column) => (column[i] === null ? null : num(column[i]!))),
      markerAt.has(i) ? text(markerAt.get(i)!.join("; ")) : null,
    ]);
  });

  return sheet(rows, [24, 10, ...series.map(() => 16), 30]);
}

function seriesSheet(s: ExcelSeries): XLSX.WorkSheet {
  const targetAt = s.target ? stepLookup(s.target) : undefined;
  const rows: Cell[][] = [
    ["Time", text(withUnit(s.title, s.unit)), ...(targetAt ? ["Target"] : [])],
  ];
  s.points.t.forEach((t, i) => {
    const target = targetAt?.(t);
    rows.push([
      date(t),
      num(s.points.v[i]),
      ...(targetAt ? [target === undefined ? null : num(target)] : []),
    ]);
  });
  return sheet(rows, [24, 18, 12]);
}

// ------------------------------------------------------------------ cells

function sheet(rows: Cell[][], widths: number[]): XLSX.WorkSheet {
  const worksheet = XLSX.utils.aoa_to_sheet(rows);
  worksheet["!cols"] = widths.map((wch) => ({ wch }));
  return worksheet;
}

/** Excel date cell in local time (Excel has no time zones). */
function date(ms: number): XLSX.CellObject {
  const offsetMs = new Date(ms).getTimezoneOffset() * 60_000;
  return { t: "n", v: (ms - offsetMs) / 86_400_000 + 25_569, z: DATE_FORMAT };
}

function num(value: number): XLSX.CellObject | null {
  return Number.isFinite(value) ? { t: "n", v: value, z: NUMBER_FORMAT } : null;
}

/* eslint-disable no-control-regex */
const INVALID_XML_CHARS =
  /[\u0000-\u0008\u000B\u000C\u000E-\u001F\u007F\uD800-\uDFFF￾￿]/g;
/* eslint-enable no-control-regex */

/** Text without characters that make the .xlsx XML invalid. */
function text(value: string): string {
  return value.replace(INVALID_XML_CHARS, "");
}

// ------------------------------------------------------------------ helpers

function withUnit(title: string, unit: string): string {
  return unit ? `${title} [${unit}]` : title;
}

/** Series title, prefixed with its graph when another series has the same title. */
function columnTitle(s: ExcelSeries, all: ExcelSeries[]): string {
  const ambiguous = all.some((o) => o !== s && o.title === s.title);
  return ambiguous && s.graphTitle !== s.title
    ? `${s.graphTitle} – ${s.title}`
    : s.title;
}

function formatMs(ms: number): string {
  if (ms < 1000) return `${ms} ms`;
  if (ms < 60_000) return `${ms / 1000} s`;
  if (ms < 3_600_000) return `${ms / 60_000} min`;
  return `${ms / 3_600_000} h`;
}

/** First index with `sorted[i] >= value`. */
function lowerBound(sorted: ArrayLike<number>, value: number): number {
  let lo = 0;
  let hi = sorted.length;
  while (lo < hi) {
    const mid = (lo + hi) >> 1;
    if (sorted[mid] < value) lo = mid + 1;
    else hi = mid;
  }
  return lo;
}

/** Value of a step series at `t` (the last step at or before `t`). */
function stepLookup(steps: Points): (t: number) => number | undefined {
  return (t) => {
    const next = lowerBound(steps.t, t + 1);
    return next > 0 ? steps.v[next - 1] : undefined;
  };
}

type Statistics = {
  min: number;
  max: number;
  mean: number;
  stdDev: number;
  p25: number;
  p50: number;
  p75: number;
};

/** Loops instead of `Math.min(...values)`, which overflows the stack on long series. */
export function statistics(values: number[]): Statistics {
  const finite = Float64Array.from(values.filter(Number.isFinite)).sort();
  const n = finite.length;
  if (n === 0) {
    return {
      min: NaN,
      max: NaN,
      mean: NaN,
      stdDev: NaN,
      p25: NaN,
      p50: NaN,
      p75: NaN,
    };
  }
  let sum = 0;
  for (const v of finite) sum += v;
  const mean = sum / n;
  let squares = 0;
  for (const v of finite) squares += (v - mean) ** 2;
  const at = (q: number) => finite[Math.min(n - 1, Math.floor(n * q))];
  return {
    min: finite[0],
    max: finite[n - 1],
    mean,
    stdDev: Math.sqrt(squares / n),
    p25: at(0.25),
    p50: at(0.5),
    p75: at(0.75),
  };
}

/** Unique Excel sheet names: at most 31 characters, none of `[]:*?/\`. */
class SheetNames {
  private used: Set<string>;

  constructor(reserved: string[]) {
    this.used = new Set(reserved.map((n) => n.toLowerCase()));
  }

  take(wanted: string): string {
    const base =
      text(wanted)
        .replace(/[[\]:*?/\\]/g, "_")
        .trim()
        .slice(0, 31) || "Sheet";
    let name = base;
    for (let n = 2; this.used.has(name.toLowerCase()); n++) {
      const suffix = ` (${n})`;
      name = base.slice(0, 31 - suffix.length) + suffix;
    }
    this.used.add(name.toLowerCase());
    return name;
  }
}
