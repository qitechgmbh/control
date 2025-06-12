import { Page } from "@/components/Page";
import {
  BigGraph,
  GraphConfig,
  GraphSyncProvider,
  FloatingExportButton,
} from "@/helpers/BigGraph";
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
            syncGroupId="mock"
            graphId="0-graph"
          />
          <BigGraph
            newData={sineWave}
            config={config}
            unit={"mm"}
            syncGroupId="mock"
            graphId="1-graph"
          />
          <BigGraph
            newData={sineWave}
            config={config}
            unit={"mm"}
            syncGroupId="mock"
            graphId="2-graph"
          />
          <BigGraph
            newData={sineWave}
            config={config}
            unit={"mm"}
            syncGroupId="mock"
            graphId="3-graph"
          />
        </div>
        <FloatingExportButton groupId="mock" />
      </GraphSyncProvider>
    </Page>
  );
}
