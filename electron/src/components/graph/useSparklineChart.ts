import { useEffect, useRef } from "react";
import { TimeSeries } from "@/lib/timeseries";
import { SparklineChart } from "./SparklineChart";

type UseSparklineChartConfig = {
  newData: TimeSeries | null;
  width: number;
  renderValue?: (value: number) => string;
};

/**
 * Thin React binding over SparklineChart: constructs the chart once real
 * data is available, then pushes prop/data changes into it via method
 * calls instead of duplicating chart logic in this hook.
 */
export function useSparklineChart(config: UseSparklineChartConfig) {
  const canvasRef = useRef<HTMLCanvasElement | null>(null);
  const chartRef = useRef<SparklineChart | null>(null);

  useEffect(() => {
    if (
      !canvasRef.current ||
      !config.newData?.short?.timeWindow ||
      chartRef.current
    ) {
      return;
    }

    chartRef.current = new SparklineChart(canvasRef.current, config.newData, {
      width: config.width,
      renderValue: config.renderValue,
    });

    return () => {
      chartRef.current?.destroy();
      chartRef.current = null;
    };
    // Intentionally created once the first real data arrives, and never
    // recreated on width/renderValue changes — those are pushed via the
    // resize()/setRenderValue() effects below instead.
  }, [config.newData?.short?.timeWindow]);

  useEffect(() => {
    chartRef.current?.setRenderValue(config.renderValue);
  }, [config.renderValue]);

  useEffect(() => {
    chartRef.current?.resize(config.width);
  }, [config.width]);

  useEffect(() => {
    if (config.newData) {
      chartRef.current?.pushLatest(config.newData);
    }
  }, [config.newData?.current?.timestamp]);

  return { canvasRef };
}
