import { render } from "@testing-library/react";
import { afterEach, expect, test, vi } from "vitest";
import React from "react";
import { MiniGraph } from "@/components/graph/MiniGraph";
import { createTimeSeries, TimeSeries } from "@/lib/timeseries";

function buildSeries(): TimeSeries {
  const { initialTimeSeries, insert } = createTimeSeries({
    sampleIntervalShort: 20,
    retentionDurationShort: 5000,
  });
  let series = initialTimeSeries;
  series = insert(series, { timestamp: 0, value: 1 });
  series = insert(series, { timestamp: 20, value: 2 });
  return series;
}

afterEach(() => {
  vi.restoreAllMocks();
});

test("renders a canvas without crashing when there is no data yet", () => {
  const { container } = render(<MiniGraph newData={null} width={200} />);

  expect(container.querySelector("canvas")).toBeInTheDocument();
});

test("renders and mounts a chart once real data is available", () => {
  const { container } = render(
    <MiniGraph newData={buildSeries()} width={200} />,
  );

  expect(container.querySelector("canvas")).toBeInTheDocument();
});

test("does not crash when width or data updates after mount", () => {
  const series = buildSeries();
  const { rerender } = render(<MiniGraph newData={series} width={200} />);

  const { insert } = createTimeSeries();
  const updated = insert(series, { timestamp: 40, value: 3 });

  expect(() =>
    rerender(<MiniGraph newData={updated} width={300} />),
  ).not.toThrow();
});
