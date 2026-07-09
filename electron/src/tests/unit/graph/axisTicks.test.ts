import { expect, test } from "vitest";
import {
  formatUniqueYAxisTicks,
  TIME_AXIS_DISPLAY_FORMATS,
} from "@/components/graph/axisTicks";

test("formatUniqueYAxisTicks uses renderValue when it already produces unique labels", () => {
  const result = formatUniqueYAxisTicks([1, 2, 3], (v) => `${v}°C`);
  expect(result).toEqual(["1°C", "2°C", "3°C"]);
});

test("formatUniqueYAxisTicks falls back to precision when renderValue collides", () => {
  const renderValue = () => "same";
  const result = formatUniqueYAxisTicks([1, 2], renderValue);
  expect(result).toEqual(["1", "2"]);
});

test("formatUniqueYAxisTicks increases precision only as far as needed for uniqueness", () => {
  const result = formatUniqueYAxisTicks([12.341, 12.343]);
  expect(result).toEqual(["12.341", "12.343"]);
});

test("formatUniqueYAxisTicks stays at precision 0 when values are already unique", () => {
  const result = formatUniqueYAxisTicks([10, 20, 30]);
  expect(result).toEqual(["10", "20", "30"]);
});

test("formatUniqueYAxisTicks caps at max precision for values that never diverge", () => {
  const result = formatUniqueYAxisTicks([5, 5]);
  expect(result).toEqual(["5.0000", "5.0000"]);
});

test("TIME_AXIS_DISPLAY_FORMATS covers every Chart.js time unit with a date-fns token string", () => {
  expect(TIME_AXIS_DISPLAY_FORMATS.second).toBe("HH:mm:ss");
  expect(TIME_AXIS_DISPLAY_FORMATS.hour).toBe("HH:mm");
  expect(Object.keys(TIME_AXIS_DISPLAY_FORMATS)).toEqual([
    "millisecond",
    "second",
    "minute",
    "hour",
    "day",
    "week",
    "month",
    "quarter",
    "year",
  ]);
});
