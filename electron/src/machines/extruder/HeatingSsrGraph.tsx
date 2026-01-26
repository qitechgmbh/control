import React from "react";
import {
  AutoSyncedBigGraph,
  type GraphConfig,
  useGraphSync,
} from "@/components/graph";
import { TimeSeries } from "@/lib/timeseries";
import { GraphColors, HeatingZoneColors, HeatingZoneLabels } from "../types";

type Props = {
  syncHook: ReturnType<typeof useGraphSync>;
  baseConfig: GraphConfig;
  nozzle: TimeSeries | null;
  front: TimeSeries | null;
  middle: TimeSeries | null;
  back: TimeSeries | null;
  graphId: string;
  zoneColors?: HeatingZoneColors;
  zoneLabels?: HeatingZoneLabels;
  graphColors?: GraphColors;
};

export function HeatingSsrGraph({
  syncHook,
  baseConfig,
  nozzle,
  front,
  middle,
  back,
  graphId,
  zoneColors,
  zoneLabels,
  graphColors,
}: Props) {
  const resolvedZoneColors: Required<HeatingZoneColors> = {
    nozzle: zoneColors?.nozzle ?? "#ef4444",
    front: zoneColors?.front ?? "#f59e0b",
    middle: zoneColors?.middle ?? "#8b5cf6",
    back: zoneColors?.back ?? "#3b82f6",
  };

  const resolvedZoneLabels: Required<HeatingZoneLabels> = {
    nozzle: zoneLabels?.nozzle ?? "Nozzle",
    front: zoneLabels?.front ?? "Front",
    middle: zoneLabels?.middle ?? "Middle",
    back: zoneLabels?.back ?? "Back",
  };

  const heatingData = [
    ...(nozzle
      ? [
          {
            newData: nozzle,
            title: `${resolvedZoneLabels.nozzle} SSR`,
            color: resolvedZoneColors.nozzle,
          },
        ]
      : []),
    ...(front
      ? [
          {
            newData: front,
            title: `${resolvedZoneLabels.front} SSR`,
            color: resolvedZoneColors.front,
          },
        ]
      : []),
    ...(middle
      ? [
          {
            newData: middle,
            title: `${resolvedZoneLabels.middle} SSR`,
            color: resolvedZoneColors.middle,
          },
        ]
      : []),
    ...(back
      ? [
          {
            newData: back,
            title: `${resolvedZoneLabels.back} SSR`,
            color: resolvedZoneColors.back,
          },
        ]
      : []),
  ];

  const config: GraphConfig = {
    ...baseConfig,
    title: "Heating SSR",
    exportFilename: "heating_ssr_data",
    colors: graphColors ?? baseConfig.colors,
  };

  return (
    <AutoSyncedBigGraph
      syncHook={syncHook}
      newData={heatingData}
      config={config}
      renderValue={(value) => (value >= 0.5 ? "On" : "Off")}
      graphId={graphId}
    />
  );
}
