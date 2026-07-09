import { fireEvent, render } from "@testing-library/react";
import { afterEach, expect, test, vi } from "vitest";
import React from "react";
import { BigGraph } from "@/components/graph/BigGraph";
import { createTimeSeries, TimeSeries } from "@/lib/timeseries";

function buildSeries(value = 10): TimeSeries {
  const { initialTimeSeries, insert } = createTimeSeries({
    sampleIntervalLong: 1000,
    retentionDurationLong: 60 * 60 * 1000,
  });
  let series = initialTimeSeries;
  series = insert(series, { timestamp: 0, value });
  series = insert(series, { timestamp: 1000, value: value + 1 });
  return series;
}

afterEach(() => {
  vi.restoreAllMocks();
});

test("renders a canvas and the single-series value badge", () => {
  const { container, getByText } = render(
    <BigGraph
      newData={{ newData: buildSeries(), title: "Temp" }}
      config={{ title: "Temperature" }}
      graphId="temp-graph"
    />,
  );

  expect(container.querySelector("canvas")).toBeInTheDocument();
  expect(getByText("Temperature")).toBeInTheDocument();
});

test("renders one pill per series for multi-series data and toggling doesn't throw", () => {
  const { getAllByRole } = render(
    <BigGraph
      newData={[
        { newData: buildSeries(1), title: "A", color: "#111111" },
        { newData: buildSeries(2), title: "B", color: "#222222" },
      ]}
      config={{ title: "Two Series" }}
      graphId="two-series-graph"
    />,
  );

  const buttons = getAllByRole("button");
  expect(buttons).toHaveLength(2);
  expect(() => fireEvent.click(buttons[0])).not.toThrow();
});

test("does not crash when newData or the selected time window updates after mount", () => {
  const { rerender } = render(
    <BigGraph
      newData={{ newData: buildSeries(), title: "Temp" }}
      config={{ title: "Temperature" }}
      graphId="temp-graph"
    />,
  );

  expect(() =>
    rerender(
      <BigGraph
        newData={{ newData: buildSeries(5), title: "Temp" }}
        config={{ title: "Temperature" }}
        graphId="temp-graph"
        syncGraph={{
          timeWindow: 60 * 1000,
          viewMode: "default",
          isLiveMode: true,
          historicalSwitchOrigin: "button",
        }}
      />,
    ),
  ).not.toThrow();
});
