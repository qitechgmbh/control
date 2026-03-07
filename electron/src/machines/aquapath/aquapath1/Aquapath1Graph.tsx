import { Page } from "@/components/Page";
import {
  MarkerProvider,
  SyncedFloatingControlPanel,
  useGraphSync,
  type GraphConfig,
} from "@/components/graph";
import { GraphWithMarkerControls } from "@/components/graph/GraphWithMarkerControls";

import React from "react";
import { useAquapath1 } from "./useAquapath";
import { TimeSeries } from "@/lib/timeseries";

export function Aquapath1GraphPage() {
  const {
    state,
    front_flow,
    back_flow,
    front_temperature,
    back_temperature,
    front_temp_reservoir,
    back_temp_reservoir,
  } = useAquapath1();

  const syncHook = useGraphSync("aquapath-group");
  const markerMachineId = "aquapath1-graphs";

  const front_temp_target =
    state?.temperature_states?.front.target_temperature ?? 0;
  const back_temp_target =
    state?.temperature_states?.back.target_temperature ?? 0;

  return (
    <Page className="pb-27">
      <MarkerProvider>
        <div className="flex flex-col gap-4">
          <FlowGraph
            syncHook={syncHook}
            flow={front_flow}
            name={"Front Flow"}
            id={"front_flow"}
            machineId={markerMachineId}
          />
          <FlowGraph
            syncHook={syncHook}
            flow={back_flow}
            name={"Back Flow"}
            id={"back_flow"}
            machineId={markerMachineId}
          />
          <TemperatureGraph
            syncHook={syncHook}
            temp_in={front_temperature}
            temp_out={front_temp_reservoir}
            targetTemp={front_temp_target}
            name={"Front Temperature"}
            id={"front_temp"}
            machineId={markerMachineId}
          />
          <TemperatureGraph
            syncHook={syncHook}
            temp_in={back_temperature}
            temp_out={back_temp_reservoir}
            targetTemp={back_temp_target}
            name={"Back Temperature"}
            id={"back_temp"}
            machineId={markerMachineId}
          />
        </div>
        <SyncedFloatingControlPanel
          controlProps={syncHook.controlProps}
          machineId={markerMachineId}
        />
      </MarkerProvider>
    </Page>
  );
}

export function FlowGraph({
  syncHook,
  flow,
  name,
  id,
  machineId,
}: {
  syncHook: ReturnType<typeof useGraphSync>;
  flow: TimeSeries | null;
  name: string;
  id: string;
  machineId: string;
}) {
  const config: GraphConfig = {
    title: name,
    icon: "lu:RotateCcw",
    colors: {
      primary: "#10b981",
      grid: "#e2e8f0",
      background: "#ffffff",
    },
    exportFilename: "flow_data",
  };

  return (
    <GraphWithMarkerControls
      syncHook={syncHook}
      newData={{
        newData: flow,
        color: "#3b82f6",
      }}
      unit="l/min"
      renderValue={(value) => value.toFixed(1)}
      config={config}
      graphId={id}
      currentTimeSeries={flow}
      machineId={machineId}
    />
  );
}
export function TemperatureGraph({
  syncHook,
  temp_in,
  temp_out,
  targetTemp,
  name,
  id,
  machineId,
}: {
  syncHook: ReturnType<typeof useGraphSync>;
  temp_in: TimeSeries | null;
  temp_out: TimeSeries | null;
  targetTemp: number;
  name: string;
  id: string;
  machineId: string;
}) {
  const config: GraphConfig = {
    title: name,
    icon: "lu:RotateCcw",
    colors: {
      primary: "#10b981",
      grid: "#e2e8f0",
      background: "#ffffff",
    },
    exportFilename: "temperature_data",
    showLegend: true,
    lines: [],
  };

  const combinedData = [
    ...(temp_in
      ? [
          {
            newData: temp_in,
            title: "Temperature In",
            color: "#3b82f6",
            lines: [
              {
                type: "threshold" as const,
                value: targetTemp,
                label: "Target Temperature",
                color: "#3b82f6",
                show: true,
                width: 2,
                dash: [5, 5],
              },
            ],
          },
        ]
      : []),

    ...(temp_out
      ? [
          {
            newData: temp_out,
            title: "Temperature Out",
            color: "#f87171",
            lines: [
              {
                type: "threshold" as const,
                value: targetTemp,
                label: "Target Temperature",
                color: "#f87171",
                show: true,
                width: 2,
                dash: [5, 5],
              },
            ],
          },
        ]
      : []),
  ];

  return (
    <GraphWithMarkerControls
      syncHook={syncHook}
      newData={combinedData}
      unit="C"
      renderValue={(value) => value.toFixed(1)}
      config={config}
      graphId={id}
      currentTimeSeries={temp_in ?? temp_out}
      machineId={machineId}
    />
  );
}
