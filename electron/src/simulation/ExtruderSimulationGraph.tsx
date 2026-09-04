import React from "react";
import {
  MarkerProvider,
  SyncedFloatingControlPanel,
  useGraphSync,
  type GraphConfig,
} from "@/components/graph";
import { GraphWithMarkerControls } from "@/components/graph/marker/GraphWithMarkerControls";
import { useExtruderSimulation } from "./useExtruderSimulation";

const ZONE_COLORS = {
  front: "#f59e0b",
  middle: "#8b5cf6",
  back: "#3b82f6",
  nozzle: "#ef4444",
};

export function ExtruderSimulationGraph() {
  const {
    state,
    frontTemperature,
    middleTemperature,
    backTemperature,
    nozzleTemperature,
    targetFrontTemperature,
    targetMiddleTemperature,
    targetBackTemperature,
    targetNozzleTemperature,
    frontPower,
    middlePower,
    backPower,
    nozzlePower,
  } = useExtruderSimulation();

  const syncHook = useGraphSync("extruder-simulation-graphs");

  const baseConfig: GraphConfig = {
    defaultTimeWindow: 30 * 60 * 1000,
    exportFilename: "extruder_simulation_data",
    showLegend: true,
    title: "extruder simulation",
  };

  const temperatureData = [
    {
      newData: frontTemperature,
      title: "Front",
      color: ZONE_COLORS.front,
      lines:
        state?.setpoints_c.front !== undefined
          ? [
              {
                type: "target" as const,
                value: state.setpoints_c.front,
                targetSeries: targetFrontTemperature,
                color: ZONE_COLORS.front,
                show: true,
              },
            ]
          : [],
    },
    {
      newData: middleTemperature,
      title: "Middle",
      color: ZONE_COLORS.middle,
      lines:
        state?.setpoints_c.middle !== undefined
          ? [
              {
                type: "target" as const,
                value: state.setpoints_c.middle,
                targetSeries: targetMiddleTemperature,
                color: ZONE_COLORS.middle,
                show: true,
              },
            ]
          : [],
    },
    {
      newData: backTemperature,
      title: "Back",
      color: ZONE_COLORS.back,
      lines:
        state?.setpoints_c.back !== undefined
          ? [
              {
                type: "target" as const,
                value: state.setpoints_c.back,
                targetSeries: targetBackTemperature,
                color: ZONE_COLORS.back,
                show: true,
              },
            ]
          : [],
    },
    {
      newData: nozzleTemperature,
      title: "Nozzle",
      color: ZONE_COLORS.nozzle,
      lines:
        state?.setpoints_c.nozzle !== undefined
          ? [
              {
                type: "target" as const,
                value: state.setpoints_c.nozzle,
                targetSeries: targetNozzleTemperature,
                color: ZONE_COLORS.nozzle,
                show: true,
              },
            ]
          : [],
    },
  ];

  const temperatureConfig: GraphConfig = {
    ...baseConfig,
    title: "Zone Temperatures",
    exportFilename: "extruder_simulation_temperatures",
    colors: {
      primary: ZONE_COLORS.nozzle,
      grid: "#e2e8f0",
      axis: "#64748b",
      background: "#ffffff",
    },
  };

  const powerData = [
    { newData: frontPower, title: "Front", color: ZONE_COLORS.front },
    { newData: middlePower, title: "Middle", color: ZONE_COLORS.middle },
    { newData: backPower, title: "Back", color: ZONE_COLORS.back },
    { newData: nozzlePower, title: "Nozzle", color: ZONE_COLORS.nozzle },
  ];

  const powerConfig: GraphConfig = {
    ...baseConfig,
    title: "Band Power",
    exportFilename: "extruder_simulation_power",
    colors: {
      primary: "#10b981",
      grid: "#e2e8f0",
      axis: "#64748b",
      background: "#ffffff",
    },
  };

  return (
    <MarkerProvider>
      <div className="flex flex-col gap-4">
        <GraphWithMarkerControls
          syncHook={syncHook}
          newData={temperatureData}
          config={temperatureConfig}
          unit="C"
          renderValue={(value) => value.toFixed(1)}
          graphId="simulation-temperatures"
          currentTimeSeries={frontTemperature}
          machineId="extruder-simulation-graphs"
        />

        <GraphWithMarkerControls
          syncHook={syncHook}
          newData={powerData}
          config={powerConfig}
          unit="W"
          renderValue={(value) => value.toFixed(0)}
          graphId="simulation-power"
          currentTimeSeries={frontPower}
          machineId="extruder-simulation-graphs"
        />
      </div>

      <SyncedFloatingControlPanel
        controlProps={syncHook.controlProps}
        machineId="extruder-simulation-graphs"
      />
    </MarkerProvider>
  );
}
