import { Page } from "@/components/Page";
import React from "react";
import {
  BigGraph,
  GraphLine,
  GraphSyncProvider,
  GraphConfig,
  FloatingExportButton,
} from "@/helpers/BigGraph";
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

  return (
    <Page>
      <GraphSyncProvider groupId="winder2-group">
        <div className="grid gap-4">
          <SpoolRpmGraph
            newData={spoolRpm}
            unit="rpm"
            renderValue={(value) => roundToDecimals(value, 0)}
          />

          <TraversePositionGraph
            newData={traversePosition}
            limitInner={traverseState?.data.limit_inner}
            limitOuter={traverseState?.data.limit_outer}
            unit="mm"
            renderValue={(value) => roundToDecimals(value, 1)}
          />

          <TensionArmAngleGraph
            newData={tensionArmAngle}
            unit="deg"
            renderValue={(value) => roundDegreesToDecimals(value, 0)}
          />

          <PullerSpeedGraph
            newData={pullerSpeed}
            targetSpeed={pullerState?.data.target_speed}
            unit="m/min"
            renderValue={(value) => roundToDecimals(value, 0)}
          />
        </div>

        <FloatingExportButton groupId="winder2-group" />
      </GraphSyncProvider>
    </Page>
  );
}

export function SpoolRpmGraph({
  newData,
  unit,
  renderValue,
}: {
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
    <BigGraph
      newData={newData}
      unit={unit}
      renderValue={renderValue}
      config={config}
      syncGroupId="winder2-group"
      graphId="spool-rpm"
    />
  );
}

export function TraversePositionGraph({
  newData,
  limitInner,
  limitOuter,
  unit,
  renderValue,
}: {
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
    <BigGraph
      newData={newData}
      unit={unit}
      renderValue={renderValue}
      config={config}
      syncGroupId="winder2-group"
      graphId="traverse-position"
    />
  );
}

export function TensionArmAngleGraph({
  newData,
  unit,
  renderValue,
}: {
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
    <BigGraph
      newData={newData}
      unit={unit}
      renderValue={renderValue}
      config={config}
      syncGroupId="winder2-group"
      graphId="tension-arm-angle"
    />
  );
}

export function PullerSpeedGraph({
  newData,
  targetSpeed,
  unit,
  renderValue,
}: {
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
    <BigGraph
      newData={newData}
      unit={unit}
      renderValue={renderValue}
      config={config}
      syncGroupId="winder2-group"
      graphId="puller-speed"
    />
  );
}
