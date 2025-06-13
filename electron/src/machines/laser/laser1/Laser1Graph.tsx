import { Page } from "@/components/Page";
import { BigGraph, GraphConfig } from "@/helpers/BigGraph";
import React from "react";
import { useLaser1 } from "./useLaser1";

export function Laser1GraphsPage() {
  const { laserDiameter, laserState } = useLaser1();

  const targetDiameter = laserState?.data?.target_diameter ?? 0;
  const lowerTolerance = laserState?.data?.lower_tolerance ?? 0;
  const higherTolerance = laserState?.data?.higher_tolerance ?? 0;

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
        newData={laserDiameter}
        unit="mm"
        renderValue={(value) => value.toFixed(3)}
        config={config}
        syncGroupId="diameter-group"
        graphId="diameter-main"
      />
    </Page>
  );
}
