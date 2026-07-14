import { afterEach, expect, test, vi } from "vitest";
import {
  SparklineChart,
  SPARKLINE_HEIGHT,
} from "@/components/graph/SparklineChart";
import { createTimeSeries, TimeSeries } from "@/lib/timeseries";

function buildSeries(): TimeSeries {
  const { initialTimeSeries, insert } = createTimeSeries({
    sampleIntervalShort: 20,
    retentionDurationShort: 5000,
  });
  let series = initialTimeSeries;
  series = insert(series, { timestamp: 0, value: 1 });
  series = insert(series, { timestamp: 20, value: 2 });
  series = insert(series, { timestamp: 40, value: 3 });
  return series;
}

let chart: SparklineChart | null = null;

afterEach(() => {
  chart?.destroy();
  chart = null;
  vi.restoreAllMocks();
});

test("constructs against a canvas without throwing and sizes it to the given width", () => {
  const canvas = document.createElement("canvas");
  const getContextSpy = vi.spyOn(canvas, "getContext");

  chart = new SparklineChart(canvas, buildSeries(), { width: 200 });

  expect(getContextSpy).toHaveBeenCalled();
  expect(chart).toBeInstanceOf(SparklineChart);
  void SPARKLINE_HEIGHT;
});

test("pushLatest applies the update on the next animation frame without throwing", async () => {
  const canvas = document.createElement("canvas");
  const series = buildSeries();
  chart = new SparklineChart(canvas, series, { width: 200 });

  const { insert } = createTimeSeries();
  const advanced = insert(series, { timestamp: 60, value: 4 });

  expect(() => chart!.pushLatest(advanced)).not.toThrow();

  await new Promise((resolve) => requestAnimationFrame(resolve));
});

test("pushLatest coalesces multiple calls within the same frame into one update", async () => {
  const canvas = document.createElement("canvas");
  const series = buildSeries();
  chart = new SparklineChart(canvas, series, { width: 200 });

  const { insert } = createTimeSeries();
  const rafSpy = vi.spyOn(window, "requestAnimationFrame");

  chart.pushLatest(insert(series, { timestamp: 60, value: 4 }));
  chart.pushLatest(insert(series, { timestamp: 80, value: 5 }));
  chart.pushLatest(insert(series, { timestamp: 100, value: 6 }));

  expect(rafSpy).toHaveBeenCalledTimes(1);

  await new Promise((resolve) => requestAnimationFrame(resolve));
});

test("resize, setRange, and destroy do not throw", () => {
  const canvas = document.createElement("canvas");
  chart = new SparklineChart(canvas, buildSeries(), { width: 200 });

  expect(() => chart!.resize(300)).not.toThrow();
  expect(() => chart!.setRange({ min: 0, max: 100 })).not.toThrow();
  expect(() => chart!.destroy()).not.toThrow();

  // pushLatest after destroy is a no-op, not a crash.
  expect(() => chart!.pushLatest(buildSeries())).not.toThrow();
});
