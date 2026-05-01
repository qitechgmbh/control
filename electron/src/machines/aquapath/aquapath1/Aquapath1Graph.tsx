import { Page } from "@/components/Page";
import {
  AutoSyncedBigGraph,
  SyncedFloatingControlPanel,
  useGraphSync,
  type GraphConfig,
} from "@/components/graph";

import React from "react";
import { useAquapath1, useAquapathBase } from "./useAquapath";
import { TimeSeries } from "@/lib/timeseries";

export function Aquapath1GraphPage({
  useHook = useAquapath1,
}: {
  useHook?: () => ReturnType<typeof useAquapathBase>;
} = {}) {
  const {
    state,
    front_flow,
    back_flow,
    front_temperature,
    back_temperature,
    front_temp_reservoir,
    back_temp_reservoir,
    front_power,
    back_power,
    front_total_energy,
    back_total_energy,
    targetFrontTemperature,
    targetBackTemperature,
  } = useHook();

  const syncHook = useGraphSync("aquapath-group");

  const reservoir1TempTarget =
    state?.temperature_states?.back.target_temperature ?? 0;
  const reservoir2TempTarget =
    state?.temperature_states?.front.target_temperature ?? 0;

  return (
    <Page className="pb-27">
      <div className="flex flex-col gap-4">
        <FlowGraph
          syncHook={syncHook}
          flow={back_flow}
          name={"Reservoir 1 (Back) Flow"}
          id={"reservoir_1_flow"}
        />
        <FlowGraph
          syncHook={syncHook}
          flow={front_flow}
          name={"Reservoir 2 (Front) Flow"}
          id={"reservoir_2_flow"}
        />
        <TemperatureGraph
          syncHook={syncHook}
          temp_in={back_temperature}
          temp_out={back_temp_reservoir}
          targetTemp={reservoir1TempTarget}
          targetSeries={targetBackTemperature}
          name={"Reservoir 1 (Back) Temperature"}
          id={"reservoir_1_temp"}
        />
        <TemperatureGraph
          syncHook={syncHook}
          temp_in={front_temperature}
          temp_out={front_temp_reservoir}
          targetTemp={reservoir2TempTarget}
          targetSeries={targetFrontTemperature}
          name={"Reservoir 2 (Front) Temperature"}
          id={"reservoir_2_temp"}
        />
        <PowerGraph
          syncHook={syncHook}
          reservoir1Power={back_power}
          reservoir2Power={front_power}
          id={"aquapath_power"}
        />
        <EnergyGraph
          syncHook={syncHook}
          reservoir1Energy={scaleTimeSeries(back_total_energy, 1 / 1000)}
          reservoir2Energy={scaleTimeSeries(front_total_energy, 1 / 1000)}
          id={"aquapath_energy"}
        />
      </div>
      <SyncedFloatingControlPanel controlProps={syncHook.controlProps} />
    </Page>
  );
}

function scaleTimeSeries(
  series: TimeSeries | null,
  factor: number,
): TimeSeries | null {
  if (!series) {
    return null;
  }

  return {
    current: series.current
      ? {
          ...series.current,
          value: series.current.value * factor,
        }
      : null,
    short: {
      ...series.short,
      values: series.short.values.map((entry) =>
        entry
          ? {
              ...entry,
              value: entry.value * factor,
            }
          : entry,
      ),
    },
    long: {
      ...series.long,
      values: series.long.values.map((entry) =>
        entry
          ? {
              ...entry,
              value: entry.value * factor,
            }
          : entry,
      ),
    },
  };
}

export function FlowGraph({
  syncHook,
  flow,
  name,
  id,
}: {
  syncHook: ReturnType<typeof useGraphSync>;
  flow: TimeSeries | null;
  name: string;
  id: string;
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
    <AutoSyncedBigGraph
      syncHook={syncHook}
      newData={{
        newData: flow,
        color: "#3b82f6",
      }}
      unit="l/min"
      renderValue={(value) => value.toFixed(1)}
      config={config}
      graphId={id}
    />
  );
}
export function TemperatureGraph({
  syncHook,
  temp_in,
  temp_out,
  targetTemp,
  targetSeries,
  name,
  id,
}: {
  syncHook: ReturnType<typeof useGraphSync>;
  temp_in: TimeSeries | null;
  temp_out: TimeSeries | null;
  targetTemp: number;
  targetSeries: TimeSeries | null;
  name: string;
  id: string;
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
                type: "target" as const,
                value: targetTemp,
                label: "Target Temperature",
                targetSeries: targetSeries ?? undefined,
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
                type: "target" as const,
                value: targetTemp,
                label: "Target Temperature",
                targetSeries: targetSeries ?? undefined,
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
    <AutoSyncedBigGraph
      syncHook={syncHook}
      newData={combinedData}
      unit="C"
      renderValue={(value) => value.toFixed(1)}
      config={config}
      graphId={id}
    />
  );
}

export function PowerGraph({
  syncHook,
  reservoir1Power,
  reservoir2Power,
  id,
}: {
  syncHook: ReturnType<typeof useGraphSync>;
  reservoir1Power: TimeSeries | null;
  reservoir2Power: TimeSeries | null;
  id: string;
}) {
  const config: GraphConfig = {
    title: "Heating Wattage",
    colors: {
      primary: "#f97316",
      grid: "#e2e8f0",
      background: "#ffffff",
    },
    exportFilename: "heating_power_data",
    showLegend: true,
  };

  const powerData = [
    ...(reservoir1Power
      ? [
          {
            newData: reservoir1Power,
            title: "Reservoir 1 (Back)",
            color: "#f97316",
          },
        ]
      : []),
    ...(reservoir2Power
      ? [
          {
            newData: reservoir2Power,
            title: "Reservoir 2 (Front)",
            color: "#ef4444",
          },
        ]
      : []),
  ];

  return (
    <AutoSyncedBigGraph
      syncHook={syncHook}
      newData={powerData}
      unit="W"
      renderValue={(value) => value.toFixed(0)}
      config={config}
      graphId={id}
    />
  );
}

export function EnergyGraph({
  syncHook,
  reservoir1Energy,
  reservoir2Energy,
  id,
}: {
  syncHook: ReturnType<typeof useGraphSync>;
  reservoir1Energy: TimeSeries | null;
  reservoir2Energy: TimeSeries | null;
  id: string;
}) {
  const config: GraphConfig = {
    title: "Heating Energy",
    colors: {
      primary: "#0f766e",
      grid: "#e2e8f0",
      background: "#ffffff",
    },
    exportFilename: "heating_energy_data",
    showLegend: true,
  };

  const energyData = [
    ...(reservoir1Energy
      ? [
          {
            newData: reservoir1Energy,
            title: "Reservoir 1 (Back)",
            color: "#0f766e",
          },
        ]
      : []),
    ...(reservoir2Energy
      ? [
          {
            newData: reservoir2Energy,
            title: "Reservoir 2 (Front)",
            color: "#14b8a6",
          },
        ]
      : []),
  ];

  return (
    <AutoSyncedBigGraph
      syncHook={syncHook}
      newData={energyData}
      unit="kWh"
      renderValue={(value) => value.toFixed(3)}
      config={config}
      graphId={id}
    />
  );
}
