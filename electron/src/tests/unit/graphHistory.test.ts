import { afterEach, describe, expect, it, vi } from "vitest";
import { HistoryLoader } from "@/components/graph/history/HistoryLoader";
import { collectTail } from "@/components/graph/chartjs/GraphEngine";
import { buildTimeTicks } from "@/components/graph/chartjs/timeAxis";
import { niceBucketMs } from "@/lib/history";
import { createTimeSeries } from "@/lib/timeseries";

const machine = {
  machine_identification: { vendor: 1, machine: 4 },
  serial: 1,
};

function mockFetch(responses: unknown[]) {
  const calls: Record<string, unknown>[] = [];
  const fn = vi.fn(async (_url: string, init: RequestInit) => {
    calls.push(JSON.parse(String(init.body)));
    const body = responses.shift();
    return { ok: true, json: async () => body } as Response;
  });
  vi.stubGlobal("fetch", fn);
  return calls;
}

const flush = () => new Promise((r) => setTimeout(r, 0));

afterEach(() => vi.unstubAllGlobals());

describe("niceBucketMs", () => {
  it("picks the smallest nice bucket that fits", () => {
    expect(niceBucketMs(60 * 60 * 1000, 1500)).toBe(5_000); // 1 h over 1500 px → 2.4 s → 5 s
    expect(niceBucketMs(10_000, 1500)).toBe(10);
    expect(niceBucketMs(30 * 24 * 3600 * 1000, 1500)).toBe(1_800_000);
  });
});

describe("HistoryLoader", () => {
  it("loads the padded view once, then appends only the newest buckets in live mode", async () => {
    const now = 1_000_000_000;
    const calls = mockFetch([
      {
        bucket_ms: 50,
        measurements: {
          a: { t: [now - 60_000, now - 6_000, now - 5_000], v: [1, 2, 3] },
        },
        targets: { tgt: { t: [now - 70_000], v: [10] } },
      },
      {
        bucket_ms: 50,
        measurements: { a: { t: [now - 4_000, now + 900], v: [4, 5] } },
        targets: { tgt: { t: [now - 4_000, now + 500], v: [10, 20] } },
      },
    ]);
    const loader = new HistoryLoader(
      machine,
      { measurements: ["a"], targets: ["tgt"] },
      () => {},
    );
    const view = { min: now - 60_000, max: now };

    loader.ensure(view, true, 1500, now);
    await flush();
    expect(calls).toHaveLength(1);
    expect(calls[0].from).toBe(now - 90_000);
    expect(calls[0].to).toBe(now);
    expect(calls[0].bucket_ms).toBe(50);

    // Same view shortly after: nothing to do.
    loader.ensure(view, true, 1500, now + 100);
    await flush();
    expect(calls).toHaveLength(1);

    // After the refresh interval: an incremental load overlapping the DB lag.
    loader.ensure(view, true, 1500, now + 1_000);
    await flush();
    expect(calls).toHaveLength(2);
    expect(calls[1].from).toBe(now - 5_000);
    expect(loader.measurement("a")).toEqual({
      t: [now - 60_000, now - 6_000, now - 4_000, now + 900],
      v: [1, 2, 4, 5],
    });
    // Steps keep the earlier value and drop the repeated one.
    expect(loader.target("tgt")).toEqual({
      t: [now - 70_000, now + 500],
      v: [10, 20],
    });
  });

  it("reloads when the zoom level changes the bucket width", async () => {
    const now = 2_000_000_000;
    const empty = {
      bucket_ms: 0,
      measurements: { a: { t: [], v: [] } },
      targets: {},
    };
    const calls = mockFetch([
      { ...empty, bucket_ms: 50 },
      { ...empty, bucket_ms: 5_000 },
    ]);
    const loader = new HistoryLoader(
      machine,
      { measurements: ["a"], targets: [] },
      () => {},
    );

    loader.ensure({ min: now - 60_000, max: now }, false, 1500, now);
    await flush();
    loader.ensure({ min: now - 3_600_000, max: now }, false, 1500, now + 500);
    await flush();
    expect(calls).toHaveLength(2);
    expect(calls[1].bucket_ms).toBe(5_000);
  });

  it("backs off after an error", async () => {
    vi.stubGlobal(
      "fetch",
      vi.fn(async () => ({
        ok: false,
        status: 500,
        json: async () => ({ error: "db down" }),
      })),
    );
    const loader = new HistoryLoader(
      machine,
      { measurements: ["a"], targets: [] },
      () => {},
    );
    loader.ensure({ min: 0, max: 60_000 }, false, 1500, 60_000);
    await flush();
    expect(loader.error).toBe("db down");
    loader.ensure({ min: 0, max: 60_000 }, false, 1500, 60_500);
    expect(fetch).toHaveBeenCalledTimes(1);
  });
});

describe("collectTail", () => {
  it("returns buffered points newer than the cutoff in order, preferring the 20 ms buffer", () => {
    const { initialTimeSeries, insert } = createTimeSeries();
    let ts = initialTimeSeries;
    for (let t = 1_000; t <= 20_000; t += 100)
      ts = insert(ts, { timestamp: t, value: t });

    const tail = collectTail(ts, 12_000);
    const xs = tail.map((p) => p.x);
    expect(xs[0]).toBeGreaterThan(12_000);
    expect(xs[xs.length - 1]).toBe(20_000);
    expect([...xs].sort((a, b) => a - b)).toEqual(xs);
    expect(new Set(xs).size).toBe(xs.length);
    // The last 5 s come from the short buffer at full rate.
    expect(xs.filter((x) => x > 15_000)).toHaveLength(50);
  });
});

describe("buildTimeTicks", () => {
  it("places ticks on round steps within the range", () => {
    const min = Date.UTC(2026, 0, 1, 10, 0, 7);
    const { ticks, step } = buildTimeTicks(min, min + 10 * 60_000, 10);
    expect(step).toBe(60_000);
    expect(ticks.every((t) => t.value % 60_000 === 0)).toBe(true);
    expect(ticks[0].value).toBeGreaterThanOrEqual(min);
  });
});
