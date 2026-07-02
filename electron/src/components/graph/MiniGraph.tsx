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

export function MiniGraph({ newData, width, renderValue }: MiniGraphProps) {
  const divRef = useRef<HTMLDivElement | null>(null);
  const chartRef = useRef<IChartApi | null>(null);
  const seriesRef = useRef<ISeriesApi<"Line"> | null>(null);
  const lastUpdateTimestamp = useRef<number>(0);
  const isInitialized = useRef(false);

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
      chart.remove();
      chartRef.current = null;
      seriesRef.current = null;
      isInitialized.current = false;
    };
    // newData.short is a new object reference on every tick (Immer); only its
    // timeWindow (a stable config value) should ever cause re-initialization.
  }, [width, newData?.short?.timeWindow, renderValue]);

  // Push the latest sample as a single incremental point
  useEffect(() => {
    const cur = newData?.current;
    if (!seriesRef.current || !cur) return;
    if (cur.timestamp <= lastUpdateTimestamp.current) return;

    lastUpdateTimestamp.current = cur.timestamp;
    seriesRef.current.update({
      time: msToTime(cur.timestamp),
      value: cur.value,
    });
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
