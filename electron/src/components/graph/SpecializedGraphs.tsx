import React from "react";
import { BigGraph } from "./BigGraph";
import { TimeSeries } from "@/lib/timeseries";
import { Unit } from "@/control/units";
import { GraphConfig, PropGraphSync } from "./types";

export function DiameterGraph({
  newData,
  threshold1,
  threshold2,
  target,
  unit,
  renderValue,
  syncGraph,
}: {
  newData: TimeSeries | null;
  threshold1: number;
  threshold2: number;
  target: number;
  unit?: Unit;
  renderValue?: (value: number) => string;
  syncGraph?: PropGraphSync;
}) {
  const config: GraphConfig = {
    title: "Diameter",
    lines: [
      {
        type: "threshold",
        value: threshold1,
        label: "Upper Threshold",
        color: "#ef4444",
        dash: [5, 5],
      },
      {
        type: "threshold",
        value: threshold2,
        label: "Lower Threshold",
        color: "#f97316",
        dash: [5, 5],
      },
      {
        type: "target",
        value: target,
        label: "Target",
        color: "#6b7280",
      },
    ],
    colors: {
      primary: "#3b82f6",
      grid: "#e2e8f0",
      axis: "#64748b",
      background: "#ffffff",
    },
    exportFilename: "diameter_data",
  };

  return (
    <BigGraph
      newData={newData}
      unit={unit}
      renderValue={renderValue}
      config={config}
      graphId="diameter-main"
      syncGraph={syncGraph}
    />
  );
}
