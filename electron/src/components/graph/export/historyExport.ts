import type { Unit } from "@/control/units";
import { renderUnitSymbol } from "@/control/units";
import type { MachineIdentificationUnique } from "@/machines/types";
import {
  fetchHistory,
  HISTORY_EXPORT_URL,
  niceBucketMs,
  type HistoryResponse,
} from "@/lib/history";
import { fetchMarkers } from "@/lib/markers";
import { seriesToUPlotData } from "@/lib/timeseries";
import {
  choosePath,
  downloadToFile,
  writeFile,
  type SaveFileResult,
} from "@/helpers/file_export_helpers";
import type { GraphConfig, GraphLine, SeriesData, ViewRange } from "../types";
import {
  buildExcelWorkbook,
  type ExcelReport,
  type ExcelSeries,
  type Points,
} from "./excelWorkbook";

export type ExportFormat = "xlsx" | "csv" | "parquet";

export const EXPORT_FORMATS: {
  value: ExportFormat;
  label: string;
  description: string;
}[] = [
  {
    value: "xlsx",
    label: "Excel",
    description: "Report with statistics, markers and one sheet per value",
  },
  {
    value: "csv",
    label: "CSV",
    description: "Plain table, opens in any spreadsheet or tool",
  },
  {
    value: "parquet",
    label: "Parquet",
    description: "Compact and typed, for Python, MATLAB or DuckDB",
  },
];

const FILE_FILTERS: Record<
  ExportFormat,
  { name: string; extensions: string[] }
> = {
  xlsx: { name: "Excel Workbook", extensions: ["xlsx"] },
  csv: { name: "CSV Table", extensions: ["csv"] },
  parquet: { name: "Parquet Table", extensions: ["parquet"] },
};

/**
 * Excel rows per series. Above this the workbook gets one averaged row per bucket
 * (the backend allows at most 20 000 buckets per series anyway).
 */
export const EXCEL_MAX_ROWS = 20_000;

/** What one graph contributes to an export. Registered by `BigGraph`. */
export type GraphExportSource = {
  machine: MachineIdentificationUnique | null;
  config: GraphConfig;
  series: SeriesData[];
  unit?: Unit;
  renderValue?: (value: number) => string;
  /** The time range the graph shows right now, or null before its first draw. */
  visibleRange: () => ViewRange | null;
};

export type ExportOptions = {
  formats: ExportFormat[];
  range: ViewRange;
  /** Bucket width of the CSV/Parquet export; 0 exports every raw sample. */
  bucketMs: number;
  groupId: string;
};

/** The range every graph currently shows (they are synced, so the first one is enough). */
export function currentExportRange(
  sources: GraphExportSource[],
): ViewRange | null {
  for (const source of sources) {
    const range = source.visibleRange();
    if (range && range.max > range.min) return range;
  }
  return null;
}

// --------------------------------------------------------------- columns

type Column = { name: string; label: string; scale: number };
type MachineColumns = {
  machine: MachineIdentificationUnique;
  measurements: Column[];
  targets: Column[];
};

function seriesLabel(source: GraphExportSource, series: SeriesData): string {
  const base =
    source.series.length > 1 && series.title
      ? `${source.config.title} – ${series.title}`
      : source.config.title;
  const unit = renderUnitSymbol(source.unit);
  return unit ? `${base} [${unit}]` : base;
}

function isTarget(line: GraphLine): line is GraphLine & { type: "target" } {
  return line.type === "target";
}

function machineKey(machine: MachineIdentificationUnique): string {
  const { vendor, machine: id } = machine.machine_identification;
  return `${vendor}-${id}-${machine.serial}`;
}

/** Database columns of all graphs, grouped by machine; each path appears once. */
export function exportColumns(sources: GraphExportSource[]): MachineColumns[] {
  const byMachine = new Map<string, MachineColumns>();
  for (const source of sources) {
    if (!source.machine) continue;
    const key = machineKey(source.machine);
    let entry = byMachine.get(key);
    if (!entry) {
      entry = { machine: source.machine, measurements: [], targets: [] };
      byMachine.set(key, entry);
    }
    for (const series of source.series) {
      const label = seriesLabel(source, series);
      if (
        series.history &&
        !entry.measurements.some((c) => c.name === series.history)
      ) {
        entry.measurements.push({
          name: series.history,
          label,
          scale: series.historyScale ?? 1,
        });
      }
      for (const line of series.lines ?? []) {
        if (
          !isTarget(line) ||
          !line.history ||
          entry.targets.some((c) => c.name === line.history)
        ) {
          continue;
        }
        entry.targets.push({
          name: line.history,
          label: line.label ? `${line.label}` : `${label} (target)`,
          scale: series.historyScale ?? 1,
        });
      }
    }
  }
  return [...byMachine.values()].filter((m) => m.measurements.length > 0);
}

// --------------------------------------------------------------- excel

function scaled(values: number[], scale: number): number[] {
  return scale === 1 ? values : values.map((v) => v * scale);
}

/** Live points of a series inside `range` (for series without database history). */
function livePoints(series: SeriesData, range: ViewRange): Points {
  const t: number[] = [];
  const v: number[] = [];
  if (!series.newData) return { t, v };
  const [ts, vs] = seriesToUPlotData(series.newData.long);
  ts.forEach((time, i) => {
    if (time >= range.min && time <= range.max) {
      t.push(time);
      v.push(vs[i]);
    }
  });
  return { t, v };
}

/**
 * Loads every series of `sources` for `range` from the database (full resolution up to
 * `EXCEL_MAX_ROWS` rows per series, averaged above) together with the markers.
 */
async function excelReport(
  sources: GraphExportSource[],
  range: ViewRange,
  title: string,
): Promise<ExcelReport> {
  const bucketMs = niceBucketMs(range.max - range.min, EXCEL_MAX_ROWS);
  const histories = new Map<string, HistoryResponse>();
  const markers: ExcelReport["markers"] = [];
  await Promise.all(
    exportColumns(sources).map(async (columns) => {
      const [history, machineMarkers] = await Promise.all([
        fetchHistory({
          machine: columns.machine,
          measurements: columns.measurements.map((c) => c.name),
          targets: columns.targets.map((c) => c.name),
          from: range.min,
          to: range.max,
          bucketMs,
        }),
        fetchMarkers(columns.machine).catch(() => []),
      ]);
      histories.set(machineKey(columns.machine), history);
      markers.push(...machineMarkers);
    }),
  );

  const series: ExcelSeries[] = [];
  for (const source of sources) {
    const history = source.machine
      ? histories.get(machineKey(source.machine))
      : undefined;
    const unit = renderUnitSymbol(source.unit) ?? "";

    for (const s of source.series) {
      const scale = s.historyScale ?? 1;
      const measured = s.history ? history?.measurements[s.history] : undefined;
      const points = measured
        ? { t: measured.t, v: scaled(measured.v, scale) }
        : livePoints(s, range);

      const line = (s.lines ?? []).find(isTarget);
      const steps = line?.history ? history?.targets[line.history] : undefined;
      const target = steps?.t.length
        ? { t: steps.t, v: scaled(steps.v, scale) }
        : line
          ? { t: [range.min], v: [line.value] }
          : undefined;

      series.push({
        graphTitle: source.config.title,
        title:
          source.series.length > 1 && s.title ? s.title : source.config.title,
        unit,
        points,
        target,
      });
    }
  }

  // The real resolution: buckets narrower than the sampling are raw samples.
  return { title, range, bucketMs, series, markers };
}

// --------------------------------------------------------------- run

const EXTENSION = /\.(xlsx|csv|parquet)$/i;

function slug(text: string): string {
  return text
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "_")
    .replace(/^_|_$/g, "");
}

/** `2026-11-23_14-05-09` in local time. */
function fileTimestamp(): string {
  const d = new Date();
  const pad = (n: number) => String(n).padStart(2, "0");
  return (
    `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}` +
    `_${pad(d.getHours())}-${pad(d.getMinutes())}-${pad(d.getSeconds())}`
  );
}

function timezone(): string {
  try {
    return Intl.DateTimeFormat().resolvedOptions().timeZone || "UTC";
  } catch {
    return "UTC";
  }
}

function bytesToBase64(bytes: Uint8Array): string {
  let binary = "";
  const chunkSize = 0x8000;
  for (let i = 0; i < bytes.length; i += chunkSize) {
    binary += String.fromCharCode(...bytes.subarray(i, i + chunkSize));
  }
  return btoa(binary);
}

async function writeExcel(
  sources: GraphExportSource[],
  options: ExportOptions,
  filePath: string,
): Promise<SaveFileResult> {
  try {
    const report = await excelReport(sources, options.range, options.groupId);
    const bytes = buildExcelWorkbook(report);
    return await writeFile({
      filePath,
      content: bytesToBase64(new Uint8Array(bytes)),
      encoding: "base64",
    });
  } catch (error) {
    return {
      success: false,
      error: `Excel: ${error instanceof Error ? error.message : String(error)}`,
    };
  }
}

async function writeTable(
  format: "csv" | "parquet",
  sources: GraphExportSource[],
  options: ExportOptions,
  basePath: string,
): Promise<SaveFileResult[]> {
  const machines = exportColumns(sources);
  if (machines.length === 0) {
    return [
      {
        success: false,
        error: `${format.toUpperCase()}: these graphs have no database history`,
      },
    ];
  }
  const results: SaveFileResult[] = [];
  for (const columns of machines) {
    // One file per machine; only pages showing several machines get a suffix.
    const suffix = machines.length > 1 ? `_${machineKey(columns.machine)}` : "";
    const result = await downloadToFile({
      filePath: `${basePath}${suffix}.${format}`,
      url: HISTORY_EXPORT_URL,
      body: {
        machine_identification_unique: columns.machine,
        measurements: columns.measurements,
        targets: columns.targets,
        from: Math.floor(options.range.min),
        to: Math.ceil(options.range.max),
        format,
        bucket_ms: options.bucketMs > 0 ? options.bucketMs : undefined,
        timezone: timezone(),
      },
    });
    results.push(
      result.success
        ? result
        : { ...result, error: `${format.toUpperCase()}: ${result.error}` },
    );
  }
  return results;
}

/**
 * Asks once for a file name, then writes every selected format next to it
 * (`name.xlsx`, `name.csv`, `name.parquet`). CSV and Parquet are streamed from the
 * database by the main process; Excel is built here.
 */
export async function runExport(
  sources: GraphExportSource[],
  options: ExportOptions,
): Promise<SaveFileResult> {
  if (options.formats.length === 0) {
    return { success: false, error: "No file type selected" };
  }
  const first = options.formats[0];
  const chosen = await choosePath({
    suggestedName: `${slug(options.groupId) || "graphs"}_export_${fileTimestamp()}.${first}`,
    filters: options.formats.map((f) => FILE_FILTERS[f]),
  });
  if (chosen.canceled) {
    return { success: false, error: "Export cancelled by user" };
  }
  const basePath = chosen.filePath.replace(EXTENSION, "");

  const results: SaveFileResult[] = [];
  for (const format of options.formats) {
    if (format === "xlsx") {
      results.push(await writeExcel(sources, options, `${basePath}.xlsx`));
    } else {
      results.push(...(await writeTable(format, sources, options, basePath)));
    }
  }

  const saved = results.filter((r) => r.success);
  const errors = results.filter((r) => !r.success).map((r) => r.error);
  const removable = saved.find((r) => r.isRemovable);
  return {
    success: errors.length === 0,
    error: errors.length ? errors.join("\n") : undefined,
    filePath: saved[0]?.filePath,
    filePaths: saved.map((r) => r.filePath!).filter(Boolean),
    isRemovable: removable !== undefined,
    mountPath: removable?.mountPath,
  };
}
