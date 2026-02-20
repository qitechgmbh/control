/**
 * Handles statistical calculations for time series data
 */
export class StatisticsCalculator {
  static calculate(values: number[]): {
    min: number;
    max: number;
    avg: number;
    stdDev: number;
    range: number;
    p25: number;
    p50: number;
    p75: number;
  } {
    if (values.length === 0) {
      throw new Error("Cannot calculate statistics for empty array");
    }

    const min = Math.min(...values);
    const max = Math.max(...values);
    const avg = values.reduce((a, b) => a + b, 0) / values.length;
    const stdDev = Math.sqrt(
      values.reduce((sum, val) => sum + Math.pow(val - avg, 2), 0) /
        values.length,
    );
    const range = max - min;

    const sortedValues = [...values].sort((a, b) => a - b);
    const p25 = sortedValues[Math.floor(sortedValues.length * 0.25)];
    const p50 = sortedValues[Math.floor(sortedValues.length * 0.5)];
    const p75 = sortedValues[Math.floor(sortedValues.length * 0.75)];

    return { min, max, avg, stdDev, range, p25, p50, p75 };
  }
}
