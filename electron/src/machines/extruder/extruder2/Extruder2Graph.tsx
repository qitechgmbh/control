import { Page } from "@/components/Page";
import {
  AutoSyncedBigGraph,
  SyncedFloatingControlPanel,
  useGraphSync,
  type GraphConfig,
} from "@/components/graph";
import React from "react";
import { useExtruder2 } from "./useExtruder";

export function Extruder2GraphsPage() {
  const {
    nozzleHeatingState,
    nozzleTemperature,
    nozzlePower,

    frontHeatingState,
    frontTemperature,
    frontPower,

    backHeatingState,
    backTemperature,
    backPower,

    middleHeatingState,
    middleTemperature,
    middlePower,

    uses_rpm,
    bar,
    rpm,
    targetBar,
    targetRpm,
  } = useExtruder2();

  const syncHook = useGraphSync(30 * 60 * 1000, "extruder-graphs");

  // Base config
  const baseConfig: GraphConfig = {
    defaultTimeWindow: 30 * 60 * 1000, // 30 minutes
    exportFilename: "extruder_data",
    showLegend: true,
    title: "extruder data",
  };

  // Combined Temperature Graph (nozzle, front, back)
  const temperatureData = [
    {
      newData: nozzleTemperature,
      title: "Nozzle",
      color: "#ef4444",
    },
    {
      newData: frontTemperature,
      title: "Front",
      color: "#f59e0b",
    },
    {
      newData: backTemperature,
      title: "Back",
      color: "#3b82f6",
    },
  ].filter((item) => item.newData); // Filter out any null/undefined data

  const temperatureConfig: GraphConfig = {
    ...baseConfig,
    title: "Temperatures (Nozzle, Front, Back)",
    exportFilename: "temperatures_data",
    lines: [
      // Target temperature lines
      ...(nozzleHeatingState?.target_temperature !== undefined
        ? [
            {
              type: "target" as const,
              value: nozzleHeatingState.target_temperature,
              label: "Nozzle Target",
              color: "#ef4444",
              show: true,
            },
          ]
        : []),
      ...(frontHeatingState?.target_temperature !== undefined
        ? [
            {
              type: "target" as const,
              value: frontHeatingState.target_temperature,
              label: "Front Target",
              color: "#f59e0b",
              show: true,
            },
          ]
        : []),
      ...(backHeatingState?.target_temperature !== undefined
        ? [
            {
              type: "target" as const,
              value: backHeatingState.target_temperature,
              label: "Back Target",
              color: "#3b82f6",
              show: true,
            },
          ]
        : []),
      // Safety threshold
      {
        type: "threshold",
        value: 250,
        label: "Safety Limit",
        color: "#dc2626",
      },
    ],
    colors: {
      primary: "#ef4444",
      grid: "#e2e8f0",
      axis: "#64748b",
      background: "#ffffff",
    },
  };

  // Combined Power Graph (nozzle, front, back)
  const powerData = [
    {
      newData: nozzlePower,
      title: "Nozzle",
      color: "#ef4444",
    },
    {
      newData: frontPower,
      title: "Front",
      color: "#f59e0b",
    },
    {
      newData: backPower,
      title: "Back",
      color: "#3b82f6",
    },
  ].filter((item) => item.newData); // Filter out any null/undefined data

  const powerConfig: GraphConfig = {
    ...baseConfig,
    title: "Power Outputs (Nozzle, Front, Back)",
    exportFilename: "power_data",
    colors: {
      primary: "#10b981",
      grid: "#e2e8f0",
      axis: "#64748b",
      background: "#ffffff",
    },
  };

  // Pressure Graph (single with target line)
  const pressureConfig: GraphConfig = {
    ...baseConfig,
    title: "Pressure",
    exportFilename: "pressure_data",
    lines: [
      ...(targetBar !== undefined
        ? [
            {
              type: "target" as const,
              value: targetBar,
              label: "Target Pressure",
              color: "#6b7280",
              show: true,
            },
          ]
        : []),
      {
        type: "threshold",
        value: 50,
        label: "Maximum Pressure",
        color: "#ef4444",
      },
    ],
    colors: {
      primary: "#3b82f6",
      grid: "#e2e8f0",
      axis: "#64748b",
      background: "#ffffff",
    },
  };

  // RPM Graph (single with target line)
  const rpmConfig: GraphConfig = {
    ...baseConfig,
    title: "RPM",
    exportFilename: "rpm_data",
    lines: [
      ...(targetRpm !== undefined
        ? [
            {
              type: "target" as const,
              value: targetRpm,
              label: "Target RPM",
              color: "#6b7280",
              show: true,
            },
          ]
        : []),
      {
        type: "threshold",
        value: 1000,
        label: "Maximum RPM",
        color: "#ef4444",
      },
    ],
    colors: {
      primary: "#8b5cf6",
      grid: "#e2e8f0",
      axis: "#64748b",
      background: "#ffffff",
    },
  };

  return (
    <Page className="pb-25">
      <div className="flex flex-col gap-4">
        {/* 1. Combined Temperature Graph (nozzle, front, back) */}
        {temperatureData.length > 0 && (
          <AutoSyncedBigGraph
            syncHook={syncHook}
            newData={temperatureData}
            config={temperatureConfig}
            unit="deg"
            renderValue={(value) => value.toFixed(1)}
            graphId="combined-temperatures"
          />
        )}

        {/* 2. Combined Power Graph (nozzle, front, back) */}
        {powerData.length > 0 && (
          <AutoSyncedBigGraph
            syncHook={syncHook}
            newData={powerData}
            config={powerConfig}
            unit="W"
            renderValue={(value) => value.toFixed(1)}
            graphId="combined-power"
          />
        )}

        {/* 3. Single Pressure Graph with target line */}
        {bar && (
          <AutoSyncedBigGraph
            syncHook={syncHook}
            newData={{ newData: bar }}
            config={pressureConfig}
            unit="bar"
            renderValue={(value) => value.toFixed(2)}
            graphId="pressure-graph"
          />
        )}

        {/* 4. Single RPM Graph with target line (only if uses_rpm is true) */}
        {uses_rpm && rpm && (
          <AutoSyncedBigGraph
            syncHook={syncHook}
            newData={{ newData: rpm }}
            config={rpmConfig}
            unit="rpm"
            renderValue={(value) => value.toFixed(0)}
            graphId="rpm-graph"
          />
        )}
      </div>

      <SyncedFloatingControlPanel controlProps={syncHook.controlProps} />
    </Page>
  );
}
