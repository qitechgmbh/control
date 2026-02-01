import { Page } from "@/components/Page";
import {
  MarkerProvider,
  SyncedFloatingControlPanel,
  useGraphSync,
  type GraphConfig,
} from "@/components/graph";
import React from "react";
import { useExtruder2 } from "./useExtruder";
import { GraphWithMarkerControls } from "@/components/graph/GraphWithMarkerControls";

export function Extruder2GraphsPage() {
  const {
    state,
    nozzleTemperature,
    nozzlePower,
    frontTemperature,
    frontPower,
    backTemperature,
    backPower,
    middleTemperature,
    middlePower,
    pressure,
    motorScrewRpm,
    motorCurrent,
    motorPower,
    combinedPower,
    // Target value history for graph target lines
    targetPressure,
    targetScrewRpm,
    targetNozzleTemperature,
    targetFrontTemperature,
    targetMiddleTemperature,
    targetBackTemperature,
  } = useExtruder2();

  const syncHook = useGraphSync("extruder-graphs");

  // Base config
  const baseConfig: GraphConfig = {
    defaultTimeWindow: 30 * 60 * 1000, // 30 minutes
    exportFilename: "extruder_data",
    showLegend: true,
    title: "extruder data",
  };

  const temperatureData = [
    ...(nozzleTemperature
      ? [
          {
            newData: nozzleTemperature,
            title: "Nozzle",
            color: "#ef4444",
            lines:
              state?.heating_states.nozzle?.target_temperature !== undefined
                ? [
                    {
                      type: "target" as const,
                      value: state.heating_states.nozzle.target_temperature,
                      targetSeries: targetNozzleTemperature,
                      color: "#ef4444",
                      show: true,
                    },
                  ]
                : [],
          },
        ]
      : []),
    ...(frontTemperature
      ? [
          {
            newData: frontTemperature,
            title: "Front",
            color: "#f59e0b",
            lines:
              state?.heating_states.front?.target_temperature !== undefined
                ? [
                    {
                      type: "target" as const,
                      value: state.heating_states.front.target_temperature,
                      targetSeries: targetFrontTemperature,
                      color: "#f59e0b",
                      show: true,
                    },
                  ]
                : [],
          },
        ]
      : []),
    ...(middleTemperature
      ? [
          {
            newData: middleTemperature,
            title: "Middle",
            color: "#8b5cf6",
            lines:
              state?.heating_states.middle?.target_temperature !== undefined
                ? [
                    {
                      type: "target" as const,
                      value: state.heating_states.middle.target_temperature,
                      targetSeries: targetMiddleTemperature,
                      color: "#8b5cf6",
                      show: true,
                    },
                  ]
                : [],
          },
        ]
      : []),
    ...(backTemperature
      ? [
          {
            newData: backTemperature,
            title: "Back",
            color: "#3b82f6",
            lines:
              state?.heating_states.back?.target_temperature !== undefined
                ? [
                    {
                      type: "target" as const,
                      value: state.heating_states.back.target_temperature,
                      targetSeries: targetBackTemperature,
                      color: "#3b82f6",
                      show: true,
                    },
                  ]
                : [],
          },
        ]
      : []),
  ];

  const temperatureConfig: GraphConfig = {
    ...baseConfig,
    title: "Temperatures",
    exportFilename: "temperatures_data",
    colors: {
      primary: "#ef4444",
      grid: "#e2e8f0",
      axis: "#64748b",
      background: "#ffffff",
    },
  };

  // Combined Power Graph (no target lines for power)
  const powerData = [
    ...(nozzlePower
      ? [
          {
            newData: nozzlePower,
            title: "Nozzle",
            color: "#ef4444",
          },
        ]
      : []),
    ...(frontPower
      ? [
          {
            newData: frontPower,
            title: "Front",
            color: "#f59e0b",
          },
        ]
      : []),
    ...(middlePower
      ? [
          {
            newData: middlePower,
            title: "Middle",
            color: "#8b5cf6",
          },
        ]
      : []),
    ...(backPower
      ? [
          {
            newData: backPower,
            title: "Back",
            color: "#3b82f6",
          },
        ]
      : []),
    ...(motorPower
      ? [
          {
            newData: motorPower,
            title: "Motor",
            color: "#10b981",
          },
        ]
      : []),
    ...(combinedPower
      ? [
          {
            newData: combinedPower,
            title: "Total",
            color: "#000000",
          },
        ]
      : []),
  ];

  const powerConfig: GraphConfig = {
    ...baseConfig,
    title: "Power Outputs",
    exportFilename: "power_data",
    colors: {
      primary: "#10b981",
      grid: "#e2e8f0",
      axis: "#64748b",
      background: "#ffffff",
    },
  };

  const currentConfig: GraphConfig = {
    ...baseConfig,
    title: "Motor Current",
    exportFilename: "motor_current_data",
    colors: {
      primary: "#3b82f6",
      grid: "#e2e8f0",
      axis: "#64748b",
      background: "#ffffff",
    },
  };

  // Pressure Graph with connected target line
  const pressureConfig: GraphConfig = {
    ...baseConfig,
    title: "Pressure",
    exportFilename: "pressure_data",
    colors: {
      primary: "#3b82f6",
      grid: "#e2e8f0",
      axis: "#64748b",
      background: "#ffffff",
    },
  };

  // RPM Graph with connected target line
  const rpmConfig: GraphConfig = {
    ...baseConfig,
    title: "RPM",
    exportFilename: "rpm_data",
    colors: {
      primary: "#8b5cf6",
      grid: "#e2e8f0",
      axis: "#64748b",
      background: "#ffffff",
    },
  };

  return (
    <Page className="pb-25">
      <MarkerProvider>
        <div className="flex flex-col gap-4">
          <GraphWithMarkerControls
            syncHook={syncHook}
            newData={{
              newData: pressure,
              color: "#3b82f6",
              lines:
                state?.pressure_state.target_bar !== undefined
                  ? [
                      {
                        type: "target" as const,
                        value: state.pressure_state.target_bar,
                        targetSeries: targetPressure,
                        color: "#3b82f6",
                        show: true,
                      },
                    ]
                  : [],
            }}
            config={pressureConfig}
            unit="bar"
            renderValue={(value) => value.toFixed(2)}
            graphId="pressure-graph"
            currentTimeSeries={pressure}
            machineId="extruder-graphs"
          />

          <GraphWithMarkerControls
            syncHook={syncHook}
            newData={temperatureData}
            config={temperatureConfig}
            unit="C"
            renderValue={(value) => value.toFixed(1)}
            graphId="combined-temperatures"
            currentTimeSeries={nozzleTemperature}
            machineId="extruder-graphs"
          />

          <GraphWithMarkerControls
            syncHook={syncHook}
            newData={powerData}
            config={powerConfig}
            unit="W"
            renderValue={(value) => value.toFixed(1)}
            graphId="combined-power"
            currentTimeSeries={combinedPower}
            machineId="extruder-graphs"
          />

          <GraphWithMarkerControls
            syncHook={syncHook}
            newData={{
              newData: motorCurrent,
              color: "#3b82f6",
            }}
            config={currentConfig}
            unit="A"
            renderValue={(value) => value.toFixed(2)}
            graphId="motor-current"
            currentTimeSeries={motorCurrent}
            machineId="extruder-graphs"
          />

          <GraphWithMarkerControls
            syncHook={syncHook}
            newData={{
              newData: motorScrewRpm,
              color: "#8b5cf6",
              lines:
                state?.screw_state.target_rpm !== undefined
                  ? [
                      {
                        type: "target" as const,
                        value: state.screw_state.target_rpm,
                        targetSeries: targetScrewRpm,
                        color: "#8b5cf6",
                        show: true,
                      },
                    ]
                  : [],
            }}
            config={rpmConfig}
            unit="rpm"
            renderValue={(value) => value.toFixed(0)}
            graphId="rpm-graph"
            currentTimeSeries={motorScrewRpm}
            machineId="extruder-graphs"
          />
        </div>

        <SyncedFloatingControlPanel
          controlProps={syncHook.controlProps}
          machineId="extruder-graphs"
        />
      </MarkerProvider>
    </Page>
  );
}
