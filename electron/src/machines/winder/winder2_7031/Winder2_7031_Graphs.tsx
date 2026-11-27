import { Page } from "@/components/layout/Page";
import {
  BigGraph,
  GraphHistoryProvider,
  GraphControlPanel,
  useGraphSync,
  type GraphConfig,
  type GraphLine,
} from "@/components/graph";
import React from "react";
import { useWinder2 } from "./useWinder";
import { roundDegreesToDecimals, roundToDecimals } from "@/lib/decimal";
import { TimeSeries } from "@/lib/timeseries";
import { Unit } from "@/control/units";

export function Winder2_7031GraphsPage() {
  const {
    machineIdentification,
    state,
    spoolRpm,
    traversePosition,
    tensionArmAngle,
    pullerSpeed,
    targetPullerSpeed,
    spoolProgress,
  } = useWinder2();

  const syncHook = useGraphSync("winder2-group");

  return (
    <Page className="pb-27">
      <GraphHistoryProvider machine={machineIdentification}>
        <div className="flex flex-col gap-4">
          <div className="grid gap-4">
            <PullerSpeedGraph
              syncHook={syncHook}
              newData={pullerSpeed}
              targetSeries={targetPullerSpeed}
              targetSpeed={state?.puller_state?.target_speed}
              unit="m/min"
              renderValue={(value) => roundToDecimals(value, 1)}
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
          </div>
        </div>

        <GraphControlPanel syncHook={syncHook} />
      </GraphHistoryProvider>
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
  };

  return (
    <BigGraph
      syncHook={syncHook}
      newData={{
        newData,
        history: "spool.rpm",
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
  };

  return (
    <BigGraph
      syncHook={syncHook}
      newData={{
        newData,
        history: "traverse.position",
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
  };

  return (
    <BigGraph
      syncHook={syncHook}
      newData={{
        newData,
        history: "tension_arm.angle",
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
  };

  // NOTE: Assuming this graph starts at 0, and the max is the total capacity.
  return (
    <BigGraph
      syncHook={syncHook}
      newData={{
        newData,
        history: "spool.progress",
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
  targetSeries,
  targetSpeed,
  unit,
  renderValue,
}: {
  syncHook: ReturnType<typeof useGraphSync>;
  newData: TimeSeries | null;
  targetSeries?: TimeSeries;
  targetSpeed?: number;
  unit?: Unit;
  renderValue?: (value: number) => string;
}) {
  const lines: GraphLine[] = [];

  if (targetSpeed !== undefined) {
    lines.push({
      type: "target",
      value: targetSpeed,
      targetSeries,
      history: "puller.speed_controller.speed_desired",
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
  };

  return (
    <BigGraph
      syncHook={syncHook}
      newData={{
        newData,
        history: "puller.speed",
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
