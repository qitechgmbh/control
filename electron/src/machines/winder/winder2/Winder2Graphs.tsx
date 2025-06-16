import { Page } from "@/components/Page";
import {
  AutoSyncedBigGraph,
  SyncedFloatingControlPanel,
  SyncedGraphControls,
  useGraphSync,
  type GraphConfig,
  type GraphLine,
} from "@/components/graph";
import React from "react";
import { useWinder2 } from "./useWinder";
import { roundDegreesToDecimals, roundToDecimals } from "@/lib/decimal";
import { TimeSeries } from "@/lib/timeseries";
import { Unit } from "@/control/units";

export function Winder2GraphsPage() {
  const {
    spoolRpm,
    traversePosition,
    traverseState,
    tensionArmAngle,
    pullerSpeed,
    pullerState,
  } = useWinder2();

  const syncHook = useGraphSync(30 * 60 * 1000, "winder2-group");

  return (
    <Page className="pb-20">
      <div className="flex flex-col gap-4">
        <SyncedGraphControls controlProps={syncHook.controlProps} />

        <div className="grid gap-4">
          <SpoolRpmGraph
            syncHook={syncHook}
            newData={spoolRpm}
            unit="rpm"
            renderValue={(value) => roundToDecimals(value, 0)}
          />

          <TraversePositionGraph
            syncHook={syncHook}
            newData={traversePosition}
            limitInner={traverseState?.data.limit_inner}
            limitOuter={traverseState?.data.limit_outer}
            unit="mm"
            renderValue={(value) => roundToDecimals(value, 1)}
          />

          <TensionArmAngleGraph
            syncHook={syncHook}
            newData={tensionArmAngle}
            unit="deg"
            renderValue={(value) => roundDegreesToDecimals(value, 0)}
          />

          <PullerSpeedGraph
            syncHook={syncHook}
            newData={pullerSpeed}
            targetSpeed={pullerState?.data.target_speed}
            unit="m/min"
            renderValue={(value) => roundToDecimals(value, 0)}
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
      newData={newData}
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
      label: "Inner Limit",
      color: "#ef4444",
      dash: [5, 5],
    });
  }

  if (limitOuter !== undefined) {
    lines.push({
      type: "threshold",
      value: limitOuter,
      label: "Outer Limit",
      color: "#ef4444",
      dash: [5, 5],
    });
  }

  const config: GraphConfig = {
    title: "Traverse Position",
    icon: "lu:Move",
    lines,
    colors: {
      primary: "#8b5cf6",
    },
    exportFilename: "traverse_position_data",
  };

  return (
    <AutoSyncedBigGraph
      syncHook={syncHook}
      newData={newData}
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
      newData={newData}
      unit={unit}
      renderValue={renderValue}
      config={config}
      graphId="tension-arm-angle"
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
      color: "#6b7280",
    });
  }

  const config: GraphConfig = {
    title: "Puller Speed",
    icon: "lu:Gauge",
    lines,
    colors: {
      primary: "#06b6d4",
    },
    exportFilename: "puller_speed_data",
  };

  return (
    <AutoSyncedBigGraph
      syncHook={syncHook}
      newData={newData}
      unit={unit}
      renderValue={renderValue}
      config={config}
      graphId="puller-speed"
    />
  );
}
