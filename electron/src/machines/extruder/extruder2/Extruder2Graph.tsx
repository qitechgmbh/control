import { Page } from "@/components/Page";
import {
  BigGraph,
  GraphConfig,
  GraphSyncProvider,
  GraphControls,
  FloatingControlPanel,
} from "@/helpers/BigGraph";
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

  // Temperature graph configs
  const createTemperatureConfig = (
    title: string,
    targetTemp?: number,
  ): GraphConfig => ({
    title,
    description: "Real-time temperature monitoring",
    defaultTimeWindow: 30 * 60 * 1000, // 30 minutes
    exportFilename: `${title.toLowerCase().replace(" ", "_")}_data`,
    lines:
      targetTemp !== undefined
        ? [
            {
              type: "target",
              value: targetTemp,
              label: "Target Temperature",
              color: "#6b7280",
            },
          ]
        : [],
    colors: {
      primary: "#ef4444",
      grid: "#e2e8f0",
      axis: "#64748b",
      background: "#ffffff",
    },
  });

  // Power graph config
  const powerConfig: GraphConfig = {
    title: "Power Output",
    description: "Real-time power monitoring",
    defaultTimeWindow: 30 * 60 * 1000, // 30 minutes
    exportFilename: "power_data",
    colors: {
      primary: "#10b981",
      grid: "#e2e8f0",
      axis: "#64748b",
      background: "#ffffff",
    },
  };

  // Pressure/RPM graph configs
  const pressureConfig: GraphConfig = {
    title: "Pressure",
    description: "Real-time pressure monitoring",
    defaultTimeWindow: 30 * 60 * 1000, // 30 minutes
    exportFilename: "pressure_data",
    lines:
      targetBar !== undefined
        ? [
            {
              type: "target",
              value: targetBar,
              label: "Target Pressure",
              color: "#6b7280",
            },
          ]
        : [],
    colors: {
      primary: "#3b82f6",
      grid: "#e2e8f0",
      axis: "#64748b",
      background: "#ffffff",
    },
  };

  const rpmConfig: GraphConfig = {
    title: "RPM",
    description: "Real-time RPM monitoring",
    defaultTimeWindow: 30 * 60 * 1000, // 30 minutes
    exportFilename: "rpm_data",
    lines:
      targetRpm !== undefined
        ? [
            {
              type: "target",
              value: targetRpm,
              label: "Target RPM",
              color: "#6b7280",
            },
          ]
        : [],
    colors: {
      primary: "#8b5cf6",
      grid: "#e2e8f0",
      axis: "#64748b",
      background: "#ffffff",
    },
  };

  return (
    <Page className="pb-20">
      <GraphSyncProvider groupId="extruder-group">
        <div className="flex flex-col gap-4">
          <GraphControls groupId="extruder-group" />

          {/* Temperature Graphs */}
          <BigGraph
            newData={nozzleTemperature}
            unit="deg"
            renderValue={(value) => value.toFixed(1)}
            config={createTemperatureConfig(
              "Nozzle Temperature",
              nozzleHeatingState?.target_temperature,
            )}
            syncGroupId="extruder-group"
            graphId="nozzle-temp-graph"
          />

          <BigGraph
            newData={frontTemperature}
            unit="deg"
            renderValue={(value) => value.toFixed(1)}
            config={createTemperatureConfig(
              "Front Temperature",
              frontHeatingState?.target_temperature,
            )}
            syncGroupId="extruder-group"
            graphId="front-temp-graph"
          />

          <BigGraph
            newData={backTemperature}
            unit="deg"
            renderValue={(value) => value.toFixed(1)}
            config={createTemperatureConfig(
              "Back Temperature",
              backHeatingState?.target_temperature,
            )}
            syncGroupId="extruder-group"
            graphId="back-temp-graph"
          />

          <BigGraph
            newData={middleTemperature}
            unit="deg"
            renderValue={(value) => value.toFixed(1)}
            config={createTemperatureConfig(
              "Middle Temperature",
              middleHeatingState?.target_temperature,
            )}
            syncGroupId="extruder-group"
            graphId="middle-temp-graph"
          />

          {/* Power Graphs */}
          <BigGraph
            newData={nozzlePower}
            unit="W"
            renderValue={(value) => value.toFixed(1)}
            config={{ ...powerConfig, title: "Nozzle Power" }}
            syncGroupId="extruder-group"
            graphId="nozzle-power-graph"
          />

          <BigGraph
            newData={frontPower}
            unit="W"
            renderValue={(value) => value.toFixed(1)}
            config={{ ...powerConfig, title: "Front Power" }}
            syncGroupId="extruder-group"
            graphId="front-power-graph"
          />

          <BigGraph
            newData={backPower}
            unit="W"
            renderValue={(value) => value.toFixed(1)}
            config={{ ...powerConfig, title: "Back Power" }}
            syncGroupId="extruder-group"
            graphId="back-power-graph"
          />

          <BigGraph
            newData={middlePower}
            unit="W"
            renderValue={(value) => value.toFixed(1)}
            config={{ ...powerConfig, title: "Middle Power" }}
            syncGroupId="extruder-group"
            graphId="middle-power-graph"
          />

          {/* Pressure Graph */}
          <BigGraph
            newData={bar}
            unit="bar"
            renderValue={(value) => value.toFixed(2)}
            config={pressureConfig}
            syncGroupId="extruder-group"
            graphId="pressure-graph"
          />

          {/* RPM Graph (only if uses_rpm is true) */}
          {uses_rpm && (
            <BigGraph
              newData={rpm}
              unit="rpm"
              renderValue={(value) => value.toFixed(0)}
              config={rpmConfig}
              syncGroupId="extruder-group"
              graphId="rpm-graph"
            />
          )}
        </div>
        <FloatingControlPanel groupId="extruder-group" />
      </GraphSyncProvider>
    </Page>
  );
}
