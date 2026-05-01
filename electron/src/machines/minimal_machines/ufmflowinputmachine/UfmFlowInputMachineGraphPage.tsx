import { Page } from "@/components/Page";
import {
  MarkerProvider,
  SyncedFloatingControlPanel,
  useGraphSync,
  type GraphConfig,
} from "@/components/graph";
import { GraphWithMarkerControls } from "@/components/graph/marker/GraphWithMarkerControls";
import React from "react";
import { useUfmFlowInputMachine } from "./useUfmFlowInputMachine";

export function UfmFlowInputMachineGraphPage() {
  const { flowLph } = useUfmFlowInputMachine();

  const syncHook = useGraphSync("ufm-flow-input-graphs");

  const baseGraphConfig: GraphConfig = {
    title: "",
    defaultTimeWindow: 30 * 60 * 1000, // 30 minutes
    colors: {
      grid: "#e2e8f0",
      axis: "#64748b",
      background: "#ffffff",
    },
  };

  const flowGraphData = [
    {
      newData: flowLph,
      color: "#3b82f6",
      title: "Flow Rate",
      unit: "l/h",
    },
  ];

  return (
    <MarkerProvider machineId="ufm-flow-input-graphs">
      <Page>
        <SyncedFloatingControlPanel syncHook={syncHook}>
          {flowGraphData.map((graph, index) => (
            <GraphWithMarkerControls
              key={index}
              newData={graph.newData}
              unit={graph.unit}
              config={{
                ...baseGraphConfig,
                title: graph.title,
              }}
              graphId={`flow-${index}`}
              syncGraph={syncHook}
              renderValue={(value) => value.toFixed(2)}
            />
          ))}
        </SyncedFloatingControlPanel>
      </Page>
    </MarkerProvider>
  );
}
