import { Page } from "@/components/Page";
import {
  AutoSyncedBigGraph,
  SyncedFloatingControlPanel,
  useGraphSync,
  type GraphConfig,
  type GraphLine,
} from "@/components/graph";
import React from "react";
import { useGluetex } from "../hooks/useGluetex";
import { roundDegreesToDecimals, roundToDecimals } from "@/lib/decimal";
import { TimeSeries } from "@/lib/timeseries";
import { Unit } from "@/control/units";
import { GluetexErrorBanner } from "../components/GluetexErrorBanner";

export function GluetexGraphsPage() {
  const {
    state,
    spoolRpm,
    traversePosition,
    tensionArmAngle,
    slaveTensionArmAngle,
    addonTensionArmAngle,
    pullerSpeed,
    slavePullerSpeed,
    spoolProgress,
    temperature1,
    temperature2,
    temperature3,
    temperature4,
    temperature5,
    temperature6,
    optris1Voltage,
    optris2Voltage,
  } = useGluetex();

  const syncHook = useGraphSync("gluetex-group");

  return (
    <Page className="pb-27">
      <GluetexErrorBanner />
      <div className="flex flex-col gap-4">
        <div className="grid gap-4">
          <PullerSpeedGraph
            syncHook={syncHook}
            newData={pullerSpeed}
            slavePullerSpeed={slavePullerSpeed}
            targetSpeed={state?.puller_state?.target_speed}
            unit="m/min"
            renderValue={(value) => roundToDecimals(value, 0)}
          />

          <TensionArmAngleGraph
            syncHook={syncHook}
            newData={tensionArmAngle}
            slaveTensionArmAngle={slaveTensionArmAngle}
            addonTensionArmAngle={addonTensionArmAngle}
            unit="deg"
            renderValue={(value) => roundDegreesToDecimals(value, 0)}
          />

          <SpoolRpmGraph
            syncHook={syncHook}
            newData={spoolRpm}
            unit="rpm"
            renderValue={(value) => roundToDecimals(value, 0)}
          />

          <TraversePositionGraph
            syncHook={syncHook}
            newData={traversePosition}
            limitInner={state?.traverse_state?.limit_inner}
            limitOuter={state?.traverse_state?.limit_outer}
            unit="mm"
            renderValue={(value) => roundToDecimals(value, 1)}
          />

          <SpoolProgressGraph
            syncHook={syncHook}
            newData={spoolProgress}
            unit="m"
            renderValue={(value) => roundToDecimals(value, 2)}
          />

          <TemperaturesGraph
            syncHook={syncHook}
            temperature1={temperature1}
            temperature2={temperature2}
            temperature3={temperature3}
            temperature4={temperature4}
            temperature5={temperature5}
            temperature6={temperature6}
            targetTemperature1={
              state?.heating_states?.zone_1?.target_temperature
            }
            targetTemperature2={
              state?.heating_states?.zone_2?.target_temperature
            }
            targetTemperature3={
              state?.heating_states?.zone_3?.target_temperature
            }
            targetTemperature4={
              state?.heating_states?.zone_4?.target_temperature
            }
            targetTemperature5={
              state?.heating_states?.zone_5?.target_temperature
            }
            targetTemperature6={
              state?.heating_states?.zone_6?.target_temperature
            }
            unit="C"
            renderValue={(value) => roundToDecimals(value, 1)}
          />

          <OptrisVoltageGraph
            syncHook={syncHook}
            optris1Voltage={optris1Voltage}
            optris2Voltage={optris2Voltage}
            unit="V"
            renderValue={(value) => roundToDecimals(value, 2)}
          />
        </div>
      </div>

      <SyncedFloatingControlPanel controlProps={syncHook.controlProps} />
    </Page>
  );
}

export function SpoolRpmGraph({
  syncHook,
  newData,
  unit,
  renderValue,
}: {
  syncHook: ReturnType<typeof useGraphSync>;
  newData: TimeSeries | null;
  unit?: Unit;
  renderValue?: (value: number) => string;
}) {
  const config: GraphConfig = {
    title: "Spool Speed",
    icon: "lu:RotateCcw",
    colors: {
      primary: "#10b981",
      grid: "#e2e8f0",
      background: "#ffffff",
    },
    exportFilename: "spool_rpm_data",
  };

  return (
    <AutoSyncedBigGraph
      syncHook={syncHook}
      newData={{
        newData,
        color: "#10b981",
      }}
      unit={unit}
      renderValue={renderValue}
      config={config}
      graphId="spool-rpm"
    />
  );
}

export function TraversePositionGraph({
  syncHook,
  newData,
  limitInner,
  limitOuter,
  unit,
  renderValue,
}: {
  syncHook: ReturnType<typeof useGraphSync>;
  newData: TimeSeries | null;
  limitInner?: number;
  limitOuter?: number;
  unit?: Unit;
  renderValue?: (value: number) => string;
}) {
  const lines: GraphLine[] = [];

  if (limitInner !== undefined) {
    lines.push({
      type: "threshold",
      value: limitInner,
      color: "#8b5cf6",
      dash: [5, 5],
    });
  }

  if (limitOuter !== undefined) {
    lines.push({
      type: "threshold",
      value: limitOuter,
      color: "#8b5cf6",
      dash: [5, 5],
    });
  }

  const config: GraphConfig = {
    title: "Traverse Position",
    icon: "lu:Move",
    colors: {
      primary: "#8b5cf6",
    },
    exportFilename: "traverse_position_data",
  };

  return (
    <AutoSyncedBigGraph
      syncHook={syncHook}
      newData={{
        newData,
        color: "#8b5cf6",
        lines,
      }}
      unit={unit}
      renderValue={renderValue}
      config={config}
      graphId="traverse-position"
    />
  );
}

export function TensionArmAngleGraph({
  syncHook,
  newData,
  slaveTensionArmAngle,
  addonTensionArmAngle,
  unit,
  renderValue,
}: {
  syncHook: ReturnType<typeof useGraphSync>;
  newData: TimeSeries | null;
  slaveTensionArmAngle: TimeSeries | null;
  addonTensionArmAngle: TimeSeries | null;
  unit?: Unit;
  renderValue?: (value: number) => string;
}) {
  const tensionArmData = [
    ...(newData
      ? [
          {
            newData,
            title: "Main Tension Arm",
            color: "#06b6d4",
          },
        ]
      : []),
    ...(addonTensionArmAngle
      ? [
          {
            newData: addonTensionArmAngle,
            title: "Addon Tension Arm",
            color: "#f59e0b",
          },
        ]
      : []),
    ...(slaveTensionArmAngle
      ? [
          {
            newData: slaveTensionArmAngle,
            title: "Slave Tension Arm",
            color: "#8b5cf6",
          },
        ]
      : []),
  ];

  const config: GraphConfig = {
    title: "Tension Arm Angle",
    icon: "lu:RotateCw",
    colors: {
      primary: "#06b6d4",
    },
    exportFilename: "tension_arm_angle_data",
    showLegend: true,
  };

  return (
    <AutoSyncedBigGraph
      syncHook={syncHook}
      newData={tensionArmData}
      unit={unit}
      renderValue={renderValue}
      config={config}
      graphId="tension-arm-angle"
    />
  );
}

export function SpoolProgressGraph({
  syncHook,
  newData,
  unit,
  renderValue,
}: {
  syncHook: ReturnType<typeof useGraphSync>;
  newData: TimeSeries | null;
  unit?: Unit;
  renderValue?: (value: number) => string;
}) {
  const config: GraphConfig = {
    title: "Spool Progress",
    icon: "lu:RotateCw",
    colors: {
      primary: "#f59e0b",
    },
    exportFilename: "spool_progress",
  };

  return (
    <AutoSyncedBigGraph
      syncHook={syncHook}
      newData={{
        newData,
        color: "#f59e0b",
      }}
      unit={unit}
      renderValue={renderValue}
      config={config}
      graphId="spool-progress"
    />
  );
}

export function PullerSpeedGraph({
  syncHook,
  newData,
  slavePullerSpeed,
  targetSpeed,
  unit,
  renderValue,
}: {
  syncHook: ReturnType<typeof useGraphSync>;
  newData: TimeSeries | null;
  slavePullerSpeed: TimeSeries | null;
  targetSpeed?: number;
  unit?: Unit;
  renderValue?: (value: number) => string;
}) {
  const pullerSpeedData = [
    ...(newData
      ? [
          {
            newData,
            title: "Main Puller Speed",
            color: "#06b6d4",
            lines:
              targetSpeed !== undefined
                ? [
                    {
                      type: "target" as const,
                      value: targetSpeed,
                      label: "Target Speed",
                      color: "#06b6d4",
                    },
                  ]
                : [],
          },
        ]
      : []),
    ...(slavePullerSpeed
      ? [
          {
            newData: slavePullerSpeed,
            title: "Slave Puller Speed",
            color: "#8b5cf6",
          },
        ]
      : []),
  ];

  const config: GraphConfig = {
    title: "Puller Speed",
    icon: "lu:Gauge",
    colors: {
      primary: "#06b6d4",
    },
    exportFilename: "puller_speed_data",
    showLegend: true,
  };

  return (
    <AutoSyncedBigGraph
      syncHook={syncHook}
      newData={pullerSpeedData}
      unit={unit}
      renderValue={renderValue}
      config={config}
      graphId="puller-speed"
    />
  );
}

export function TemperaturesGraph({
  syncHook,
  temperature1,
  temperature2,
  temperature3,
  temperature4,
  temperature5,
  temperature6,
  targetTemperature1,
  targetTemperature2,
  targetTemperature3,
  targetTemperature4,
  targetTemperature5,
  targetTemperature6,
  unit,
  renderValue,
}: {
  syncHook: ReturnType<typeof useGraphSync>;
  temperature1: TimeSeries | null;
  temperature2: TimeSeries | null;
  temperature3: TimeSeries | null;
  temperature4: TimeSeries | null;
  temperature5: TimeSeries | null;
  temperature6: TimeSeries | null;
  targetTemperature1?: number;
  targetTemperature2?: number;
  targetTemperature3?: number;
  targetTemperature4?: number;
  targetTemperature5?: number;
  targetTemperature6?: number;
  unit?: Unit;
  renderValue?: (value: number) => string;
}) {
  const temperatureData = [
    ...(temperature1
      ? [
          {
            newData: temperature1,
            title: "Zone 1",
            color: "#ef4444",
            lines:
              targetTemperature1 !== undefined
                ? [
                    {
                      type: "target" as const,
                      value: targetTemperature1,
                      color: "#ef4444",
                      show: true,
                    },
                  ]
                : [],
          },
        ]
      : []),
    ...(temperature2
      ? [
          {
            newData: temperature2,
            title: "Zone 2",
            color: "#f97316",
            lines:
              targetTemperature2 !== undefined
                ? [
                    {
                      type: "target" as const,
                      value: targetTemperature2,
                      color: "#f97316",
                      show: true,
                    },
                  ]
                : [],
          },
        ]
      : []),
    ...(temperature3
      ? [
          {
            newData: temperature3,
            title: "Zone 3",
            color: "#f59e0b",
            lines:
              targetTemperature3 !== undefined
                ? [
                    {
                      type: "target" as const,
                      value: targetTemperature3,
                      color: "#f59e0b",
                      show: true,
                    },
                  ]
                : [],
          },
        ]
      : []),
    ...(temperature4
      ? [
          {
            newData: temperature4,
            title: "Zone 4",
            color: "#84cc16",
            lines:
              targetTemperature4 !== undefined
                ? [
                    {
                      type: "target" as const,
                      value: targetTemperature4,
                      color: "#84cc16",
                      show: true,
                    },
                  ]
                : [],
          },
        ]
      : []),
    ...(temperature5
      ? [
          {
            newData: temperature5,
            title: "Zone 5",
            color: "#22c55e",
            lines:
              targetTemperature5 !== undefined
                ? [
                    {
                      type: "target" as const,
                      value: targetTemperature5,
                      color: "#22c55e",
                      show: true,
                    },
                  ]
                : [],
          },
        ]
      : []),
    ...(temperature6
      ? [
          {
            newData: temperature6,
            title: "Zone 6",
            color: "#14b8a6",
            lines:
              targetTemperature6 !== undefined
                ? [
                    {
                      type: "target" as const,
                      value: targetTemperature6,
                      color: "#14b8a6",
                      show: true,
                    },
                  ]
                : [],
          },
        ]
      : []),
  ];

  const config: GraphConfig = {
    title: "Heater Temperatures",
    icon: "lu:Thermometer",
    colors: {
      primary: "#ef4444",
      grid: "#e2e8f0",
      axis: "#64748b",
      background: "#ffffff",
    },
    exportFilename: "heater_temperatures_data",
    showLegend: true,
  };

  return (
    <AutoSyncedBigGraph
      syncHook={syncHook}
      newData={temperatureData}
      unit={unit}
      renderValue={renderValue}
      config={config}
      graphId="heater-temperatures"
    />
  );
}

export function OptrisVoltageGraph({
  syncHook,
  optris1Voltage,
  optris2Voltage,
  unit,
  renderValue,
}: {
  syncHook: ReturnType<typeof useGraphSync>;
  optris1Voltage: TimeSeries | null;
  optris2Voltage: TimeSeries | null;
  unit?: Unit;
  renderValue?: (value: number) => string;
}) {
  const optrisVoltageData = [
    ...(optris1Voltage
      ? [
          {
            newData: optris1Voltage,
            title: "Optris 1",
            color: "#3b82f6",
          },
        ]
      : []),
    ...(optris2Voltage
      ? [
          {
            newData: optris2Voltage,
            title: "Optris 2",
            color: "#8b5cf6",
          },
        ]
      : []),
  ];

  const config: GraphConfig = {
    title: "Optris Voltage",
    icon: "lu:Zap",
    colors: {
      primary: "#3b82f6",
      grid: "#e2e8f0",
      axis: "#64748b",
      background: "#ffffff",
    },
    exportFilename: "optris_voltage_data",
    showLegend: true,
  };

  return (
    <AutoSyncedBigGraph
      syncHook={syncHook}
      newData={optrisVoltageData}
      unit={unit}
      renderValue={renderValue}
      config={config}
      graphId="optris-voltage"
    />
  );
}
