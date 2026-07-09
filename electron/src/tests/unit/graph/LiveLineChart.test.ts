import { afterEach, expect, test } from "vitest";
import {
  LiveLineChart,
  type LiveLineChartConfig,
} from "@/components/graph/LiveLineChart";
import { createTimeSeries, TimeSeries } from "@/lib/timeseries";
import type { GraphLine } from "@/components/graph/types";

function buildSeries(): TimeSeries {
  const { initialTimeSeries, insert } = createTimeSeries({
    sampleIntervalLong: 1000,
    retentionDurationLong: 60 * 60 * 1000,
  });
  let series = initialTimeSeries;
  series = insert(series, { timestamp: 0, value: 10 });
  series = insert(series, { timestamp: 1000, value: 12 });
  series = insert(series, { timestamp: 2000, value: 11 });
  return series;
}

function buildConfig(overrides: Partial<LiveLineChartConfig> = {}): LiveLineChartConfig {
  return {
    newData: { newData: buildSeries(), title: "Temp" },
    config: { title: "Temperature" },
    colors: { primary: "#000", grid: "#ccc", axis: "#666", background: "#fff" },
    selectedTimeWindow: "all",
    visibleSeries: [true],
    ...overrides,
  };
}

let chart: LiveLineChart | null = null;

afterEach(() => {
  chart?.destroy();
  chart = null;
});

test("constructs against a canvas without throwing and exposes the raw Chart.js instance", () => {
  const canvas = document.createElement("canvas");
  chart = new LiveLineChart(canvas, buildConfig());

  expect(chart.chartInstance).toBeDefined();
  expect(chart.chartInstance.data.datasets).toHaveLength(1);
});

test("getSnapshot starts with null cursor values sized to the series count", () => {
  const canvas = document.createElement("canvas");
  chart = new LiveLineChart(canvas, buildConfig());

  expect(chart.getSnapshot()).toEqual({
    cursorValue: null,
    cursorValues: [null],
  });
});

test("updateData rebuilds datasets for multi-series data without throwing", () => {
  const canvas = document.createElement("canvas");
  const config = buildConfig({
    newData: [
      { newData: buildSeries(), title: "A", color: "#111" },
      { newData: buildSeries(), title: "B", color: "#222" },
    ],
    visibleSeries: [true, true],
  });
  chart = new LiveLineChart(canvas, config);

  expect(() =>
    chart!.updateData({ ...config, visibleSeries: [true, false] }),
  ).not.toThrow();
  expect(chart.chartInstance.data.datasets[0].hidden).toBe(false);
  expect(chart.chartInstance.data.datasets[1].hidden).toBe(true);
});

test("updateData renders a constant threshold line as an annotation", () => {
  const canvas = document.createElement("canvas");
  const lines: GraphLine[] = [
    { type: "threshold", value: 50, color: "#ff0000", label: "Max" },
  ];
  const config = buildConfig({ config: { title: "Temperature", lines } });
  chart = new LiveLineChart(canvas, config);

  const annotations = chart.chartInstance.options.plugins!.annotation!
    .annotations as Record<string, unknown>;
  expect(Object.keys(annotations)).toContain("line-0");
});

test("updateData renders a target line with a targetSeries as an extra dataset, not an annotation", () => {
  const canvas = document.createElement("canvas");
  const lines: GraphLine[] = [
    {
      type: "target",
      value: 10,
      color: "#00ff00",
      targetSeries: buildSeries(),
    },
  ];
  const config = buildConfig({ config: { title: "Temperature", lines } });
  chart = new LiveLineChart(canvas, config);

  // main series dataset + the target-series-tracking dataset
  expect(chart.chartInstance.data.datasets).toHaveLength(2);
  const annotations = chart.chartInstance.options.plugins!.annotation!
    .annotations as Record<string, unknown>;
  expect(Object.keys(annotations)).not.toContain("line-0");
});

test("destroy tears down the underlying Chart.js instance without throwing", () => {
  const canvas = document.createElement("canvas");
  chart = new LiveLineChart(canvas, buildConfig());

  expect(() => chart!.destroy()).not.toThrow();
});
