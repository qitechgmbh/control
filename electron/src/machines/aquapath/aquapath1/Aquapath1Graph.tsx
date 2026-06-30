import { Page } from "@/components/Page";
import {
  AutoSyncedBigGraph,
  SyncedFloatingControlPanel,
  useGraphSync,
  type GraphConfig,
} from "@/components/graph";

import React from "react";
import { useAquapath1 } from "./useAquapath";
import { TimeSeries } from "@/lib/timeseries";

export function Aquapath1GraphPage() {
  const {
    state,
    left_flow,
    right_flow,
    left_temperature,
    right_temperature,
    left_power,
    right_power,
    left_total_energy,
    right_total_energy,
    targetLeftTemperature,
    targetRightTemperature,
  } = useAquapath1();

  const syncHook = useGraphSync("aquapath-group");

  const rightReservoirTempTarget =
    state?.temperature_states?.right.target_temperature ?? 0;
  const leftReservoirTempTarget =
    state?.temperature_states?.left.target_temperature ?? 0;

  return (
    <Page className="pb-27">
      <div className="flex flex-col gap-4">
        <FlowGraph
          syncHook={syncHook}
          flow={right_flow}
          name={"Right Reservoir Flow"}
          id={"right_reservoir_flow"}
        />
        <FlowGraph
          syncHook={syncHook}
          flow={left_flow}
          name={"Left Reservoir Flow"}
          id={"left_reservoir_flow"}
        />
        <TemperatureGraph
          syncHook={syncHook}
          temperature={right_temperature}
          targetTemp={rightReservoirTempTarget}
          targetSeries={targetRightTemperature}
          name={"Right Reservoir Temperature"}
          id={"right_reservoir_temp"}
        />
        <TemperatureGraph
          syncHook={syncHook}
          temperature={left_temperature}
          targetTemp={leftReservoirTempTarget}
          targetSeries={targetLeftTemperature}
          name={"Left Reservoir Temperature"}
          id={"left_reservoir_temp"}
        />
        <PowerGraph
          syncHook={syncHook}
          rightReservoirPower={right_power}
          leftReservoirPower={left_power}
          id={"aquapath_power"}
        />
        <EnergyGraph
          syncHook={syncHook}
          rightReservoirEnergy={scaleTimeSeries(right_total_energy, 1 / 1000)}
          leftReservoirEnergy={scaleTimeSeries(left_total_energy, 1 / 1000)}
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
  temperature,
  targetTemp,
  targetSeries,
  name,
  id,
}: {
  syncHook: ReturnType<typeof useGraphSync>;
  temperature: TimeSeries | null;
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

  const combinedData = temperature
    ? [
        {
          newData: temperature,
          title: "Temperature",
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
    : [];

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
  rightReservoirPower,
  leftReservoirPower,
  id,
}: {
  syncHook: ReturnType<typeof useGraphSync>;
  rightReservoirPower: TimeSeries | null;
  leftReservoirPower: TimeSeries | null;
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
    ...(rightReservoirPower
      ? [
          {
            newData: rightReservoirPower,
            title: "Right Reservoir",
            color: "#f97316",
          },
        ]
      : []),
    ...(leftReservoirPower
      ? [
          {
            newData: leftReservoirPower,
            title: "Left Reservoir",
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
  rightReservoirEnergy,
  leftReservoirEnergy,
  id,
}: {
  syncHook: ReturnType<typeof useGraphSync>;
  rightReservoirEnergy: TimeSeries | null;
  leftReservoirEnergy: TimeSeries | null;
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
    ...(rightReservoirEnergy
      ? [
          {
            newData: rightReservoirEnergy,
            title: "Right Reservoir",
            color: "#0f766e",
          },
        ]
      : []),
    ...(leftReservoirEnergy
      ? [
          {
            newData: leftReservoirEnergy,
            title: "Left Reservoir",
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
