import { Page } from "@/components/Page";
import {
  MarkerProvider,
  SyncedFloatingControlPanel,
  useGraphSync,
  type GraphConfig,
} from "@/components/graph";
import { GraphWithMarkerControls } from "@/components/graph/marker/GraphWithMarkerControls";
import { GraphHistoryProvider } from "@/components/graph/history/GraphHistoryContext";
import React from "react";
import { useExtruder3 } from "./useExtruder";

export function Extruder3GraphsPage() {
  const {
    machineIdentification,
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
  } = useExtruder3();

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
            history: "heating.nozzle.temperature",
            title: "Nozzle",
            color: "#ef4444",
            lines:
              state?.heating_states.nozzle?.target_temperature !== undefined
                ? [
                    {
                      type: "target" as const,
                      value: state.heating_states.nozzle.target_temperature,
                      targetSeries: targetNozzleTemperature,
                      history: "heating.nozzle.target_temperature",
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
            history: "heating.front.temperature",
            title: "Front",
            color: "#f59e0b",
            lines:
              state?.heating_states.front?.target_temperature !== undefined
                ? [
                    {
                      type: "target" as const,
                      value: state.heating_states.front.target_temperature,
                      targetSeries: targetFrontTemperature,
                      history: "heating.front.target_temperature",
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
            history: "heating.middle.temperature",
            title: "Middle",
            color: "#8b5cf6",
            lines:
              state?.heating_states.middle?.target_temperature !== undefined
                ? [
                    {
                      type: "target" as const,
                      value: state.heating_states.middle.target_temperature,
                      targetSeries: targetMiddleTemperature,
                      history: "heating.middle.target_temperature",
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
            history: "heating.back.temperature",
            title: "Back",
            color: "#3b82f6",
            lines:
              state?.heating_states.back?.target_temperature !== undefined
                ? [
                    {
                      type: "target" as const,
                      value: state.heating_states.back.target_temperature,
                      targetSeries: targetBackTemperature,
                      history: "heating.back.target_temperature",
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
            history: "heating.nozzle.power",
            title: "Nozzle",
            color: "#ef4444",
          },
        ]
      : []),
    ...(frontPower
      ? [
          {
            newData: frontPower,
            history: "heating.front.power",
            title: "Front",
            color: "#f59e0b",
          },
        ]
      : []),
    ...(middlePower
      ? [
          {
            newData: middlePower,
            history: "heating.middle.power",
            title: "Middle",
            color: "#8b5cf6",
          },
        ]
      : []),
    ...(backPower
      ? [
          {
            newData: backPower,
            history: "heating.back.power",
            title: "Back",
            color: "#3b82f6",
          },
        ]
      : []),
    ...(motorPower
      ? [
          {
            newData: motorPower,
            history: "motor.power",
            title: "Motor",
            color: "#10b981",
          },
        ]
      : []),
    ...(combinedPower
      ? [
          {
            newData: combinedPower,
            history: "power.combined",
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
      <GraphHistoryProvider machine={machineIdentification}>
        <MarkerProvider>
          <div className="flex flex-col gap-4">
            <GraphWithMarkerControls
              syncHook={syncHook}
              newData={{
                newData: pressure,
                history: "pressure.value",
                color: "#3b82f6",
                lines:
                  state?.pressure_state.target_bar !== undefined
                    ? [
                        {
                          type: "target" as const,
                          value: state.pressure_state.target_bar,
                          targetSeries: targetPressure,
                          history: "screw.target_pressure",
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
              currentTimeSeries={nozzleTemperature ?? null}
              machineId="extruder-graphs"
            />

            <GraphWithMarkerControls
              syncHook={syncHook}
              newData={powerData}
              config={powerConfig}
              unit="W"
              renderValue={(value) => value.toFixed(1)}
              graphId="combined-power"
              currentTimeSeries={combinedPower ?? null}
              machineId="extruder-graphs"
            />

            <GraphWithMarkerControls
              syncHook={syncHook}
              newData={{
                newData: motorCurrent,
                history: "motor.current",
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
                history: "motor.rpm",
                color: "#8b5cf6",
                lines:
                  state?.screw_state.target_rpm !== undefined
                    ? [
                        {
                          type: "target" as const,
                          value: state.screw_state.target_rpm,
                          targetSeries: targetScrewRpm,
                          history: "screw.target_rpm",
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
      </GraphHistoryProvider>
    </Page>
  );
}
