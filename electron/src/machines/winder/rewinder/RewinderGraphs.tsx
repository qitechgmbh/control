import { Page } from "@/components/layout/Page";
import {
  BigGraph,
  GraphControlPanel,
  useGraphSync,
  type GraphConfig,
  type GraphLine,
} from "@/components/graph";
import { IconName } from "@/components/icons/Icon";
import { Unit } from "@/control/units";
import { roundDegreesToDecimals, roundToDecimals } from "@/lib/decimal";
import { TimeSeries } from "@/lib/timeseries";
import React from "react";
import { useRewinder } from "./useRewinder";

export function RewinderGraphsPage() {
  const {
    state,
    pullerSpeed,
    takeupSpoolRpm,
    sourceSpoolRpm,
    takeupTensionArmAngle,
    sourceTensionArmAngle,
    traversePosition,
    rewindProgress,
  } = useRewinder();
  const syncHook = useGraphSync("rewinder-group");

  return (
    <Page className="pb-27">
      <div className="grid gap-4">
        <SeriesGraph
          syncHook={syncHook}
          graphId="rewinder-puller-speed"
          title="Puller Speed"
          icon="lu:Gauge"
          color="#2563eb"
          data={pullerSpeed}
          unit="m/min"
          renderValue={(value) => roundToDecimals(value, 1)}
        />
        <SeriesGraph
          syncHook={syncHook}
          graphId="rewinder-takeup-arm"
          title="Takeup Tension Arm"
          icon="lu:RotateCw"
          color="#f59e0b"
          data={takeupTensionArmAngle}
          unit="deg"
          renderValue={(value) => roundDegreesToDecimals(value, 0)}
        />
        <SeriesGraph
          syncHook={syncHook}
          graphId="rewinder-source-arm"
          title="Source Tension Arm"
          icon="lu:RotateCcw"
          color="#ef4444"
          data={sourceTensionArmAngle}
          unit="deg"
          renderValue={(value) => roundDegreesToDecimals(value, 0)}
        />
        <SeriesGraph
          syncHook={syncHook}
          graphId="rewinder-takeup-spool"
          title="Takeup Spool Speed"
          icon="lu:RefreshCw"
          color="#10b981"
          data={takeupSpoolRpm}
          unit="rpm"
          renderValue={(value) => roundToDecimals(value, 0)}
        />
        <SeriesGraph
          syncHook={syncHook}
          graphId="rewinder-source-spool"
          title="Source Spool Speed"
          icon="lu:RefreshCcw"
          color="#06b6d4"
          data={sourceSpoolRpm}
          unit="rpm"
          renderValue={(value) => roundToDecimals(value, 0)}
        />
        <SeriesGraph
          syncHook={syncHook}
          graphId="rewinder-traverse"
          title="Traverse Position"
          icon="lu:Move"
          color="#8b5cf6"
          data={traversePosition}
          unit="mm"
          renderValue={(value) => roundToDecimals(value, 1)}
          lines={[
            {
              type: "threshold",
              value: state?.traverse_state.limit_inner ?? 0,
              color: "#8b5cf6",
              dash: [5, 5],
            },
            {
              type: "threshold",
              value: state?.traverse_state.limit_outer ?? 0,
              color: "#8b5cf6",
              dash: [5, 5],
            },
          ]}
        />
        <SeriesGraph
          syncHook={syncHook}
          graphId="rewinder-progress"
          title="Rewind Progress"
          icon="lu:Ruler"
          color="#64748b"
          data={rewindProgress}
          unit="m"
          renderValue={(value) => roundToDecimals(value, 2)}
        />
      </div>

      <GraphControlPanel syncHook={syncHook} />
    </Page>
  );
}

function SeriesGraph({
  syncHook,
  graphId,
  title,
  icon,
  color,
  data,
  unit,
  renderValue,
  lines = [],
}: {
  syncHook: ReturnType<typeof useGraphSync>;
  graphId: string;
  title: string;
  icon?: IconName;
  color: string;
  data: TimeSeries | null;
  unit?: Unit;
  renderValue?: (value: number) => string;
  lines?: GraphLine[];
}) {
  const config: GraphConfig = {
    title,
    icon,
    colors: { primary: color },
  };

  return (
    <BigGraph
      syncHook={syncHook}
      newData={{ newData: data, color, lines }}
      unit={unit}
      renderValue={renderValue}
      config={config}
      graphId={graphId}
    />
  );
}
