import React, { useCallback, useEffect, useState } from "react";
import { z } from "zod";
import { toast } from "sonner";
import { Page } from "@/components/layout/Page";
import { SectionTitle } from "@/components/layout/SectionTitle";
import { TouchButton } from "@/components/touch/TouchButton";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { DemoMachinesToggle } from "./DemoMachinesToggle";

const API = "http://localhost:3001/api/v1/database";
const DATABASE = "control_hub";
const RETENTION_DAYS = 30;

const storageSchema = z.object({
  tables: z.array(
    z.object({
      table: z.string(),
      rows: z.number(),
      bytes_on_disk: z.number(),
      compressed_bytes: z.number(),
      uncompressed_bytes: z.number(),
      parts: z.number(),
    }),
  ),
  days: z.array(
    z.object({
      day: z.string(),
      rows: z.number(),
      bytes_on_disk: z.number(),
    }),
  ),
  databases: z.array(
    z.object({ database: z.string(), bytes_on_disk: z.number() }),
  ),
  disks: z.array(
    z.object({
      name: z.string(),
      path: z.string(),
      free_space: z.number(),
      total_space: z.number(),
    }),
  ),
  pending_mutations: z.number(),
  projections: z.object({
    parts_total: z.number(),
    parts_ready: z.number(),
  }),
  demo: z.object({
    enabled: z.boolean(),
    backfill: z
      .object({
        state: z.enum(["pending", "running", "done", "failed"]),
        done: z.number(),
        total: z.number(),
        rows: z.number(),
        error: z.string().nullable(),
      })
      .nullable(),
  }),
});

type Storage = z.infer<typeof storageSchema>;

async function fetchStorage(): Promise<Storage> {
  const response = await fetch(`${API}/storage`);
  const json = await response.json();
  if (!response.ok) throw new Error(json?.error ?? `HTTP ${response.status}`);
  return storageSchema.parse(json);
}

async function clearDemoData(): Promise<void> {
  const response = await fetch(`${API}/demo/clear`, { method: "POST" });
  if (!response.ok) {
    const json = await response.json().catch(() => null);
    throw new Error(json?.error ?? `HTTP ${response.status}`);
  }
}

export function formatBytes(bytes: number): string {
  const units = ["B", "KB", "MB", "GB", "TB"];
  let value = bytes;
  let unit = 0;
  while (value >= 1000 && unit < units.length - 1) {
    value /= 1000;
    unit++;
  }
  return `${value.toFixed(unit === 0 || value >= 100 ? 0 : 1)} ${units[unit]}`;
}

function formatCount(n: number): string {
  return n.toLocaleString("en-US");
}

/** `20261127` → `27.11.` */
function formatDay(day: string): string {
  return `${day.slice(6, 8)}.${day.slice(4, 6)}.`;
}

/**
 * Average bytes per full day. The first and last partitions are usually partial (the
 * oldest one is being expired, today is still being written), so they are left out
 * when there are enough days.
 */
export function averageBytesPerDay(days: Storage["days"]): number | null {
  const full = days.length >= 3 ? days.slice(1, -1) : days;
  if (full.length === 0) return null;
  return full.reduce((sum, d) => sum + d.bytes_on_disk, 0) / full.length;
}

/** Polls the storage endpoint; faster while the demo backfill or a delete is running. */
function useStorage() {
  const [storage, setStorage] = useState<Storage | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  // bumped by `refresh` to reload right away
  const [reloads, setReloads] = useState(0);

  const busy =
    storage?.demo.backfill?.state === "running" ||
    (storage?.pending_mutations ?? 0) > 0 ||
    (storage !== null &&
      storage.projections.parts_ready < storage.projections.parts_total);

  useEffect(() => {
    let cancelled = false;
    const load = () =>
      fetchStorage()
        .then(
          (next) => {
            if (cancelled) return;
            setStorage(next);
            setError(null);
          },
          (e) => {
            if (cancelled) return;
            setError(e instanceof Error ? e.message : String(e));
          },
        )
        .finally(() => {
          if (!cancelled) setIsLoading(false);
        });
    void load();
    const id = setInterval(() => void load(), busy ? 3_000 : 30_000);
    return () => {
      cancelled = true;
      clearInterval(id);
    };
  }, [busy, reloads]);

  const refresh = useCallback(() => {
    setIsLoading(true);
    setReloads((n) => n + 1);
  }, []);

  return { storage, error, isLoading, refresh };
}

function Tile({
  label,
  value,
  detail,
}: {
  label: string;
  value: string;
  detail?: React.ReactNode;
}) {
  return (
    <div className="flex min-w-48 flex-1 flex-col gap-1 rounded-xl border p-4">
      <span className="text-muted-foreground text-sm">{label}</span>
      <span className="text-3xl font-semibold tabular-nums">{value}</span>
      {detail && (
        <span className="text-muted-foreground text-sm">{detail}</span>
      )}
    </div>
  );
}

function Bar({ share, className }: { share: number; className?: string }) {
  return (
    <div className="bg-muted h-3 w-full overflow-hidden rounded-full">
      <div
        className={`h-full rounded-full ${className ?? "bg-primary"}`}
        style={{ width: `${Math.min(100, Math.max(0, share * 100))}%` }}
      />
    </div>
  );
}

export function DatabasePage() {
  const { storage, error, isLoading, refresh } = useStorage();
  const [confirmClear, setConfirmClear] = useState(false);
  const [isClearing, setIsClearing] = useState(false);

  const clear = async () => {
    setIsClearing(true);
    try {
      await clearDemoData();
      toast.success("Deleting demo data, the space is freed in the background");
      void refresh();
    } catch (e) {
      toast.error(`Failed to delete demo data: ${e}`);
    } finally {
      setIsClearing(false);
      setConfirmClear(false);
    }
  };

  const refreshButton = (
    <TouchButton
      variant="outline"
      icon="lu:RefreshCw"
      isLoading={isLoading}
      onClick={refresh}
      className="px-4 py-2 text-sm"
    >
      Refresh
    </TouchButton>
  );

  if (!storage) {
    return (
      <Page>
        <SectionTitle title="Storage" right={refreshButton} />
        <p className="text-muted-foreground">
          {error
            ? `Could not read the database size: ${error}`
            : "Reading the database size…"}
        </p>
      </Page>
    );
  }

  const machineData =
    storage.databases.find((d) => d.database === DATABASE)?.bytes_on_disk ?? 0;
  const otherData = storage.databases
    .filter((d) => d.database !== DATABASE)
    .reduce((sum, d) => sum + d.bytes_on_disk, 0);
  const disk =
    storage.disks.find((d) => d.name === "default") ?? storage.disks[0];
  const perDay = averageBytesPerDay(storage.days);
  const maxDay = Math.max(1, ...storage.days.map((d) => d.bytes_on_disk));
  const backfill = storage.demo.backfill;

  return (
    <Page>
      <SectionTitle title="Storage" right={refreshButton}>
        {error && (
          <span className="text-sm text-red-600">Refresh failed: {error}</span>
        )}
      </SectionTitle>

      <div className="flex flex-wrap gap-4">
        <Tile
          label="Machine data"
          value={formatBytes(machineData)}
          detail={`${formatCount(
            storage.tables.reduce((sum, t) => sum + t.rows, 0),
          )} rows, kept ${RETENTION_DAYS} days`}
        />
        <Tile
          label="Per day"
          value={perDay === null ? "–" : formatBytes(perDay)}
          detail={
            perDay === null
              ? "No measurements yet"
              : `≈ ${formatBytes(perDay * RETENTION_DAYS)} at ${RETENTION_DAYS} days`
          }
        />
        <Tile
          label="Database logs"
          value={formatBytes(otherData)}
          detail="ClickHouse's own system tables"
        />
        {disk && (
          <Tile
            label="Disk free"
            value={formatBytes(disk.free_space)}
            detail={
              <span className="flex flex-col gap-2">
                <span>of {formatBytes(disk.total_space)}</span>
                <Bar
                  share={1 - disk.free_space / Math.max(1, disk.total_space)}
                  className={
                    disk.free_space / Math.max(1, disk.total_space) < 0.1
                      ? "bg-red-600"
                      : "bg-primary"
                  }
                />
              </span>
            }
          />
        )}
      </div>

      {storage.pending_mutations > 0 && (
        <p className="text-muted-foreground">
          Deleting data: {storage.pending_mutations} table
          {storage.pending_mutations === 1 ? "" : "s"} still being rewritten,
          the space is freed when they finish.
        </p>
      )}

      {storage.projections.parts_ready < storage.projections.parts_total && (
        <div className="flex max-w-3xl flex-col gap-2">
          <div className="flex justify-between text-sm">
            <span>
              Preparing fast graphs: long ranges (7 / 30 days) load slower until
              this is done
            </span>
            <span className="tabular-nums">
              {Math.floor(
                (storage.projections.parts_ready /
                  Math.max(1, storage.projections.parts_total)) *
                  100,
              )}{" "}
              %
            </span>
          </div>
          <Bar
            share={
              storage.projections.parts_ready /
              Math.max(1, storage.projections.parts_total)
            }
          />
        </div>
      )}

      <SectionTitle title="Tables" />
      <Table>
        <TableHeader>
          <TableRow>
            <TableHead>Table</TableHead>
            <TableHead className="text-right">Rows</TableHead>
            <TableHead className="text-right">On disk</TableHead>
            <TableHead className="text-right">Uncompressed</TableHead>
            <TableHead className="text-right">Compression</TableHead>
            <TableHead className="text-right">Parts</TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          {storage.tables.map((t) => (
            <TableRow key={t.table}>
              <TableCell>{t.table}</TableCell>
              <TableCell className="text-right tabular-nums">
                {formatCount(t.rows)}
              </TableCell>
              <TableCell className="text-right tabular-nums">
                {formatBytes(t.bytes_on_disk)}
              </TableCell>
              <TableCell className="text-right tabular-nums">
                {formatBytes(t.uncompressed_bytes)}
              </TableCell>
              <TableCell className="text-right tabular-nums">
                {t.compressed_bytes > 0
                  ? `${(t.uncompressed_bytes / t.compressed_bytes).toFixed(1)}×`
                  : "–"}
              </TableCell>
              <TableCell className="text-right tabular-nums">
                {t.parts}
              </TableCell>
            </TableRow>
          ))}
        </TableBody>
      </Table>

      {storage.days.length > 0 && (
        <>
          <SectionTitle title="Measurements per day" />
          <div className="flex flex-col gap-1.5">
            {storage.days.map((d) => (
              <div
                key={d.day}
                className="grid grid-cols-[4rem_1fr_5rem] items-center gap-3 text-sm"
              >
                <span className="tabular-nums">{formatDay(d.day)}</span>
                <Bar share={d.bytes_on_disk / maxDay} />
                <span className="text-right tabular-nums">
                  {formatBytes(d.bytes_on_disk)}
                </span>
              </div>
            ))}
          </div>
        </>
      )}

      <SectionTitle title="Demo machines" right={<DemoMachinesToggle />} />
      <p className="text-muted-foreground max-w-3xl">
        Adds a simulated production line (Extruder, Aquapath, Laser, Winder)
        that writes the same data as real machines, and fills the last{" "}
        {RETENTION_DAYS} days of history, to test the graphs and the database
        with a full load.
      </p>
      {backfill && (
        <div className="flex max-w-3xl flex-col gap-2">
          <div className="flex justify-between text-sm">
            <span>
              {backfill.state === "running" && "Writing 30 days of history…"}
              {backfill.state === "pending" && "History waiting to start"}
              {backfill.state === "done" && "History complete"}
              {backfill.state === "failed" &&
                `History failed: ${backfill.error ?? "unknown error"}`}
            </span>
            <span className="tabular-nums">
              {formatCount(backfill.rows)} rows
              {backfill.total > 0 &&
                ` · ${Math.floor((backfill.done / backfill.total) * 100)} %`}
            </span>
          </div>
          <Bar
            share={backfill.total > 0 ? backfill.done / backfill.total : 0}
            className={
              backfill.state === "failed" ? "bg-red-600" : "bg-green-600"
            }
          />
        </div>
      )}
      {!storage.demo.enabled && (
        <div>
          <TouchButton
            variant="outline"
            icon="lu:Trash2"
            onClick={() => setConfirmClear(true)}
          >
            Delete demo data
          </TouchButton>
        </div>
      )}

      <Dialog open={confirmClear} onOpenChange={setConfirmClear}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Delete demo data?</DialogTitle>
            <DialogDescription className="text-base">
              Deletes every measurement, event, setting and marker of the demo
              machines. Data of real machines is not touched. ClickHouse frees
              the space in the background, which can take a few minutes.
            </DialogDescription>
          </DialogHeader>
          <DialogFooter>
            <TouchButton
              variant="outline"
              onClick={() => setConfirmClear(false)}
            >
              Cancel
            </TouchButton>
            <TouchButton
              icon="lu:Trash2"
              variant="destructive"
              isLoading={isClearing}
              onClick={() => void clear()}
            >
              Delete
            </TouchButton>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </Page>
  );
}
