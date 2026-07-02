import {
  createChart as createLwcChart,
  IChartApi,
  ISeriesApi,
  LineSeries,
  LineStyle,
  CrosshairMode,
  Time,
  TickMarkType,
} from "lightweight-charts";
import {
  seriesToUPlotData,
  TimeSeries,
  alignTargetSeriesToTimestamps,
} from "@/lib/timeseries";
import { BigGraphProps, CreateChartParams, LineSeriesRef } from "./types";
import { normalizeDataSeries, msToTime, timeToMs } from "./dataHelpers";

const DEFAULT_SERIES_COLORS = [
  "#3b82f6",
  "#ef4444",
  "#10b981",
  "#f59e0b",
  "#8b5cf6",
];

// A config line is only overlay-driven (invisible native series + SVG dashes)
// when it's a time-varying target line with a dash pattern — lightweight-charts,
// like uPlot before it, can't natively draw a stepped-dashed line from a second
// time series. Constant-value threshold/target lines render as normal series.
export function isOverlayDrivenLine(line: {
  type: string;
  dash?: number[];
  targetSeries?: TimeSeries;
}): boolean {
  return line.type === "target" && !!line.targetSeries && !!line.dash?.length;
}

function formatTickMark(time: Time, tickMarkType: TickMarkType): string {
  const date = new Date(timeToMs(time));
  switch (tickMarkType) {
    case TickMarkType.Year:
      return date.toLocaleDateString(undefined, { year: "numeric" });
    case TickMarkType.Month:
      return date.toLocaleDateString(undefined, {
        month: "short",
        year: "numeric",
      });
    case TickMarkType.DayOfMonth:
      return date.toLocaleDateString(undefined, {
        month: "short",
        day: "numeric",
      });
    case TickMarkType.TimeWithSeconds:
      return date.toLocaleTimeString(undefined, {
        hour12: false,
        hour: "2-digit",
        minute: "2-digit",
        second: "2-digit",
      });
    case TickMarkType.Time:
    default:
      return date.toLocaleTimeString(undefined, {
        hour12: false,
        hour: "2-digit",
        minute: "2-digit",
      });
  }
}

/**
 * Sets the visible time range without triggering the "user changed the view"
 * side effects (manual mode, zoom sync) wired up in createChart's range
 * subscription — used for every *programmatic* range update (live ticking,
 * time-window changes, incoming sync state).
 */
export function setVisibleRangeSilently(
  chart: IChartApi,
  suppressRangeEventRef: React.RefObject<boolean>,
  range: { min: number; max: number },
): void {
  suppressRangeEventRef.current = true;
  try {
    chart.timeScale().setVisibleRange({
      from: msToTime(range.min),
      to: msToTime(range.max),
    });
  } catch {
    // Range can be rejected (e.g. min === max); the suppress flag is cleared
    // below regardless so the subscription doesn't get stuck ignoring input.
  }
}

export function setYAutoScale(chart: IChartApi, enabled: boolean): void {
  chart.priceScale("right").applyOptions({ autoScale: enabled });
}

export function createChart({
  containerRef,
  chartRef,
  chartRefOut,
  seriesRefs,
  newData,
  config,
  colors,
  renderValue,
  viewMode,
  selectedTimeWindow,
  isLiveMode,
  startTimeRef,
  manualScaleRef,
  suppressRangeEventRef,
  graphId,
  syncGraph,
  getHistoricalEndTimestamp,
  setViewMode,
  setIsLiveMode,
  setCursorValue,
  setCursorValues,
  visibleSeries,
  showFromTimestamp,
}: CreateChartParams): void {
  if (!containerRef.current) return;

  const allOriginalSeries = getAllTimeSeries(newData);
  if (allOriginalSeries.length === 0) return;

  const primarySeries = allOriginalSeries[0].series;
  const [timestamps] = seriesToUPlotData(primarySeries.long);
  if (timestamps.length === 0) return;

  if (startTimeRef.current === null) {
    startTimeRef.current = timestamps[0];
  }

  const fullStart = startTimeRef.current ?? timestamps[0] ?? 0;

  let initialMin: number, initialMax: number;
  if (viewMode === "manual" && manualScaleRef.current) {
    initialMin = manualScaleRef.current.x.min;
    initialMax = manualScaleRef.current.x.max;
  } else if (selectedTimeWindow === "all") {
    initialMin = showFromTimestamp ?? fullStart;
    initialMax = getHistoricalEndTimestamp();
  } else {
    const endTimestamp = getHistoricalEndTimestamp();
    if (isLiveMode) {
      const defaultViewStart = Math.max(
        endTimestamp - (selectedTimeWindow as number),
        fullStart,
      );
      initialMin = showFromTimestamp ?? defaultViewStart;
    } else {
      initialMin =
        showFromTimestamp ?? endTimestamp - (selectedTimeWindow as number);
    }
    initialMax = endTimestamp;
  }

  // Always destroy the existing chart before creating a new one.
  if (chartRef.current) {
    if (chartRefOut?.current != null) {
      chartRefOut.current = null;
    }
    chartRef.current.remove();
    chartRef.current = null;
  }

  const chart = createLwcChart(containerRef.current, {
    autoSize: true,
    layout: {
      background: { color: colors.background },
      textColor: colors.axis,
      fontFamily: "Inter, system-ui, sans-serif",
      fontSize: 12,
      attributionLogo: false,
    },
    grid: {
      vertLines: { color: colors.grid },
      horzLines: { color: colors.grid },
    },
    rightPriceScale: {
      borderVisible: false,
      autoScale: true,
    },
    timeScale: {
      borderVisible: false,
      timeVisible: true,
      secondsVisible: true,
      tickMarkFormatter: formatTickMark,
    },
    handleScroll: {
      mouseWheel: true,
      pressedMouseMove: true,
      horzTouchDrag: true,
      vertTouchDrag: false,
    },
    handleScale: {
      mouseWheel: true,
      pinch: true,
      axisPressedMouseMove: { time: false, price: false },
    },
    crosshair: {
      mode: CrosshairMode.Magnet,
    },
  });

  chartRef.current = chart;
  if (chartRefOut) {
    chartRefOut.current = chart;
  }

  const defaultColors = DEFAULT_SERIES_COLORS;
  const dataSeries: ISeriesApi<"Line">[] = allOriginalSeries.map(
    ({ title, color, series: timeSeries }, index) => {
      const isVisible = !visibleSeries || visibleSeries[index];
      const seriesColor = color || defaultColors[index % defaultColors.length];
      const series = chart.addSeries(LineSeries, {
        title: title || `Series ${index + 1}`,
        color: seriesColor,
        lineWidth: 2,
        visible: isVisible,
        priceLineVisible: false,
        lastValueVisible: allOriginalSeries.length > 1,
        priceFormat: renderValue
          ? {
              type: "custom",
              formatter: renderValue,
              minMove: 0.001,
            }
          : { type: "price", precision: 2, minMove: 0.01 },
      });
      const [seriesTimestamps, seriesValues] = seriesToUPlotData(
        timeSeries.long,
      );
      series.setData(
        seriesTimestamps.map((t, i) => ({
          time: msToTime(t),
          value: seriesValues[i],
        })),
      );
      return series;
    },
  );

  const lineSeries: LineSeriesRef[] = [];
  config.lines?.forEach((line) => {
    if (line.show === false) return;

    const dash = line.dash ?? (line.type === "threshold" ? [5, 5] : undefined);
    const overlayDriven = isOverlayDrivenLine({ ...line, dash });
    const targetSeries = line.type === "target" ? line.targetSeries : undefined;
    const lineData = targetSeries
      ? alignTargetSeriesToTimestamps(targetSeries, timestamps, line.value)
      : timestamps.map(() => line.value);

    const series = chart.addSeries(LineSeries, {
      title: line.label,
      color: line.color,
      lineWidth: (line.width ?? 1) as 1 | 2 | 3 | 4,
      lineStyle: dash?.length ? LineStyle.Dashed : LineStyle.Solid,
      lineVisible: !overlayDriven,
      visible: true,
      priceLineVisible: false,
      lastValueVisible: false,
      crosshairMarkerVisible: false,
    });
    series.setData(
      timestamps.map((t, i) => ({ time: msToTime(t), value: lineData[i] })),
    );

    lineSeries.push({ api: series, line, isOverlayDriven: overlayDriven });
  });

  seriesRefs.current = { dataSeries, lineSeries };

  // Guard against the range-change subscription (registered below) treating
  // this initial programmatic range as a user pan/zoom gesture.
  suppressRangeEventRef.current = true;
  chart.timeScale().setVisibleRange({
    from: msToTime(initialMin),
    to: msToTime(initialMax),
  });

  // Cursor readout — lightweight-charts has no built-in tooltip, so BigGraph
  // renders the numeric value itself from this state, same as uPlot before.
  chart.subscribeCrosshairMove((param) => {
    if (!param.time) {
      setCursorValue(null);
      setCursorValues(new Array(allOriginalSeries.length).fill(null));
      return;
    }

    const timestamp = timeToMs(param.time);

    if (allOriginalSeries.length === 1) {
      const point = param.seriesData.get(dataSeries[0]) as
        | { value: number }
        | undefined;
      const cur = allOriginalSeries[0]?.series?.current;
      const isNearCurrent = cur && Math.abs(timestamp - cur.timestamp) < 1000;
      const displayValue = isNearCurrent ? cur.value : point?.value;
      setCursorValue(displayValue ?? null);
    } else {
      const cursorValuesArray: (number | null)[] = allOriginalSeries.map(
        (originalSeries, index) => {
          const point = param.seriesData.get(dataSeries[index]) as
            | { value: number }
            | undefined;
          const cur = originalSeries.series?.current;
          const isNearCurrent =
            cur && Math.abs(timestamp - cur.timestamp) < 1000;
          const displayValue = isNearCurrent ? cur.value : point?.value;
          return displayValue ?? null;
        },
      );
      setCursorValues(cursorValuesArray);
      const firstVisibleValue = cursorValuesArray.find((v) => v !== null);
      setCursorValue(firstVisibleValue ?? null);
    }
  });

  // Fires on every visible-range change: user pan/zoom AND our own
  // programmatic setVisibleRange calls. Suppress the latter via the ref flag
  // set immediately before each programmatic call (see setVisibleRangeSilently).
  chart.timeScale().subscribeVisibleTimeRangeChange((range) => {
    if (suppressRangeEventRef.current) {
      suppressRangeEventRef.current = false;
      return;
    }
    if (!range) return;

    const newMin = timeToMs(range.from);
    const newMax = timeToMs(range.to);

    manualScaleRef.current = { x: { min: newMin, max: newMax } };
    setViewMode("manual");
    setIsLiveMode(false);
    // Native autoScale already fit Y to the new visible range as part of this
    // same gesture; freeze it there so panning afterward doesn't keep rescaling.
    setYAutoScale(chart, false);

    syncGraph?.onZoomChange?.(graphId, { min: newMin, max: newMax });
    syncGraph?.onViewModeChange?.(graphId, "manual", false);
  });
}

// Helper function to get all valid TimeSeries from DataSeries
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
