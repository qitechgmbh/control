import { Chart, type ChartConfiguration } from "chart.js";
import {
  getSeriesMinMax,
  seriesToUPlotData,
  TimeSeries,
} from "@/lib/timeseries";
import { ensureChartJsRegistered } from "./chartSetup";

export const SPARKLINE_HEIGHT = 64;

type SparklinePoint = { x: number; y: number };
type MutableScaleRange = { min?: number; max?: number };
export type SparklineRange = { min: number; max: number };

type SparklineChartInit = {
  width: number;
  renderValue?: (value: number) => string;
  /**
   * Fixed Y-axis bounds. When set, the scale never recomputes from data:
   * no per-update min/max scan and no rescaling redraw, which matters for
   * mini graphs that redraw on every live tick. Pass the machine's known
   * physical range (e.g. a sensor's 0-to-rated-max) rather than leaving the
   * scale to auto-fit noisy live data.
   */
  range?: SparklineRange;
};

function hashSeries(timestamps: number[], values: number[]): string {
  if (timestamps.length === 0) return "";
  // Hash only first, last, and length: cheap and sufficient to detect
  // whether the rolling window actually produced a new sample.
  return `${timestamps[0]}-${timestamps[timestamps.length - 1]}-${timestamps.length}-${values[values.length - 1]}`;
}

function toPoints(timestamps: number[], values: number[]): SparklinePoint[] {
  return timestamps.map((x, i) => ({ x, y: values[i] }));
}

/**
 * Owns one Chart.js sparkline: a hidden x-axis, minimal y-axis, and
 * RAF-throttled live updates with a data-hash dedup check so redundant
 * redraws (no new sample since the last frame) are skipped. Ported 1:1 from
 * MiniGraph.tsx's inline uPlot logic, with one deliberate behavior change:
 * width changes now call `resize()` instead of destroying and recreating
 * the whole chart instance (the previous implementation had `width` in its
 * chart-creation effect's dependency array, so every container resize tore
 * down and rebuilt the chart even though a resize was all that was needed).
 */
export class SparklineChart {
  #chart: Chart<"line", SparklinePoint[]>;
  #renderValue?: (value: number) => string;
  #fixedRange?: SparklineRange;
  #lastUpdateTimestamp = 0;
  #lastDataHash = "";
  #lastMinMax = { min: 0, max: 0 };
  #rafId = 0;
  #pendingSeries: TimeSeries | null = null;
  #destroyed = false;

  constructor(
    canvas: HTMLCanvasElement,
    series: TimeSeries,
    init: SparklineChartInit,
  ) {
    ensureChartJsRegistered();
    this.#renderValue = init.renderValue;
    this.#fixedRange = init.range;

    const short = series.short;
    const [timestamps, values] = seriesToUPlotData(short);
    const latestTimestamp =
      timestamps.length > 0 ? timestamps[timestamps.length - 1] : Date.now();
    const cutoff = latestTimestamp - short.timeWindow;

    let yMin: number;
    let yMax: number;
    if (this.#fixedRange) {
      yMin = this.#fixedRange.min;
      yMax = this.#fixedRange.max;
    } else {
      const { min: minY, max: maxY } = getSeriesMinMax(short);
      const dataRange = maxY - minY || 1;
      this.#lastMinMax = { min: minY, max: maxY };
      yMin = minY - dataRange * 0.1;
      yMax = maxY + dataRange * 0.1;
    }

    this.#lastDataHash = hashSeries(timestamps, values);
    this.#lastUpdateTimestamp = series.current?.timestamp ?? 0;

    this.#chart = new Chart(
      canvas,
      this.#buildConfig(
        toPoints(timestamps, values),
        cutoff,
        latestTimestamp,
        yMin,
        yMax,
      ),
    );
    this.#chart.resize(init.width, SPARKLINE_HEIGHT);
  }

  #buildConfig(
    data: SparklinePoint[],
    xMin: number,
    xMax: number,
    yMin: number,
    yMax: number,
  ): ChartConfiguration<"line", SparklinePoint[]> {
    return {
      type: "line",
      data: {
        datasets: [
          {
            data,
            borderColor: "black",
            borderWidth: 2,
            pointRadius: 0,
            spanGaps: true,
          },
        ],
      },
      options: {
        responsive: false,
        maintainAspectRatio: false,
        animation: false,
        events: [],
        parsing: false,
        normalized: true,
        layout: { padding: { top: 4, bottom: 4 } },
        plugins: {
          legend: { display: false },
          tooltip: { enabled: false },
        },
        scales: {
          x: { type: "time", display: false, min: xMin, max: xMax },
          y: {
            type: "linear",
            position: "right",
            min: yMin,
            max: yMax,
            border: { color: "#ccc", width: 0.5 },
            grid: { color: "#ccc", lineWidth: 0.5 },
            ticks: {
              callback: (value) => this.#formatTick(Number(value)),
            },
          },
        },
      },
    };
  }

  #formatTick(value: number): string {
    return this.#renderValue ? this.#renderValue(value) : value.toFixed(1);
  }

  setRenderValue(renderValue: ((value: number) => string) | undefined): void {
    this.#renderValue = renderValue;
    this.#chart.update("none");
  }

  setRange(range: SparklineRange | undefined): void {
    if (
      range?.min === this.#fixedRange?.min &&
      range?.max === this.#fixedRange?.max
    ) {
      return;
    }
    this.#fixedRange = range;
    if (range) {
      const yScale = this.#chart.options.scales!.y as MutableScaleRange;
      yScale.min = range.min;
      yScale.max = range.max;
      this.#chart.update("none");
    }
  }

  /** Call whenever the live tick (newData.current.timestamp) advances. */
  pushLatest(series: TimeSeries): void {
    if (this.#destroyed) return;
    this.#pendingSeries = series;
    if (this.#rafId) return;
    this.#rafId = requestAnimationFrame(this.#applyPending);
  }

  #applyPending = (): void => {
    this.#rafId = 0;
    const series = this.#pendingSeries;
    this.#pendingSeries = null;
    if (!series || this.#destroyed) return;
    this.#applyUpdate(series);
  };

  #applyUpdate(series: TimeSeries): void {
    const short = series.short;
    const current = series.current;
    if (!short || !current) return;
    if (current.timestamp <= this.#lastUpdateTimestamp) return;

    const [timestamps, values] = seriesToUPlotData(short);
    if (timestamps.length === 0) return;

    const dataHash = hashSeries(timestamps, values);
    if (dataHash === this.#lastDataHash) return;

    this.#lastUpdateTimestamp = current.timestamp;
    this.#lastDataHash = dataHash;

    const cutoff = current.timestamp - short.timeWindow;

    this.#chart.data.datasets[0].data = toPoints(timestamps, values);

    const scales = this.#chart.options.scales!;
    const xScale = scales.x as MutableScaleRange;
    xScale.min = cutoff;
    xScale.max = current.timestamp;

    // Fixed-range charts skip the min/max scan and scale rewrite below
    // entirely -- that's the performance win over auto-scaling.
    if (!this.#fixedRange) {
      const { min: minY, max: maxY } = getSeriesMinMax(short);
      const scalesChanged =
        minY !== this.#lastMinMax.min || maxY !== this.#lastMinMax.max;

      if (scalesChanged) {
        this.#lastMinMax = { min: minY, max: maxY };
        const dataRange = maxY - minY || 1;
        const yScale = scales.y as MutableScaleRange;
        yScale.min = minY - dataRange * 0.1;
        yScale.max = maxY + dataRange * 0.1;
      }
    }

    this.#chart.update("none");
  }

  resize(width: number): void {
    if (this.#destroyed) return;
    this.#chart.resize(width, SPARKLINE_HEIGHT);
  }

  destroy(): void {
    this.#destroyed = true;
    if (this.#rafId) {
      cancelAnimationFrame(this.#rafId);
      this.#rafId = 0;
    }
    this.#chart.destroy();
  }
}
