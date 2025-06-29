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
    lines: [],
  };

  // Create inverted sine wave (Sine Wave 2)
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

  const combinedData = [
    {
      newData: sineWave,
      title: "Sine Wave 1",
      color: "#3b82f6",
      lines: [
        {
          type: "threshold" as const,
          value: 0.8,
          color: "#3b82f6",
          show: true,
          width: 2,
        },
      ],
    },
    {
      newData: sineWave2,
      title: "Sine Wave 2",
      color: "#ef4444",
      lines: [
        {
          type: "target" as const,
          value: -0.3,
          color: "#ef4444",
          show: true,
          width: 1,
        },
      ],
    },
  ];

  // Single sine wave data
  const singleData = {
    newData: sineWave,
    title: "Sine Wave",
    color: "#8b5cf6",
  };

  const singleGraphConfig: GraphConfig = {
    ...config,
    title: "Sine Wave",
  };

  return (
    <Page className="pb-27">
      <div className="flex flex-col gap-4">
        <AutoSyncedBigGraph
          syncHook={syncHook}
          newData={singleData}
          config={singleGraphConfig}
          unit={"mm"}
          renderValue={(value) => value.toFixed(3)}
          graphId="single-graph1"
        />
        <AutoSyncedBigGraph
          syncHook={syncHook}
          newData={combinedData}
          config={{
            ...config,
            title: "Combined Sine Waves",
          }}
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
          graphId="single-graph2"
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
