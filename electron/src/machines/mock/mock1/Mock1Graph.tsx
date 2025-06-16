import { Page } from "@/components/Page";
import {
  AutoSyncedBigGraph,
  SyncedFloatingControlPanel,
  SyncedGraphControls,
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
  };

  return (
    <Page className="pb-20">
      <div className="flex flex-col gap-4">
        <SyncedGraphControls controlProps={syncHook.controlProps} />

        <AutoSyncedBigGraph
          syncHook={syncHook}
          newData={sineWave}
          config={{ ...config, title: "Sine Wave 1" }}
          unit={"mm"}
          renderValue={(value) => value.toFixed(3)}
          graphId="0-graph"
        />

        <AutoSyncedBigGraph
          syncHook={syncHook}
          newData={sineWave}
          config={{ ...config, title: "Sine Wave 2" }}
          unit={"mm"}
          renderValue={(value) => value.toFixed(3)}
          graphId="1-graph"
        />

        <AutoSyncedBigGraph
          syncHook={syncHook}
          newData={sineWave}
          config={{ ...config, title: "Sine Wave 3" }}
          unit={"mm"}
          renderValue={(value) => value.toFixed(3)}
          graphId="2-graph"
        />

        <AutoSyncedBigGraph
          syncHook={syncHook}
          newData={sineWave}
          config={{ ...config, title: "Sine Wave 4" }}
          unit={"mm"}
          renderValue={(value) => value.toFixed(3)}
          graphId="3-graph"
        />
      </div>

      <SyncedFloatingControlPanel controlProps={syncHook.controlProps} />
    </Page>
  );
}
