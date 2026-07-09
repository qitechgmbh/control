import {
  Chart,
  type ChartConfiguration,
  type ChartDataset,
  type TooltipModel,
} from "chart.js";
import type { Marker } from "@/stores/markerStore";
import { seriesToUPlotData, TimeSeries } from "@/lib/timeseries";
import { ensureChartJsRegistered } from "./chartSetup";
import { formatUniqueYAxisTicks, TIME_AXIS_DISPLAY_FORMATS } from "./axisTicks";
import { buildAnnotations, buildTargetSeriesDatasets } from "./annotations";
import { getAllTimeSeries } from "./graphDataUtils";
import { DEFAULT_SERIES_COLORS } from "./constants";
import { BigGraphProps, GraphConfig } from "./types";

type SeriesPoint = { x: number; y: number };
type MutableScaleRange = { min?: number; max?: number };
type OriginalSeries = { series: TimeSeries; title?: string; color?: string };

export type LiveLineChartConfig = {
  newData: BigGraphProps["newData"];
  config: GraphConfig;
  colors: { primary: string; grid: string; axis: string; background: string };
  renderValue?: (value: number) => string;
  selectedTimeWindow: number | "all";
  visibleSeries: boolean[];
  markers?: Marker[];
};

export type LiveLineChartSnapshot = {
  cursorValue: number | null;
  cursorValues: (number | null)[];
};

type ComputedRender = {
  allOriginalSeries: OriginalSeries[];
  datasets: ChartDataset<"line", SeriesPoint[]>[];
  xMin: number;
  xMax: number;
  yMin: number;
  yMax: number;
};

/**
 * Owns one Chart.js line-chart instance for a machine telemetry graph:
 * builds datasets/scales/annotations from a GraphConfig + live TimeSeries
 * snapshot, tracks per-series cursor values on hover, and exposes series
 * visibility toggling. This is a "skeleton": it always renders whatever
 * data snapshot it's given (via updateData) — true incremental live-append,
 * historical-mode freezing, and interactive zoom/pan are added on top of
 * this in later phases, not yet present here.
 */
export class LiveLineChart {
  #chart: Chart<"line", SeriesPoint[]>;
  #config: LiveLineChartConfig;
  #allOriginalSeries: OriginalSeries[] = [];
  #snapshot: LiveLineChartSnapshot;
  #listeners = new Set<(snapshot: LiveLineChartSnapshot) => void>();
  #destroyed = false;

  constructor(canvas: HTMLCanvasElement, initial: LiveLineChartConfig) {
    ensureChartJsRegistered();
    this.#config = initial;

    const computed = this.#compute();
    this.#allOriginalSeries = computed.allOriginalSeries;
    this.#snapshot = {
      cursorValue: null,
      cursorValues: new Array(computed.allOriginalSeries.length).fill(null),
    };

    this.#chart = new Chart(canvas, this.#buildChartConfig(computed));
  }

  get chartInstance(): Chart {
    return this.#chart;
  }

  getSnapshot(): LiveLineChartSnapshot {
    return this.#snapshot;
  }

  subscribe(listener: (snapshot: LiveLineChartSnapshot) => void): () => void {
    this.#listeners.add(listener);
    return () => this.#listeners.delete(listener);
  }

  /** Full recompute-and-render from a fresh data/config snapshot. */
  updateData(next: LiveLineChartConfig): void {
    if (this.#destroyed) return;
    this.#config = next;

    const computed = this.#compute();
    this.#allOriginalSeries = computed.allOriginalSeries;

    this.#chart.data.datasets = computed.datasets;

    const scales = this.#chart.options.scales!;
    const xScale = scales.x as MutableScaleRange;
    const yScale = scales.y as MutableScaleRange;
    xScale.min = computed.xMin;
    xScale.max = computed.xMax;
    yScale.min = computed.yMin;
    yScale.max = computed.yMax;

    const annotationOptions = this.#chart.options.plugins!.annotation!;
    annotationOptions.annotations = buildAnnotations(
      next.config.lines,
      next.markers,
    );

    this.#chart.update("none");
  }

  destroy(): void {
    this.#destroyed = true;
    this.#chart.destroy();
  }

  #compute(): ComputedRender {
    const { newData, config, visibleSeries, selectedTimeWindow } =
      this.#config;

    const allOriginalSeries = getAllTimeSeries(newData);
    const { min: xMin, max: xMax } = this.#computeXRange(
      allOriginalSeries,
      selectedTimeWindow,
    );

    const seriesDatasets: ChartDataset<"line", SeriesPoint[]>[] =
      allOriginalSeries.map(({ series, title, color }, index) => {
        const [timestamps, values] = seriesToUPlotData(series.long);
        return {
          label: title || `Series ${index + 1}`,
          data: timestamps.map((x, i) => ({ x, y: values[i] })),
          borderColor:
            color || DEFAULT_SERIES_COLORS[index % DEFAULT_SERIES_COLORS.length],
          borderWidth: 2,
          spanGaps: true,
          pointRadius: 0,
          hidden: visibleSeries[index] === false,
        };
      });

    const primaryTimestamps =
      allOriginalSeries.length > 0
        ? seriesToUPlotData(allOriginalSeries[0].series.long)[0]
        : [];

    const targetDatasets: ChartDataset<"line", SeriesPoint[]>[] =
      buildTargetSeriesDatasets(config.lines, primaryTimestamps).map(
        (dataset) => ({
          label: dataset.label,
          data: dataset.data,
          borderColor: dataset.borderColor,
          borderWidth: dataset.borderWidth,
          borderDash: dataset.borderDash,
          stepped: dataset.stepped,
          pointRadius: 0,
          spanGaps: true,
        }),
      );

    const { min: yMin, max: yMax } = this.#computeYRange(
      allOriginalSeries,
      visibleSeries,
      config,
      xMin,
      xMax,
    );

    return {
      allOriginalSeries,
      datasets: [...seriesDatasets, ...targetDatasets],
      xMin,
      xMax,
      yMin,
      yMax,
    };
  }

  #computeXRange(
    allOriginalSeries: OriginalSeries[],
    selectedTimeWindow: number | "all",
  ): { min: number; max: number } {
    let end = 0;
    let fullStart: number | undefined;

    allOriginalSeries.forEach(({ series }) => {
      if (series.current && series.current.timestamp > end) {
        end = series.current.timestamp;
      }
      const [timestamps] = seriesToUPlotData(series.long);
      if (timestamps.length > 0) {
        end = Math.max(end, timestamps[timestamps.length - 1]);
        fullStart =
          fullStart === undefined
            ? timestamps[0]
            : Math.min(fullStart, timestamps[0]);
      }
    });

    if (end === 0) end = Date.now();
    if (fullStart === undefined) fullStart = end;

    if (selectedTimeWindow === "all") {
      return { min: fullStart, max: end };
    }
    return { min: Math.max(end - selectedTimeWindow, fullStart), max: end };
  }

  #computeYRange(
    allOriginalSeries: OriginalSeries[],
    visibleSeries: boolean[],
    config: GraphConfig,
    xMin: number,
    xMax: number,
  ): { min: number; max: number } {
    let allValues: number[] = [];

    allOriginalSeries.forEach(({ series }, index) => {
      if (visibleSeries[index] === false) return;
      const [timestamps, values] = seriesToUPlotData(series.long);
      for (let i = 0; i < timestamps.length; i++) {
        if (timestamps[i] >= xMin && timestamps[i] <= xMax) {
          allValues.push(values[i]);
        }
      }
    });

    config.lines?.forEach((line) => {
      if (line.show !== false) allValues.push(line.value);
    });

    if (allValues.length === 0 && allOriginalSeries.length > 0) {
      const [, values] = seriesToUPlotData(allOriginalSeries[0].series.long);
      allValues = values;
    }

    if (allValues.length === 0) {
      return { min: -1, max: 1 };
    }

    const minY = Math.min(...allValues);
    const maxY = Math.max(...allValues);
    const range = maxY - minY || Math.abs(maxY) * 0.1 || 1;
    return { min: minY - range * 0.1, max: maxY + range * 0.1 };
  }

  #buildChartConfig(
    computed: ComputedRender,
  ): ChartConfiguration<"line", SeriesPoint[]> {
    const { colors, renderValue, config, markers } = this.#config;

    return {
      type: "line",
      data: { datasets: computed.datasets },
      options: {
        responsive: true,
        maintainAspectRatio: false,
        animation: false,
        parsing: false,
        normalized: true,
        interaction: { mode: "index", intersect: false },
        plugins: {
          legend: { display: false },
          tooltip: {
            enabled: false,
            mode: "index",
            intersect: false,
            external: (context) => this.#handleTooltip(context),
          },
          annotation: {
            annotations: buildAnnotations(config.lines, markers),
          },
        },
        scales: {
          x: {
            type: "time",
            min: computed.xMin,
            max: computed.xMax,
            time: { displayFormats: TIME_AXIS_DISPLAY_FORMATS },
            grid: { color: colors.grid },
            border: { color: colors.axis },
            ticks: { color: colors.axis },
          },
          y: {
            type: "linear",
            min: computed.yMin,
            max: computed.yMax,
            grid: { color: colors.grid },
            border: { color: colors.axis },
            ticks: {
              color: colors.axis,
              callback: (value, index, ticks) =>
                (ticks[index] as { label?: string }).label ?? String(value),
            },
            afterBuildTicks: (axis) => {
              const labels = formatUniqueYAxisTicks(
                axis.ticks.map((tick) => tick.value),
                renderValue,
              );
              axis.ticks.forEach((tick, index) => {
                (tick as { label?: string }).label = labels[index];
              });
            },
          },
        },
      },
    };
  }

  #handleTooltip = (context: {
    chart: Chart;
    tooltip: TooltipModel<"line">;
  }): void => {
    const dataPoints = context.tooltip.dataPoints;
    const seriesCount = this.#allOriginalSeries.length;

    if (context.tooltip.opacity === 0 || !dataPoints?.length) {
      this.#emit({
        cursorValue: null,
        cursorValues: new Array(seriesCount).fill(null),
      });
      return;
    }

    const timestamp = dataPoints[0]?.parsed.x as number | undefined;
    const cursorValues: (number | null)[] = new Array(seriesCount).fill(null);

    dataPoints.forEach((point) => {
      if (point.datasetIndex >= seriesCount) return;
      const series = this.#allOriginalSeries[point.datasetIndex].series;
      const value = point.parsed.y as number;
      const current = series.current;
      const isNearCurrent =
        current &&
        timestamp !== undefined &&
        Math.abs(timestamp - current.timestamp) < 1000;
      cursorValues[point.datasetIndex] = isNearCurrent ? current.value : value;
    });

    const cursorValue = cursorValues.find((value) => value !== null) ?? null;
    this.#emit({ cursorValue, cursorValues });
  };

  #emit(patch: Partial<LiveLineChartSnapshot>): void {
    this.#snapshot = { ...this.#snapshot, ...patch };
    this.#listeners.forEach((listener) => listener(this.#snapshot));
  }
}
