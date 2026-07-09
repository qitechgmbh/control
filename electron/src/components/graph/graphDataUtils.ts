import { TimeSeries } from "@/lib/timeseries";
import { BigGraphProps, SeriesData } from "./types";

/** Normalizes a BigGraph `newData` prop (a single series or an array) to an array. */
export function normalizeDataSeries(
  data: BigGraphProps["newData"],
): SeriesData[] {
  if (Array.isArray(data)) {
    return data;
  }
  return [data];
}

export function getPrimarySeries(
  data: BigGraphProps["newData"],
): SeriesData | null {
  const normalized = normalizeDataSeries(data);
  return normalized.find((series) => series.newData !== null) || null;
}

export function getPrimarySeriesData(
  data: BigGraphProps["newData"],
): TimeSeries | null {
  return getPrimarySeries(data)?.newData ?? null;
}

export function formatDisplayValue(
  value: number | undefined | null,
  renderValue?: (value: number) => string,
): string {
  if (value === undefined || value === null) return "N/A";
  return renderValue ? renderValue(value) : value.toFixed(2);
}

/** Flattens a BigGraph `newData` prop to the TimeSeries + display metadata for every non-null series. */
export function getAllTimeSeries(
  data: BigGraphProps["newData"],
): Array<{ series: TimeSeries; title?: string; color?: string }> {
  const normalized = normalizeDataSeries(data);
  return normalized
    .filter((series) => series.newData !== null)
    .map((series) => ({
      series: series.newData!,
      title: series.title,
      color: series.color,
    }));
}
