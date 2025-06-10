import { Page } from "@/components/Page";
import { BigGraph, GraphConfig, GraphSyncProvider } from "@/helpers/BigGraph";
import React from "react";
import { useMock1 } from "./useMock";

export function Mock1GraphPage() {
  const { sineWave } = useMock1();

  const config: GraphConfig = {
    title: "Sine Wave",
    defaultTimeWindow: 30 * 60 * 1000, // 30 minute
    exportFilename: "sine_wave_data",
  };

  return (
    <Page>
      <GraphSyncProvider groupId="main-dashboard">
        <div className="grid gap-4">
          <BigGraph
            newData={sineWave}
            config={config}
            unit={"mm"}
            renderValue={(value) => value.toFixed(3)}
            syncGroupId="main-dashboard"
            graphId="diameter-graph"
          />
          <BigGraph
            newData={sineWave}
            config={config}
            unit={"mm"}
            syncGroupId="main-dashboard"
            graphId="1-graph"
          />
          <BigGraph
            newData={sineWave}
            config={config}
            unit={"mm"}
            syncGroupId="main-dashboard"
            graphId="2-graph"
          />
          <BigGraph
            newData={sineWave}
            config={config}
            unit={"mm"}
            syncGroupId="main-dashboard"
            graphId="3-graph"
          />
        </div>
      </GraphSyncProvider>
    </Page>
  );
}
