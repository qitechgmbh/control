import { Page } from "@/components/Page";
import {
  MarkerProvider,
  SyncedFloatingControlPanel,
  useGraphSync,
  type GraphConfig,
} from "@/components/graph";
import { GraphWithMarkerControls } from "@/components/graph/marker/GraphWithMarkerControls";
import React from "react";
import { useMixerV1 } from "./useMixerV1";

export function MixerV1GraphsPage() {
  const { hopperARpm, hopperBRpm } = useMixerV1();

  const syncHook = useGraphSync("mixer-graphs");

  const config: GraphConfig = {
    title: "Hopper RPM",
    defaultTimeWindow: 30 * 60 * 1000,
    exportFilename: "mixer_hopper_rpm",
    showLegend: true,
  };

  const rpmData = [
    {
      newData: hopperARpm,
      title: "Hopper A",
      color: "#3b82f6",
    },
    {
      newData: hopperBRpm,
      title: "Hopper B",
      color: "#f59e0b",
    },
  ];

  return (
    <Page className="pb-27">
      <MarkerProvider>
        <div className="flex flex-col gap-4">
          <GraphWithMarkerControls
            syncHook={syncHook}
            newData={rpmData}
            config={config}
            unit="rpm"
            renderValue={(value) => value.toFixed(1)}
            graphId="hopper-rpm-graph"
            currentTimeSeries={hopperARpm}
            machineId="mixer-graphs"
          />
        </div>

        <SyncedFloatingControlPanel
          controlProps={syncHook.controlProps}
          machineId="mixer-graphs"
        />
      </MarkerProvider>
    </Page>
  );
}
