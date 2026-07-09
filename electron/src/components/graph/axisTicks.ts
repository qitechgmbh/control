const MAX_Y_TICK_PRECISION = 4;

/**
 * Formats Y-axis tick values, increasing decimal precision only as far as
 * needed to keep every label distinct (e.g. avoids 12.34 and 12.28 both
 * rounding to "12.3"). Ported from uPlot's axis `values` callback.
 */
export function formatUniqueYAxisTicks(
  values: number[],
  renderValue?: (value: number) => string,
): string[] {
  if (renderValue) {
    const rendered = values.map((value) => renderValue(value));
    if (new Set(rendered).size === rendered.length) {
      return rendered;
    }
  }

  for (let precision = 0; precision <= MAX_Y_TICK_PRECISION; precision++) {
    const formatted = values.map((value) => value.toFixed(precision));
    if (new Set(formatted).size === formatted.length) {
      return formatted;
    }
  }

  return values.map((value) => value.toFixed(MAX_Y_TICK_PRECISION));
}

/**
 * Chart.js time-scale `time.displayFormats`, keyed by the unit Chart.js
 * auto-selects for the visible range. Format tokens are date-fns syntax
 * (chartjs-adapter-date-fns is the configured adapter).
 */
export const TIME_AXIS_DISPLAY_FORMATS = {
  millisecond: "HH:mm:ss.SSS",
  second: "HH:mm:ss",
  minute: "HH:mm:ss",
  hour: "HH:mm",
  day: "MMM d, HH:mm",
  week: "MMM d",
  month: "MMM yyyy",
  quarter: "MMM yyyy",
  year: "yyyy",
} as const;
