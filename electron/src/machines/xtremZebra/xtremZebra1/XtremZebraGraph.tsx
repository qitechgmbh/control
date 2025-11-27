import { Page } from "@/components/Page";
import {
  AutoSyncedBigGraph,
  SyncedFloatingControlPanel,
  useGraphSync,
  type GraphConfig,
} from "@/components/graph";

import React from "react";
import { useXtremZebra1 } from "./useXtremZebra";

export function XtremZebraGraphPage() {
  const { currentWeight, plate1Counter, plate2Counter, plate3Counter, state } =
    useXtremZebra1();

  const syncHook = useGraphSync("weight-group");

  const weightColor = "#3b82f6";
  const counterColor = "#3b82f6";
  const plate1CounterColor = "#ef4444";
  const plate2CounterColor = "#22c55e";
  const plate3CounterColor = "#eab308";

  // Base config
  const baseConfig: GraphConfig = {
    defaultTimeWindow: 30 * 60 * 1000, // 30 minutes
    exportFilename: "xtrem_zebra_data",
    showLegend: true,
    title: "xtrem zebra data",
  };

  const plateCounterData = [
    ...(plate1Counter
      ? [
          {
            newData: plate1Counter,
            title: "Plate 1 Counter",
            color: plate1CounterColor,
          },
        ]
      : []),
    ...(plate2Counter
      ? [
          {
            newData: plate2Counter,
            title: "Plate 2 Counter",
            color: plate2CounterColor,
          },
        ]
      : []),
    ...(plate3Counter
      ? [
          {
            newData: plate3Counter,
            title: "Plate 3 Counter",
            color: plate3CounterColor,
          },
        ]
      : []),
  ];

  const weightConfig: GraphConfig = {
    title: "Weight",
    defaultTimeWindow: 30 * 60 * 1000, // 30 minutes
    exportFilename: "weight_data",
    colors: {
      primary: weightColor,
      grid: "#e2e8f0",
      axis: "#64748b",
      background: "#ffffff",
    },
  };

  const counterConfig: GraphConfig = {
    title: "Counter",
    defaultTimeWindow: 30 * 60 * 1000, // 30 minutes
    exportFilename: "counter_data",
    colors: {
      primary: counterColor,
      grid: "#e2e8f0",
      axis: "#64748b",
      background: "#ffffff",
    },
  };

  return (
    <Page className="pb-27">
      <div className="flex flex-col gap-4">
        <AutoSyncedBigGraph
          syncHook={syncHook}
          newData={plateCounterData}
          unit="pcs"
          renderValue={(value) => value.toFixed(3)}
          config={counterConfig}
          graphId="counter-graph"
        />
        <AutoSyncedBigGraph
          syncHook={syncHook}
          newData={{
            newData: currentWeight,
            color: weightColor,
          }}
          unit="kg"
          renderValue={(value) => value.toFixed(3)}
          config={weightConfig}
          graphId="weight-graph"
        />
      </div>
      <SyncedFloatingControlPanel controlProps={syncHook.controlProps} />
    </Page>
  );
}
