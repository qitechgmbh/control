import React, { useState } from "react";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { TouchButton } from "@/components/touch/TouchButton";
import { Icon, type IconName } from "@/components/icons/Icon";
import { HISTORY_RETENTION_MS } from "@/lib/history";
import { EXCEL_MAX_ROWS, EXPORT_FORMATS, type ExportFormat } from "../export";
import type { ViewRange } from "../types";
import type { GraphSync } from "../useGraphSync";

const FORMAT_ICONS: Record<ExportFormat, IconName> = {
  xlsx: "lu:FileSpreadsheet",
  csv: "lu:FileText",
  parquet: "lu:Database",
};

const RESOLUTIONS: { value: number; label: string }[] = [
  { value: 0, label: "Raw" },
  { value: 1_000, label: "1 s" },
  { value: 10_000, label: "10 s" },
  { value: 60_000, label: "1 min" },
];

const HOUR = 60 * 60 * 1000;

const RANGE_PRESETS: { label: string; span: number | null }[] = [
  { label: "Current view", span: null },
  { label: "Last hour", span: HOUR },
  { label: "Last 8 h", span: 8 * HOUR },
  { label: "Last 24 h", span: 24 * HOUR },
  { label: "Last 7 days", span: 7 * 24 * HOUR },
];

/** `datetime-local` value (local time, seconds precision) of a Unix ms timestamp. */
function toLocalInput(ms: number): string {
  const d = new Date(ms);
  const pad = (n: number) => String(n).padStart(2, "0");
  return (
    `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}` +
    `T${pad(d.getHours())}:${pad(d.getMinutes())}:${pad(d.getSeconds())}`
  );
}

function fromLocalInput(value: string): number | null {
  const ms = new Date(value).getTime();
  return Number.isFinite(ms) ? ms : null;
}

function formatSpan(ms: number): string {
  const minutes = Math.round(ms / 60_000);
  if (minutes < 60) return `${Math.max(1, minutes)} min`;
  const hours = Math.floor(minutes / 60);
  if (hours < 48)
    return minutes % 60 ? `${hours} h ${minutes % 60} min` : `${hours} h`;
  return `${Math.round(hours / 24)} days`;
}

/** The range of a preset: the last `span` ms, or what the graphs show for `null`. */
function rangeFor(syncHook: GraphSync, span: number | null): ViewRange {
  const now = Date.now();
  if (span !== null) return { min: now - span, max: now };
  return syncHook.exportRange() ?? { min: now - HOUR, max: now };
}

function oldestStored(): number {
  return Date.now() - HISTORY_RETENTION_MS;
}

/**
 * The graph page's "Export" button: opens a dialog to pick one or more file types and
 * the time range (default: what the graphs show), then writes the files.
 */
export function ExportButton({ syncHook }: { syncHook: GraphSync }) {
  const [open, setOpen] = useState(false);
  const [formats, setFormats] = useState<ExportFormat[]>(["xlsx"]);
  const [bucketMs, setBucketMs] = useState(0);
  const [range, setRange] = useState<ViewRange | null>(null);
  const [preset, setPreset] = useState<string | null>("Current view");
  const [isExporting, setIsExporting] = useState(false);
  /** Oldest time the database still holds, taken when the dialog opens. */
  const [oldest, setOldest] = useState(0);

  const openDialog = () => {
    setRange(rangeFor(syncHook, null));
    setPreset("Current view");
    setOldest(oldestStored());
    setOpen(true);
  };

  const applyPreset = (label: string, span: number | null) => {
    setRange(rangeFor(syncHook, span));
    setPreset(label);
  };

  const setBound = (bound: "min" | "max", value: string) => {
    const ms = fromLocalInput(value);
    if (ms === null || !range) return;
    setRange({ ...range, [bound]: ms });
    setPreset(null);
  };

  const toggleFormat = (format: ExportFormat) =>
    setFormats((prev) =>
      prev.includes(format)
        ? prev.filter((f) => f !== format)
        : // keep the order of EXPORT_FORMATS
          EXPORT_FORMATS.map((f) => f.value).filter(
            (f) => f === format || prev.includes(f),
          ),
    );

  const rangeError = !range
    ? null
    : range.max <= range.min
      ? "The end must be after the start."
      : range.max < oldest
        ? "The database only keeps the last 30 days."
        : null;
  const hasTable = formats.includes("csv") || formats.includes("parquet");
  const canExport = formats.length > 0 && range !== null && !rangeError;

  const handleExport = async () => {
    if (!canExport || !range) return;
    setIsExporting(true);
    try {
      const result = await syncHook.runExport({ formats, range, bucketMs });
      if (result.success) setOpen(false);
    } finally {
      setIsExporting(false);
    }
  };

  return (
    <>
      <TouchButton
        onClick={openDialog}
        variant="outline"
        className="h-auto bg-green-600 px-3 py-3 text-base font-medium text-white hover:bg-green-700"
      >
        Export
      </TouchButton>

      <Dialog
        open={open}
        onOpenChange={(next) => !isExporting && setOpen(next)}
      >
        <DialogContent className="sm:max-w-2xl">
          <DialogHeader>
            <DialogTitle>Export</DialogTitle>
            <DialogDescription>
              Data is read from the machine database for the chosen range.
            </DialogDescription>
          </DialogHeader>

          <section className="flex flex-col gap-2">
            <h3 className="text-sm font-medium text-gray-700">File types</h3>
            <div className="grid grid-cols-3 gap-2">
              {EXPORT_FORMATS.map((format) => {
                const selected = formats.includes(format.value);
                return (
                  <button
                    key={format.value}
                    type="button"
                    onClick={() => toggleFormat(format.value)}
                    aria-pressed={selected}
                    className={`flex min-h-[96px] flex-col items-start gap-1 rounded-lg border-2 p-3 text-left transition-colors ${
                      selected
                        ? "border-black bg-gray-50"
                        : "border-gray-200 bg-white hover:bg-gray-50"
                    }`}
                  >
                    <span className="flex w-full items-center gap-2 text-base font-medium">
                      <Icon
                        name={FORMAT_ICONS[format.value]}
                        className="size-5"
                      />
                      {format.label}
                      <Icon
                        name={selected ? "lu:SquareCheck" : "lu:Square"}
                        className="ml-auto size-5"
                      />
                    </span>
                    <span className="text-sm text-gray-500">
                      {format.description}
                    </span>
                  </button>
                );
              })}
            </div>
          </section>

          <section className="flex flex-col gap-2">
            <h3 className="text-sm font-medium text-gray-700">Time range</h3>
            <div className="flex flex-wrap gap-2">
              {RANGE_PRESETS.map((p) => (
                <TouchButton
                  key={p.label}
                  variant="outline"
                  onClick={() => applyPreset(p.label, p.span)}
                  className={`h-auto px-3 py-2 text-sm ${
                    preset === p.label
                      ? "bg-black text-white hover:bg-black/90 hover:text-white"
                      : "border-gray-300 bg-white text-gray-700"
                  }`}
                >
                  {p.label}
                </TouchButton>
              ))}
            </div>
            {range && (
              <div className="grid grid-cols-2 gap-2">
                <label className="flex flex-col gap-1 text-sm text-gray-600">
                  From
                  <input
                    type="datetime-local"
                    step={1}
                    value={toLocalInput(range.min)}
                    onChange={(e) => setBound("min", e.target.value)}
                    className="rounded-md border border-gray-300 px-3 py-3 text-base text-gray-900"
                  />
                </label>
                <label className="flex flex-col gap-1 text-sm text-gray-600">
                  To
                  <input
                    type="datetime-local"
                    step={1}
                    value={toLocalInput(range.max)}
                    onChange={(e) => setBound("max", e.target.value)}
                    className="rounded-md border border-gray-300 px-3 py-3 text-base text-gray-900"
                  />
                </label>
              </div>
            )}
            <p
              className={`text-sm ${rangeError ? "text-red-600" : "text-gray-500"}`}
            >
              {rangeError ??
                (range ? `${formatSpan(range.max - range.min)} of data` : "")}
            </p>
          </section>

          {hasTable && (
            <section className="flex flex-col gap-2">
              <h3 className="text-sm font-medium text-gray-700">
                Resolution (CSV, Parquet)
              </h3>
              <div className="flex flex-wrap gap-2">
                {RESOLUTIONS.map((r) => (
                  <TouchButton
                    key={r.value}
                    variant="outline"
                    onClick={() => setBucketMs(r.value)}
                    className={`h-auto px-3 py-2 text-sm ${
                      bucketMs === r.value
                        ? "bg-black text-white hover:bg-black/90 hover:text-white"
                        : "border-gray-300 bg-white text-gray-700"
                    }`}
                  >
                    {r.label}
                  </TouchButton>
                ))}
              </div>
              <p className="text-sm text-gray-500">
                {bucketMs === 0
                  ? "Every recorded sample."
                  : "One row per interval with mean, min and max of each value."}
              </p>
            </section>
          )}

          {formats.includes("xlsx") && (
            <p className="text-sm text-gray-500">
              Excel holds up to {EXCEL_MAX_ROWS.toLocaleString()} rows per
              value; longer ranges are averaged to fit.
            </p>
          )}

          <DialogFooter>
            <TouchButton
              variant="outline"
              onClick={() => setOpen(false)}
              disabled={isExporting}
            >
              Cancel
            </TouchButton>
            <TouchButton
              icon="lu:Download"
              onClick={handleExport}
              isLoading={isExporting}
              disabled={!canExport}
              className="bg-green-600 text-white hover:bg-green-700"
            >
              {isExporting ? "Exporting…" : "Export"}
            </TouchButton>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </>
  );
}
