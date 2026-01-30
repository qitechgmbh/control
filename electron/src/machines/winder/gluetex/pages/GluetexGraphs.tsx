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

export function GluetexGraphsPage() {
  const {
    state,
    spoolRpm,
    traversePosition,
    tensionArmAngle,
    pullerSpeed,
    spoolProgress,
    temperature1,
    temperature2,
    temperature3,
    temperature4,
    temperature5,
    temperature6,
  } = useGluetex();

  const syncHook = useGraphSync("gluetex-group");

  return (
    <Page className="pb-27">
      <div className="flex flex-col gap-4">
        <div className="grid gap-4">
          <PullerSpeedGraph
            syncHook={syncHook}
            newData={pullerSpeed}
            targetSpeed={state?.puller_state?.target_speed}
            unit="m/min"
            renderValue={(value) => roundToDecimals(value, 0)}
          />

          <TensionArmAngleGraph
            syncHook={syncHook}
            newData={tensionArmAngle}
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

          <TemperatureZone1Graph
            syncHook={syncHook}
            newData={temperature1}
            targetTemperature={state?.heating_states?.zone_1?.target_temperature}
            unit="C"
            renderValue={(value) => roundToDecimals(value, 1)}
          />

          <TemperatureZone2Graph
            syncHook={syncHook}
            newData={temperature2}
            targetTemperature={state?.heating_states?.zone_2?.target_temperature}
            unit="C"
            renderValue={(value) => roundToDecimals(value, 1)}
          />

          <TemperatureZone3Graph
            syncHook={syncHook}
            newData={temperature3}
            targetTemperature={state?.heating_states?.zone_3?.target_temperature}
            unit="C"
            renderValue={(value) => roundToDecimals(value, 1)}
          />

          <TemperatureZone4Graph
            syncHook={syncHook}
            newData={temperature4}
            targetTemperature={state?.heating_states?.zone_4?.target_temperature}
            unit="C"
            renderValue={(value) => roundToDecimals(value, 1)}
          />

          <TemperatureZone5Graph
            syncHook={syncHook}
            newData={temperature5}
            targetTemperature={state?.heating_states?.zone_5?.target_temperature}
            unit="C"
            renderValue={(value) => roundToDecimals(value, 1)}
          />

          <TemperatureZone6Graph
            syncHook={syncHook}
            newData={temperature6}
            targetTemperature={state?.heating_states?.zone_6?.target_temperature}
            unit="C"
            renderValue={(value) => roundToDecimals(value, 1)}
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
  unit,
  renderValue,
}: {
  syncHook: ReturnType<typeof useGraphSync>;
  newData: TimeSeries | null;
  unit?: Unit;
  renderValue?: (value: number) => string;
}) {
  const config: GraphConfig = {
    title: "Tension Arm Angle",
    icon: "lu:RotateCw",
    colors: {
      primary: "#f59e0b",
    },
    exportFilename: "tension_arm_angle_data",
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
  targetSpeed,
  unit,
  renderValue,
}: {
  syncHook: ReturnType<typeof useGraphSync>;
  newData: TimeSeries | null;
  targetSpeed?: number;
  unit?: Unit;
  renderValue?: (value: number) => string;
}) {
  const lines: GraphLine[] = [];

  if (targetSpeed !== undefined) {
    lines.push({
      type: "target",
      value: targetSpeed,
      label: "Target Speed",
      color: "#06b6d4", // Match series color
    });
  }

  const config: GraphConfig = {
    title: "Puller Speed",
    icon: "lu:Gauge",
    colors: {
      primary: "#06b6d4",
    },
    exportFilename: "puller_speed_data",
  };

  return (
    <AutoSyncedBigGraph
      syncHook={syncHook}
      newData={{
        newData,
        color: "#06b6d4",
        lines,
      }}
      unit={unit}
      renderValue={renderValue}
      config={config}
      graphId="puller-speed"
    />
  );
}

export function TemperatureZone1Graph({
  syncHook,
  newData,
  targetTemperature,
  unit,
  renderValue,
}: {
  syncHook: ReturnType<typeof useGraphSync>;
  newData: TimeSeries | null;
  targetTemperature?: number;
  unit?: Unit;
  renderValue?: (value: number) => string;
}) {
  const lines: GraphLine[] = [];

  if (targetTemperature !== undefined) {
    lines.push({
      type: "target",
      value: targetTemperature,
      label: "Target Temperature",
      color: "#ef4444",
    });
  }

  const config: GraphConfig = {
    title: "Temperature Zone 1",
    icon: "lu:Thermometer",
    colors: {
      primary: "#ef4444",
    },
    exportFilename: "temperature_zone_1_data",
  };

  return (
    <AutoSyncedBigGraph
      syncHook={syncHook}
      newData={{
        newData,
        color: "#ef4444",
        lines,
      }}
      unit={unit}
      renderValue={renderValue}
      config={config}
      graphId="temperature-zone-1"
    />
  );
}

export function TemperatureZone2Graph({
  syncHook,
  newData,
  targetTemperature,
  unit,
  renderValue,
}: {
  syncHook: ReturnType<typeof useGraphSync>;
  newData: TimeSeries | null;
  targetTemperature?: number;
  unit?: Unit;
  renderValue?: (value: number) => string;
}) {
  const lines: GraphLine[] = [];

  if (targetTemperature !== undefined) {
    lines.push({
      type: "target",
      value: targetTemperature,
      label: "Target Temperature",
      color: "#f97316",
    });
  }

  const config: GraphConfig = {
    title: "Temperature Zone 2",
    icon: "lu:Thermometer",
    colors: {
      primary: "#f97316",
    },
    exportFilename: "temperature_zone_2_data",
  };

  return (
    <AutoSyncedBigGraph
      syncHook={syncHook}
      newData={{
        newData,
        color: "#f97316",
        lines,
      }}
      unit={unit}
      renderValue={renderValue}
      config={config}
      graphId="temperature-zone-2"
    />
  );
}

export function TemperatureZone3Graph({
  syncHook,
  newData,
  targetTemperature,
  unit,
  renderValue,
}: {
  syncHook: ReturnType<typeof useGraphSync>;
  newData: TimeSeries | null;
  targetTemperature?: number;
  unit?: Unit;
  renderValue?: (value: number) => string;
}) {
  const lines: GraphLine[] = [];

  if (targetTemperature !== undefined) {
    lines.push({
      type: "target",
      value: targetTemperature,
      label: "Target Temperature",
      color: "#f59e0b",
    });
  }

  const config: GraphConfig = {
    title: "Temperature Zone 3",
    icon: "lu:Thermometer",
    colors: {
      primary: "#f59e0b",
    },
    exportFilename: "temperature_zone_3_data",
  };

  return (
    <AutoSyncedBigGraph
      syncHook={syncHook}
      newData={{
        newData,
        color: "#f59e0b",
        lines,
      }}
      unit={unit}
      renderValue={renderValue}
      config={config}
      graphId="temperature-zone-3"
    />
  );
}

export function TemperatureZone4Graph({
  syncHook,
  newData,
  targetTemperature,
  unit,
  renderValue,
}: {
  syncHook: ReturnType<typeof useGraphSync>;
  newData: TimeSeries | null;
  targetTemperature?: number;
  unit?: Unit;
  renderValue?: (value: number) => string;
}) {
  const lines: GraphLine[] = [];

  if (targetTemperature !== undefined) {
    lines.push({
      type: "target",
      value: targetTemperature,
      label: "Target Temperature",
      color: "#84cc16",
    });
  }

  const config: GraphConfig = {
    title: "Temperature Zone 4",
    icon: "lu:Thermometer",
    colors: {
      primary: "#84cc16",
    },
    exportFilename: "temperature_zone_4_data",
  };

  return (
    <AutoSyncedBigGraph
      syncHook={syncHook}
      newData={{
        newData,
        color: "#84cc16",
        lines,
      }}
      unit={unit}
      renderValue={renderValue}
      config={config}
      graphId="temperature-zone-4"
    />
  );
}

export function TemperatureZone5Graph({
  syncHook,
  newData,
  targetTemperature,
  unit,
  renderValue,
}: {
  syncHook: ReturnType<typeof useGraphSync>;
  newData: TimeSeries | null;
  targetTemperature?: number;
  unit?: Unit;
  renderValue?: (value: number) => string;
}) {
  const lines: GraphLine[] = [];

  if (targetTemperature !== undefined) {
    lines.push({
      type: "target",
      value: targetTemperature,
      label: "Target Temperature",
      color: "#22c55e",
    });
  }

  const config: GraphConfig = {
    title: "Temperature Zone 5",
    icon: "lu:Thermometer",
    colors: {
      primary: "#22c55e",
    },
    exportFilename: "temperature_zone_5_data",
  };

  return (
    <AutoSyncedBigGraph
      syncHook={syncHook}
      newData={{
        newData,
        color: "#22c55e",
        lines,
      }}
      unit={unit}
      renderValue={renderValue}
      config={config}
      graphId="temperature-zone-5"
    />
  );
}

export function TemperatureZone6Graph({
  syncHook,
  newData,
  targetTemperature,
  unit,
  renderValue,
}: {
  syncHook: ReturnType<typeof useGraphSync>;
  newData: TimeSeries | null;
  targetTemperature?: number;
  unit?: Unit;
  renderValue?: (value: number) => string;
}) {
  const lines: GraphLine[] = [];

  if (targetTemperature !== undefined) {
    lines.push({
      type: "target",
      value: targetTemperature,
      label: "Target Temperature",
      color: "#14b8a6",
    });
  }

  const config: GraphConfig = {
    title: "Temperature Zone 6",
    icon: "lu:Thermometer",
    colors: {
      primary: "#14b8a6",
    },
    exportFilename: "temperature_zone_6_data",
  };

  return (
    <AutoSyncedBigGraph
      syncHook={syncHook}
      newData={{
        newData,
        color: "#14b8a6",
        lines,
      }}
      unit={unit}
      renderValue={renderValue}
      config={config}
      graphId="temperature-zone-6"
    />
  );
}

