import { Page } from "@/components/Page";
import {
  AutoSyncedBigGraph,
  SyncedFloatingControlPanel,
  useGraphSync,
  type GraphConfig,
} from "@/components/graph";
import React from "react";
import { useMock1 } from "./useMock";
import { TimeSeriesValue, type Series, TimeSeries } from "@/lib/timeseries";

export function Mock1GraphPage() {
  const { sineWave } = useMock1();

  const syncHook = useGraphSync(30 * 60 * 1000, "mock-graphs");

  const config: GraphConfig = {
    title: "Sine Wave",
    defaultTimeWindow: 30 * 60 * 1000,
    exportFilename: "sine_wave_data",
    showLegend: true,
    // Add lines configuration
    lines: [
      {
        value: 0.5,
        label: "Upper Threshold",
        color: "#ef4444", // Red
        type: "threshold",
        show: true,
      },
      {
        value: -0.5,
        label: "Lower Threshold",
        color: "#ef4444", // Red
        type: "threshold",
        show: true,
      },
      {
        value: 0,
        label: "Target Line",
        color: "#10b981", // Green
        type: "target",
        show: true,
      },
    ],
  };
  const offsetValues: (TimeSeriesValue | null)[] = sineWave.long.values.map(
    (v) =>
      v !== null ? { value: v.value * -1, timestamp: v.timestamp } : null,
  );

  const series: Series = {
    values: offsetValues,
    index: sineWave.long.index,
    size: sineWave.long.size,
    lastTimestamp: sineWave.long.lastTimestamp,
    timeWindow: sineWave.long.timeWindow,
    sampleInterval: sineWave.long.sampleInterval,
    validCount: sineWave.long.validCount,
  };
  const currentValue: TimeSeriesValue | null =
    sineWave.current !== null
      ? {
          value: sineWave.current.value * -1,
          timestamp: sineWave.current.timestamp,
        }
      : null;
  const sineWave2: TimeSeries = {
    current: currentValue,
    long: series,
    short: sineWave.short,
  };
  // Combined data for 3 sine waves
  const combinedData = [
    { newData: sineWave, title: "Sine Wave 1", color: "#3b82f6" },
    { newData: sineWave2, title: "Sine Wave 2", color: "#ef4444" },
  ];

  // Single sine wave data with different lines
  const singleData = { newData: sineWave };

  const singleGraphConfig: GraphConfig = {
    ...config,
    title: "Sine Wave 3",
    lines: [
      {
        value: 0.3,
        label: "Safe Zone Upper",
        color: "#3b82f6", // Blue
        type: "threshold",
        show: true,
      },
      {
        value: -0.3,
        label: "Safe Zone Lower",
        color: "#3b82f6", // Blue
        type: "threshold",
        show: true,
      },
    ],
  };

  return (
    <Page className="pb-25">
      <div className="flex flex-col gap-4">
        <AutoSyncedBigGraph
          syncHook={syncHook}
          newData={combinedData}
          config={{ ...config, title: "Combined Sine Waves (1, 2)" }}
          unit={"mm"}
          renderValue={(value) => value.toFixed(3)}
          graphId="combined-graph"
        />

        <AutoSyncedBigGraph
          syncHook={syncHook}
          newData={singleData}
          config={singleGraphConfig}
          unit={"mm"}
          renderValue={(value) => value.toFixed(3)}
          graphId="single-graph"
        />
      </div>

      <SyncedFloatingControlPanel controlProps={syncHook.controlProps} />
    </Page>
  );
}
