import { Page } from "@/components/Page";
import {
  BigGraph,
  GraphConfig,
  GraphSyncProvider,
  FloatingControlPanel,
  GraphControls,
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
    <Page className="pb-20">
      <GraphSyncProvider groupId="main-dashboard">
        <div className="flex flex-col gap-4">
          <GraphControls groupId="mock" />
          <BigGraph
            newData={sineWave}
            config={{ ...config, title: "Sine Wave (Original)" }}
            unit={"mm"}
            renderValue={(value) => value.toFixed(3)}
            syncGroupId="mock"
            graphId="0-graph"
          />
          <BigGraph
            newData={sineWave}
            config={{ ...config, title: "Sine Wave (+0.1)" }}
            unit={"mm"}
            renderValue={(value) => value.toFixed(3)}
            syncGroupId="mock"
            graphId="1-graph"
          />
          <BigGraph
            newData={sineWave}
            config={{ ...config, title: "Sine Wave (+0.2)" }}
            unit={"mm"}
            renderValue={(value) => value.toFixed(3)}
            syncGroupId="mock"
            graphId="2-graph"
          />
          <BigGraph
            newData={sineWave}
            config={{ ...config, title: "Sine Wave (+0.3)" }}
            unit={"mm"}
            renderValue={(value) => value.toFixed(3)}
            syncGroupId="mock"
            graphId="3-graph"
          />
        </div>
        <FloatingControlPanel groupId="mock" />
      </GraphSyncProvider>
    </Page>
  );
}
