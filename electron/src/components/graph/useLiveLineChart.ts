/* eslint-disable react-compiler/react-compiler */
import { useEffect, useRef, useState } from "react";
import type { Chart } from "chart.js";
import { LiveLineChart, LiveLineChartConfig, LiveLineChartSnapshot } from "./LiveLineChart";
import { getPrimarySeriesData } from "./graphDataUtils";

/**
 * Thin React binding over LiveLineChart: constructs the chart once real
 * series config is available, then pushes prop/data changes into it via
 * updateData() instead of duplicating chart logic here.
 */
export function useLiveLineChart(
  config: LiveLineChartConfig,
  chartRefOut?: React.MutableRefObject<Chart | null>,
) {
  const canvasRef = useRef<HTMLCanvasElement | null>(null);
  const chartRef = useRef<LiveLineChart | null>(null);
  const [snapshot, setSnapshot] = useState<LiveLineChartSnapshot | null>(null);

  const primaryTimeWindow = getPrimarySeriesData(config.newData)?.long
    ?.timeWindow;

  useEffect(() => {
    if (!canvasRef.current || !primaryTimeWindow || chartRef.current) {
      return;
    }

    const chart = new LiveLineChart(canvasRef.current, config);
    chartRef.current = chart;
    if (chartRefOut) chartRefOut.current = chart.chartInstance;

    setSnapshot(chart.getSnapshot());
    const unsubscribe = chart.subscribe(setSnapshot);

    return () => {
      unsubscribe();
      chart.destroy();
      chartRef.current = null;
      if (chartRefOut) chartRefOut.current = null;
    };
    // Constructed once real series config is available; later prop changes
    // are pushed via updateData()/toggleSeries() below, never recreated.
  }, [primaryTimeWindow]);

  useEffect(() => {
    chartRef.current?.updateData(config);
  }, [
    config.newData,
    config.config,
    config.colors,
    config.renderValue,
    config.selectedTimeWindow,
    config.visibleSeries,
    config.markers,
  ]);

  return { canvasRef, snapshot };
}
