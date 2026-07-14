import React from "react";
import { TimeSeries } from "@/lib/timeseries";
import { SPARKLINE_HEIGHT, SparklineRange } from "./SparklineChart";
import { useSparklineChart } from "./useSparklineChart";

type MiniGraphProps = {
  newData: TimeSeries | null;
  width: number;
  /**
   * Fixed Y-axis bounds (e.g. a sensor's physical 0-to-rated-max). When
   * omitted, the graph auto-scales to the visible data as before.
   */
  range?: SparklineRange;
};

export function MiniGraph({ newData, width, range }: MiniGraphProps) {
  const { canvasRef } = useSparklineChart({
    newData,
    width,
    range,
  });

  return (
    <div
      style={{
        width: "100%",
        height: SPARKLINE_HEIGHT,
        overflow: "hidden",
      }}
    >
      <canvas ref={canvasRef} />
    </div>
  );
}
