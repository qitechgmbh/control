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

  const syncHook = useGraphSync("diameter-roundness-group");
  const targetDiameter = state?.laser_state?.target_diameter ?? 0;
  const lowerTolerance = state?.laser_state?.lower_tolerance ?? 0;
  const higherTolerance = state?.laser_state?.higher_tolerance ?? 0;

  const isTwoAxis = !!x_diameter?.current || !!y_diameter?.current;

  // Convert roundness from ratio (0-1) to percentage (0-100)
  const roundnessPercent = React.useMemo(() => {
    if (!roundness) return null;

    const transformValue = (
      val: { value: number; timestamp: number } | null,
    ) => {
      if (!val) return null;
      return { value: val.value * 100, timestamp: val.timestamp };
    };

    const transformSeries = (series: any) => {
      if (!series || !series.values) return series;
      return {
        ...series,
        values: series.values.map(transformValue),
      };
    };

    return {
      current: roundness.current ? transformValue(roundness.current) : null,
      long: transformSeries(roundness.long),
      short: transformSeries(roundness.short),
    };
  }, [roundness]);

  const diameterColor = "#3b82f6"; // blue
  const xDiameterColor = "#ef4444"; // red
  const yDiameterColor = "#22c55e"; // green
  const roundnessColor = "#eab308"; // yellow

  // config for all graphs
  const baseGraphConfig: GraphConfig = {
    title: "",
    defaultTimeWindow: 30 * 60 * 1000, // 30 minutes Standard
    colors: {
      grid: "#e2e8f0",
      axis: "#64748b",
      background: "#ffffff",
    },
  };

  // Diameter-Graph (Diameter, X-Diameter, Y-Diameter)
  const diameterGraphData = [
    {
      newData: diameter,
      color: diameterColor,
      title: "Diameter",
      lines: [
        // Tolerance for diameter
        {
          type: "threshold" as const,
          value: targetDiameter + higherTolerance,
          label: "Upper Tolerance",
          color: diameterColor,
          dash: [5, 5],
        },
        {
          type: "threshold" as const,
          value: targetDiameter - lowerTolerance,
          label: "Lower Tolerance",
          color: diameterColor,
          dash: [5, 5],
        },
        {
          type: "target" as const,
          value: targetDiameter,
          label: "Target",
          color: diameterColor,
        },
      ],
    },

    ...(isTwoAxis && x_diameter
      ? [{ newData: x_diameter, color: xDiameterColor, title: "X-Diameter" }]
      : []),
    ...(isTwoAxis && y_diameter
      ? [{ newData: y_diameter, color: yDiameterColor, title: "Y-Diameter" }]
      : []),
  ];

  // Roundness-Graph
  const roundnessGraphData = {
    newData: roundnessPercent,
    color: roundnessColor,
    title: "Roundness (%)",
  };

  return (
    <Page className="pb-27">
      <div className="flex flex-col gap-4">
        {/* Graph 1: Diameter */}
        <AutoSyncedBigGraph
          syncHook={syncHook}
          newData={diameterGraphData}
          unit="mm"
          renderValue={(value) => value.toFixed(3)}
          config={{
            ...baseGraphConfig,
            title: "Diameter (mm)",
            exportFilename: "diameter_data",
            colors: {
              ...baseGraphConfig.colors,
              primary: diameterColor,
            },
          }}
          graphId="diameter-graph"
        />

        {/* Graph 2: Roundness */}
        {isTwoAxis && roundnessPercent && (
          <AutoSyncedBigGraph
            syncHook={syncHook}
            newData={roundnessGraphData}
            unit="%"
            renderValue={(value) => value.toFixed(2)}
            config={{
              ...baseGraphConfig,
              title: "Roundness (%)",
              exportFilename: "roundness_data",
              colors: {
                ...baseGraphConfig.colors,
                primary: roundnessColor,
              },
            }}
            graphId="roundness-graph"
          />
        )}
      </div>
      <SyncedFloatingControlPanel controlProps={syncHook.controlProps} />
    </Page>
  );
}
