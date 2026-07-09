import React from "react";
import { TimeSeries } from "@/lib/timeseries";
import { SPARKLINE_HEIGHT } from "./SparklineChart";
import { useSparklineChart } from "./useSparklineChart";

type MiniGraphProps = {
  newData: TimeSeries | null;
  width: number;
  renderValue?: (value: number) => string;
};

export function MiniGraph({ newData, width, renderValue }: MiniGraphProps) {
  const { canvasRef } = useSparklineChart({ newData, width, renderValue });

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
