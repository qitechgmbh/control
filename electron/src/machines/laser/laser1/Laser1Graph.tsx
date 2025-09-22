import { Page } from "@/components/Page";
import {
  AutoSyncedBigGraph,
  SyncedFloatingControlPanel,
  useGraphSync,
  type GraphConfig,
} from "@/components/graph";

import React from "react";
import { useLaser1 } from "./useLaser1";

export function Laser1GraphsPage() {
  const { diameter, x_diameter, y_diameter, roundness, state } = useLaser1();

  const syncHook = useGraphSync("diameter-group");
  const targetDiameter = state?.laser_state?.target_diameter ?? 0;
  const lowerTolerance = state?.laser_state?.lower_tolerance ?? 0;
  const higherTolerance = state?.laser_state?.higher_tolerance ?? 0;

  const isTwoAxis = !!x_diameter?.current || !!y_diameter?.current;

  const diameterColor = "#3b82f6";
  const xDiameterColor = "#ef4444";
  const yDiameterColor = "#22c55e";
  const roundnessColor = "#eab308";

  const config: GraphConfig = {
    title: "Diameter",
    defaultTimeWindow: 30 * 60 * 1000, // 30 minutes
    exportFilename: "diameter_data",
    colors: {
      primary: diameterColor,
      grid: "#e2e8f0",
      axis: "#64748b",
      background: "#ffffff",
    },
  };
  if (!isTwoAxis) {
    return (
      <Page className="pb-27">
        <div className="flex flex-col gap-4">
          <AutoSyncedBigGraph
            syncHook={syncHook}
            newData={{
              newData: diameter,
              color: diameterColor,
              lines: [
                {
                  type: "threshold",
                  value: targetDiameter + higherTolerance,
                  label: "Upper Threshold",
                  color: diameterColor,
                  dash: [5, 5],
                },
                {
                  type: "threshold",
                  value: targetDiameter - lowerTolerance,
                  label: "Lower Threshold",
                  color: diameterColor,
                  dash: [5, 5],
                },
                {
                  type: "target",
                  value: targetDiameter,
                  label: "Target",
                  color: diameterColor,
                },
              ],
            }}
            unit="mm"
            renderValue={(value) => value.toFixed(3)}
            config={config}
            graphId="diameter-graph"
          />
        </div>
        <SyncedFloatingControlPanel controlProps={syncHook.controlProps} />
      </Page>
    );
  } else {
    return (
      <Page className="pb-27">
        <div className="flex flex-col gap-4">
          <AutoSyncedBigGraph
            syncHook={syncHook}
            newData={[
              {
                newData: diameter,
                color: diameterColor,
                title: "Diameter",
                lines: [
                  {
                    type: "threshold",
                    value: targetDiameter + higherTolerance,
                    label: "Upper Threshold",
                    color: diameterColor,
                    dash: [5, 5],
                  },
                  {
                    type: "threshold",
                    value: targetDiameter - lowerTolerance,
                    label: "Lower Threshold",
                    color: diameterColor,
                    dash: [5, 5],
                  },
                  {
                    type: "target",
                    value: targetDiameter,
                    label: "Target",
                    color: diameterColor,
                  },
                ],
              },
              {
                newData: x_diameter,
                color: xDiameterColor,
                title: "X-Diameter",
              },
              {
                newData: y_diameter,
                color: yDiameterColor,
                title: "Y-Diameter",
              },
              {
                newData: roundness,
                color: roundnessColor,
                title: "Roundness",
              },
            ]}
            unit="mm"
            renderValue={(value) => value.toFixed(3)}
            config={config}
            graphId="diameter-graph"
          />
        </div>
        <SyncedFloatingControlPanel controlProps={syncHook.controlProps} />
      </Page>
    );
  }
}
