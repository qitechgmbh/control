import type { Time, UTCTimestamp } from "lightweight-charts";
import { BigGraphProps, SeriesData } from "./types";

// lightweight-charts' Time/UTCTimestamp is fractional Unix seconds; our data is
// tracked in milliseconds everywhere else. Convert only at the chart boundary.
// Fractional (not truncated) seconds are required: the short buffer samples
// every 20ms, so truncating to whole seconds would collide many points together.
export function msToTime(ms: number): Time {
  return (ms / 1000) as UTCTimestamp;
}

export function timeToMs(time: Time): number {
  return (time as number) * 1000;
}

// Helper function to normalize data to array format
export function normalizeDataSeries(
  data: BigGraphProps["newData"],
): SeriesData[] {
  if (Array.isArray(data)) {
    return data;
  }
  return [data];
}

// Helper function to get primary series for display value
export function getPrimarySeries(
  data: BigGraphProps["newData"],
): SeriesData | null {
  const normalized = normalizeDataSeries(data);
  return normalized.find((series) => series.newData !== null) || null;
}

export function getPrimarySeriesData(data: BigGraphProps["newData"]) {
  const normalized = normalizeDataSeries(data);
  const primarySeries = normalized.find((series) => series.newData !== null);
  return primarySeries?.newData || null;
}

// Helper function to format value for display
export function formatDisplayValue(
  value: number | undefined | null,
  renderValue?: (value: number) => string,
): string {
  if (value === undefined || value === null) return "N/A";
  return renderValue ? renderValue(value) : value.toFixed(2);
}
