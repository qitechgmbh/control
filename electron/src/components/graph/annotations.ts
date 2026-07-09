import type { AnnotationOptions } from "chartjs-plugin-annotation";
import type { Marker } from "@/stores/markerStore";
import { alignTargetSeriesToTimestamps } from "@/lib/timeseries";
import type { GraphLine } from "./types";

const DEFAULT_THRESHOLD_DASH = [5, 5];
const DEFAULT_MARKER_DASH = [4, 4];

function buildLineLabel(
  content: string | undefined,
  position: "start" | "end",
): AnnotationOptions<"line">["label"] | undefined {
  if (!content) return undefined;
  return { display: true, content, position };
}

/**
 * Maps a single constant-value GraphLine (threshold, or a target line with
 * no time-varying targetSeries) to a horizontal chartjs-plugin-annotation
 * line. Target lines that track a targetSeries over time are NOT annotations
 * — see buildTargetSeriesDatasets, which renders them as regular (stepped,
 * dashed) Chart.js datasets instead, since annotation lines are straight.
 */
function buildConstantLineAnnotation(
  line: GraphLine,
): AnnotationOptions<"line"> | null {
  if (line.show === false) return null;

  switch (line.type) {
    case "threshold":
      return {
        type: "line",
        scaleID: "y",
        value: line.value,
        borderColor: line.color,
        borderWidth: line.width ?? 1,
        borderDash: line.dash ?? DEFAULT_THRESHOLD_DASH,
        label: buildLineLabel(line.label, "end"),
      };
    case "target":
      if (line.targetSeries) return null;
      return {
        type: "line",
        scaleID: "y",
        value: line.value,
        borderColor: line.color,
        borderWidth: line.width ?? 1,
        borderDash: line.dash,
        label: buildLineLabel(line.label, "end"),
      };
    case "user_marker":
      return {
        type: "line",
        scaleID: "x",
        value: line.markerTimestamp,
        borderColor: line.color,
        borderWidth: line.width ?? 2,
        borderDash: line.dash ?? DEFAULT_MARKER_DASH,
        label: buildLineLabel(line.label, "start"),
      };
  }
}

/**
 * Builds the `plugins.annotation.annotations` map from a GraphConfig's
 * lines, keyed so React re-renders with a stable set of ids.
 */
export function buildLineAnnotations(
  lines: GraphLine[] | undefined,
): Record<string, AnnotationOptions<"line">> {
  const annotations: Record<string, AnnotationOptions<"line">> = {};

  (lines ?? []).forEach((line, index) => {
    const annotation = buildConstantLineAnnotation(line);
    if (annotation) {
      annotations[`line-${index}`] = annotation;
    }
  });

  return annotations;
}

/**
 * Builds vertical marker annotations from user-placed timeline markers
 * (electron/src/stores/markerStore.ts), replacing marker/MarkerOverlay.tsx's
 * hand-rolled SVG coordinate math.
 */
export function buildMarkerAnnotations(
  markers: Marker[] | undefined,
): Record<string, AnnotationOptions<"line">> {
  const annotations: Record<string, AnnotationOptions<"line">> = {};

  (markers ?? []).forEach((marker, index) => {
    annotations[`marker-${index}`] = {
      type: "line",
      scaleID: "x",
      value: marker.timestamp,
      borderColor: marker.color ?? "#000000",
      borderWidth: 2,
      borderDash: DEFAULT_MARKER_DASH,
      label: buildLineLabel(marker.name, "start"),
    };
  });

  return annotations;
}

export function buildAnnotations(
  lines: GraphLine[] | undefined,
  markers: Marker[] | undefined,
): Record<string, AnnotationOptions<"line">> {
  return {
    ...buildLineAnnotations(lines),
    ...buildMarkerAnnotations(markers),
  };
}

export type TargetSeriesDataset = {
  key: string;
  label?: string;
  data: Array<{ x: number; y: number }>;
  borderColor: string;
  borderWidth: number;
  borderDash: number[];
  stepped: "before";
};

/**
 * Target lines that track a time-varying targetSeries render as regular
 * Chart.js line datasets (stepped, dashed) instead of annotations, since
 * chartjs-plugin-annotation's line type is a single straight segment.
 * Replaces the transparent-uPlot-series + SVG-overlay double-render that
 * TargetDashOverlay.tsx used to work around this.
 */
export function buildTargetSeriesDatasets(
  lines: GraphLine[] | undefined,
  timestamps: number[],
): TargetSeriesDataset[] {
  return (lines ?? [])
    .filter(
      (line): line is Extract<GraphLine, { type: "target" }> =>
        line.type === "target" && line.show !== false && !!line.targetSeries,
    )
    .map((line, index) => {
      const values = alignTargetSeriesToTimestamps(
        line.targetSeries!,
        timestamps,
        line.value,
      );

      return {
        key: `target-series-${index}`,
        label: line.label,
        data: timestamps.map((timestamp, i) => ({
          x: timestamp,
          y: values[i],
        })),
        borderColor: line.color,
        borderWidth: line.width ?? 1,
        borderDash: line.dash ?? DEFAULT_THRESHOLD_DASH,
        stepped: "before" as const,
      };
    });
}
