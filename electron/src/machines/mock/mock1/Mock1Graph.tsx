import { Page } from "@/components/Page";
import {
  AutoSyncedBigGraph,
  SyncedFloatingControlPanel,
  useGraphSync,
  type GraphConfig,
} from "@/components/graph";
import React from "react";
import { useMock1 } from "./useMock";

export function Mock1GraphPage() {
  const { sineWave } = useMock1();

  const syncHook = useGraphSync(30 * 60 * 1000, "mock-graphs");

  const config: GraphConfig = {
    title: "Sine Wave",
    defaultTimeWindow: 30 * 60 * 1000,
    exportFilename: "sine_wave_data",
    showLegend: true,
    // Add lines configuration
    lines: [
      {
        value: 0.5,
        label: "Upper Threshold",
        color: "#ef4444", // Red
        type: "threshold",
        show: true,
      },
      {
        value: -0.5,
        label: "Lower Threshold",
        color: "#ef4444", // Red
        type: "threshold",
        show: true,
      },
      {
        value: 0,
        label: "Target Line",
        color: "#10b981", // Green
        type: "target",
        show: true,
      },
      {
        value: 0.8,
        label: "Critical Level",
        color: "#f59e0b", // Orange
        show: true,
        type: "threshold",
      },
    ],
  };

  if (!sineWave) {
    return <Page className="pb-25">Loading...</Page>;
  }

  // Combined data for 3 sine waves
  const combinedData = [
    { newData: sineWave, title: "Sine Wave 1", color: "#3b82f6" },
    { newData: sineWave, title: "Sine Wave 2", color: "#ef4444" },
    { newData: sineWave, title: "Sine Wave 3", color: "#10b981" },
  ];

  // Single sine wave data with different lines
  const singleData = { newData: sineWave };

  const singleGraphConfig: GraphConfig = {
    ...config,
    title: "Sine Wave 4",
    lines: [
      {
        value: 0.3,
        label: "Safe Zone Upper",
        color: "#3b82f6", // Blue
        type: "threshold",
        show: true,
        strokeWidth: 2,
        strokeDashArray: "3,3",
      },
      {
        value: -0.3,
        label: "Safe Zone Lower",
        color: "#3b82f6", // Blue
        type: "threshold",
        show: true,
        strokeWidth: 2,
        strokeDashArray: "3,3",
      },
    ],
  };

  return (
    <Page className="pb-25">
      <div className="flex flex-col gap-4">
        <AutoSyncedBigGraph
          syncHook={syncHook}
          newData={combinedData}
          config={{ ...config, title: "Combined Sine Waves (1, 2, 3)" }}
          unit={"mm"}
          renderValue={(value) => value.toFixed(3)}
          graphId="combined-graph"
        />

        <AutoSyncedBigGraph
          syncHook={syncHook}
          newData={singleData}
          config={singleGraphConfig}
          unit={"mm"}
          renderValue={(value) => value.toFixed(3)}
          graphId="single-graph"
        />
      </div>

      <SyncedFloatingControlPanel controlProps={syncHook.controlProps} />
    </Page>
  );
}
