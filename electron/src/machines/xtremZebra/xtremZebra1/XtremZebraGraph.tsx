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
  const { current_weight, state } = useXtremZebra1();

  const syncHook = useGraphSync("weight-group");

  const weightColor = "#3b82f6";

  const config: GraphConfig = {
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
            newData: current_weight,
            color: weightColor,
          }}
          unit="mm"
          renderValue={(value) => value.toFixed(3)}
          config={config}
          graphId="weight-graph"
        />
      </div>
      <SyncedFloatingControlPanel controlProps={syncHook.controlProps} />
    </Page>
  );
}
