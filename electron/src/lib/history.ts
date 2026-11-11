import { MachineIdentificationUnique } from "@/machines/types";

/** ClickHouse keeps measurements for 30 days (TTL in the migrations). */
export const HISTORY_RETENTION_MS = 30 * 24 * 60 * 60 * 1000;

const HISTORY_URL = "http://localhost:3001/api/v1/machine/history";

/** Columnar series: `t[i]` (Unix ms) belongs to `v[i]`. */
export type HistorySeries = { t: number[]; v: number[] };

export type HistoryResponse = {
  bucket_ms: number;
  measurements: Record<string, HistorySeries>;
  targets: Record<string, HistorySeries>;
};

export type HistoryRequest = {
  machine: MachineIdentificationUnique;
  measurements: string[];
  targets: string[];
  from: number;
  to: number;
  bucketMs: number;
};

export async function fetchHistory(
  req: HistoryRequest,
  signal?: AbortSignal,
): Promise<HistoryResponse> {
  const response = await fetch(HISTORY_URL, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({
      machine_identification_unique: req.machine,
      measurements: req.measurements,
      targets: req.targets,
      from: Math.floor(req.from),
      to: Math.ceil(req.to),
      bucket_ms: req.bucketMs,
    }),
    signal,
  });
  if (!response.ok) {
    let message = `HTTP ${response.status}`;
    try {
      const body = await response.json();
      if (body?.error) message = body.error;
    } catch {
      // keep the status message
    }
    throw new Error(message);
  }
  return response.json();
}

const NICE_BUCKETS_MS = [
  10, 20, 50, 100, 200, 500, 1_000, 2_000, 5_000, 10_000, 15_000, 30_000,
  60_000, 120_000, 300_000, 600_000, 900_000, 1_800_000, 3_600_000, 7_200_000,
  10_800_000, 21_600_000, 43_200_000, 86_400_000,
];

/**
 * Smallest "nice" bucket width so that `span` fits into at most `maxBuckets` buckets.
 * Nice widths keep the bucket stable while the view pans, so cached data stays valid.
 */
export function niceBucketMs(span: number, maxBuckets: number): number {
  const raw = span / Math.max(1, maxBuckets);
  for (const bucket of NICE_BUCKETS_MS) {
    if (bucket >= raw) return bucket;
  }
  return NICE_BUCKETS_MS[NICE_BUCKETS_MS.length - 1];
}
