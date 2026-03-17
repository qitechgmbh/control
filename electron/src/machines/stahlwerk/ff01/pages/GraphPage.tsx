import { Page } from "@/components/Page";
import {
  AutoSyncedBigGraph,
  SyncedFloatingControlPanel,
  useGraphSync,
  type GraphConfig,
} from "@/components/graph";

import React from "react";
import { useFF01_v1 } from "../use";

export function GraphPage() {
  const { weightPrev } = useFF01_v1();

  const syncHook = useGraphSync("weight-group");

  const weightColor = "#3b82f6";
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

  return (
    <Page className="pb-27">
      <div className="flex flex-col gap-4">
        <AutoSyncedBigGraph
          syncHook={syncHook}
          newData={{
            newData: weightPrev,
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
