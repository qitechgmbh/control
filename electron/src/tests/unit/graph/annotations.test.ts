import { expect, test } from "vitest";
import {
  buildAnnotations,
  buildLineAnnotations,
  buildMarkerAnnotations,
  buildTargetSeriesDatasets,
} from "@/components/graph/annotations";
import type { GraphLine } from "@/components/graph/types";
import type { Marker } from "@/stores/markerStore";
import { createTimeSeries } from "@/lib/timeseries";

test("buildLineAnnotations maps a threshold line to a horizontal y-scale annotation with the default dash", () => {
  const lines: GraphLine[] = [
    { type: "threshold", value: 42, color: "#ff0000", label: "Max" },
  ];

  const annotations = buildLineAnnotations(lines);

  expect(annotations["line-0"]).toMatchObject({
    type: "line",
    scaleID: "y",
    value: 42,
    borderColor: "#ff0000",
    borderDash: [5, 5],
    label: { display: true, content: "Max", position: "end" },
  });
});

test("buildLineAnnotations skips lines with show:false", () => {
  const lines: GraphLine[] = [
    { type: "threshold", value: 42, color: "#ff0000", show: false },
  ];

  expect(buildLineAnnotations(lines)).toEqual({});
});

test("buildLineAnnotations includes a target line with no targetSeries as a constant y-scale line", () => {
  const lines: GraphLine[] = [
    { type: "target", value: 10, color: "#00ff00" },
  ];

  const annotations = buildLineAnnotations(lines);

  expect(annotations["line-0"]).toMatchObject({
    type: "line",
    scaleID: "y",
    value: 10,
  });
});

test("buildLineAnnotations excludes a target line that has a targetSeries (rendered as a dataset instead)", () => {
  const { initialTimeSeries } = createTimeSeries();
  const lines: GraphLine[] = [
    {
      type: "target",
      value: 10,
      color: "#00ff00",
      targetSeries: initialTimeSeries,
    },
  ];

  expect(buildLineAnnotations(lines)).toEqual({});
});

test("buildLineAnnotations maps a user_marker line to a vertical x-scale annotation", () => {
  const lines: GraphLine[] = [
    {
      type: "user_marker",
      value: 0,
      color: "#0000ff",
      label: "Batch start",
      markerTimestamp: 123456,
    },
  ];

  const annotations = buildLineAnnotations(lines);

  expect(annotations["line-0"]).toMatchObject({
    type: "line",
    scaleID: "x",
    value: 123456,
    label: { display: true, content: "Batch start", position: "start" },
  });
});

test("buildMarkerAnnotations maps stored markers to vertical x-scale annotations, defaulting color", () => {
  const markers: Marker[] = [
    { timestamp: 1000, name: "Spool change" },
    { timestamp: 2000, name: "Jam", color: "#ff00ff" },
  ];

  const annotations = buildMarkerAnnotations(markers);

  expect(annotations["marker-0"]).toMatchObject({
    scaleID: "x",
    value: 1000,
    borderColor: "#000000",
    label: { content: "Spool change" },
  });
  expect(annotations["marker-1"]).toMatchObject({
    value: 2000,
    borderColor: "#ff00ff",
  });
});

test("buildAnnotations merges lines and markers into one map", () => {
  const lines: GraphLine[] = [
    { type: "threshold", value: 1, color: "#111111" },
  ];
  const markers: Marker[] = [{ timestamp: 5, name: "M" }];

  const annotations = buildAnnotations(lines, markers);

  expect(Object.keys(annotations)).toEqual(["line-0", "marker-0"]);
});

test("buildTargetSeriesDatasets aligns a targetSeries to the given timestamps as a stepped dataset", () => {
  const { initialTimeSeries, insert } = createTimeSeries();
  let targetSeries = insert(initialTimeSeries, { timestamp: 0, value: 5 });
  targetSeries = insert(targetSeries, { timestamp: 2000, value: 9 });

  const lines: GraphLine[] = [
    {
      type: "target",
      value: 5,
      color: "#123456",
      label: "Target RPM",
      targetSeries,
    },
  ];
  const timestamps = [0, 1000, 2000, 3000];

  const datasets = buildTargetSeriesDatasets(lines, timestamps);

  expect(datasets).toHaveLength(1);
  expect(datasets[0]).toMatchObject({
    label: "Target RPM",
    borderColor: "#123456",
    stepped: "before",
    borderDash: [5, 5],
  });
  expect(datasets[0].data).toEqual([
    { x: 0, y: 5 },
    { x: 1000, y: 5 },
    { x: 2000, y: 9 },
    { x: 3000, y: 9 },
  ]);
});

test("buildTargetSeriesDatasets excludes target lines without a targetSeries", () => {
  const lines: GraphLine[] = [{ type: "target", value: 5, color: "#123456" }];

  expect(buildTargetSeriesDatasets(lines, [0, 1000])).toEqual([]);
});
