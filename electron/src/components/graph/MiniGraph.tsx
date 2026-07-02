import React, { useEffect, useRef } from "react";
import {
  createChart,
  IChartApi,
  ISeriesApi,
  LineSeries,
} from "lightweight-charts";
import { seriesToUPlotData, TimeSeries } from "@/lib/timeseries";
import { msToTime } from "./dataHelpers";

type MiniGraphProps = {
  newData: TimeSeries | null;
  width: number;
  renderValue?: (value: number) => string;
};

const HEIGHT = 64;

// Sparklines this small don't need to redraw faster than this to look smooth,
// and dashboards mount many MiniGraph instances at once (e.g. 11 on a single
// page) — capping well below display refresh rate meaningfully cuts the
// aggregate redraw work across all of them.
const UPDATE_FPS_CAP = 20;
const MIN_UPDATE_INTERVAL_MS = 1000 / UPDATE_FPS_CAP;

export function MiniGraph({ newData, width, renderValue }: MiniGraphProps) {
  const divRef = useRef<HTMLDivElement | null>(null);
  const chartRef = useRef<IChartApi | null>(null);
  const seriesRef = useRef<ISeriesApi<"Line"> | null>(null);
  const lastUpdateTimestamp = useRef<number>(0);
  const isInitialized = useRef(false);
  const pendingPoint = useRef<{ time: ReturnType<typeof msToTime>; value: number } | null>(
    null,
  );
  const flushTimeoutId = useRef<ReturnType<typeof setTimeout> | null>(null);
  const lastFlushTime = useRef(0);

  // Initialize chart once
  useEffect(() => {
    if (!divRef.current || !newData?.short || isInitialized.current) return;

    const chart = createChart(divRef.current, {
      width,
      height: HEIGHT,
      handleScroll: false,
      handleScale: false,
      layout: { attributionLogo: false },
      crosshair: {
        vertLine: { visible: false, labelVisible: false },
        horzLine: { visible: false, labelVisible: false },
      },
      timeScale: { visible: false },
      rightPriceScale: {
        visible: true,
        borderVisible: false,
        autoScale: true,
      },
      grid: {
        vertLines: { visible: false },
        horzLines: { color: "#ccc" },
      },
    });

    const series = chart.addSeries(LineSeries, {
      color: "black",
      lineWidth: 2,
      priceLineVisible: false,
      lastValueVisible: false,
      crosshairMarkerVisible: false,
      priceFormat: renderValue
        ? { type: "custom", formatter: renderValue, minMove: 0.001 }
        : { type: "price", precision: 1, minMove: 0.1 },
    });

    const [timestamps, values] = seriesToUPlotData(newData.short);
    series.setData(
      timestamps.map((t, i) => ({ time: msToTime(t), value: values[i] })),
    );

    chartRef.current = chart;
    seriesRef.current = series;
    lastUpdateTimestamp.current = newData.current?.timestamp ?? 0;
    isInitialized.current = true;

    return () => {
      if (flushTimeoutId.current !== null) {
        clearTimeout(flushTimeoutId.current);
        flushTimeoutId.current = null;
      }
      pendingPoint.current = null;
      chart.remove();
      chartRef.current = null;
      seriesRef.current = null;
      isInitialized.current = false;
    };
    // newData.short is a new object reference on every tick (Immer); only its
    // timeWindow (a stable config value) should ever cause re-initialization.
  }, [width, newData?.short?.timeWindow, renderValue]);

  // Stash the latest sample and flush at most once per UPDATE_FPS_CAP window —
  // dashboards mount many MiniGraph instances at once (e.g. 11 on a single
  // page), each ticking at ~30Hz; calling update() unthrottled per instance
  // reintroduces the redraw pressure the old uPlot version's RAF-throttle
  // existed to avoid (update() does real bookkeeping work on every call, not
  // just on the frame that actually repaints). A fixed-interval throttle caps
  // the redraw rate below display refresh rate, which RAF alone doesn't.
  useEffect(() => {
    const cur = newData?.current;
    if (!seriesRef.current || !cur) return;
    if (cur.timestamp <= lastUpdateTimestamp.current) return;

    lastUpdateTimestamp.current = cur.timestamp;
    pendingPoint.current = { time: msToTime(cur.timestamp), value: cur.value };

    if (flushTimeoutId.current !== null) return;

    const elapsed = performance.now() - lastFlushTime.current;
    const delay = Math.max(MIN_UPDATE_INTERVAL_MS - elapsed, 0);

    flushTimeoutId.current = setTimeout(() => {
      flushTimeoutId.current = null;
      lastFlushTime.current = performance.now();
      const point = pendingPoint.current;
      pendingPoint.current = null;
      if (point) {
        seriesRef.current?.update(point);
      }
    }, delay);
  }, [newData?.current?.timestamp, newData?.current?.value]);

  // Resize on width change
  useEffect(() => {
    chartRef.current?.resize(width, HEIGHT);
  }, [width]);

  return (
    <div
      ref={divRef}
      style={{
        width: "100%",
        height: HEIGHT,
        overflow: "hidden",
      }}
    />
  );
}
