import { Page } from "@/components/Page";
import { BigGraph, GraphConfig } from "@/helpers/BigGraph";
import React from "react";
import { useDre1 } from "./useDre";

export function Dre1GraphsPage() {
  const { dreDiameter, dreState } = useDre1();

  // Controlled local states synced with dreState
  const targetDiameter = dreState?.data?.target_diameter ?? 0;
  const lowerTolerance = dreState?.data?.lower_tolerance ?? 0;
  const higherTolerance = dreState?.data?.higher_tolerance ?? 0;

  const config: GraphConfig = {
    title: "Diameter",
    description: "Real-time diameter measurements with thresholds",
    defaultTimeWindow: 30 * 60 * 1000, // 30 minute
    exportFilename: "diameter_data",
    lines: [
      {
        type: "threshold",
        value: targetDiameter + higherTolerance,
        label: "Upper Threshold",
        color: "#ef4444",
        dash: [5, 5],
      },
      {
        type: "threshold",
        value: targetDiameter - lowerTolerance,
        label: "Lower Threshold",
        color: "#f97316",
        dash: [5, 5],
      },
      {
        type: "target",
        value: targetDiameter,
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
  };

  return (
    <Page>
      <BigGraph
        newData={dreDiameter}
        unit="mm"
        renderValue={(value) => value.toFixed(3)}
        config={config}
      />
    </Page>
  );
}
