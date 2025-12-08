import { Page } from "@/components/Page";
import {
  AutoSyncedBigGraph,
  SyncedFloatingControlPanel,
  useGraphSync,
  type GraphConfig,
} from "@/components/graph";
import React from "react";
import { useState } from "react";
import { useMock1 } from "./useMock";
import { TimeSeriesValue, type Series, TimeSeries } from "@/lib/timeseries";

export function Mock1GraphPage() {
  const { sineWaveSum } = useMock1();

  const syncHook = useGraphSync("mock-graphs");
  const graph1Ref = React.useRef<HTMLDivElement | null>(null);
  const [marker, setMarker] = useState<string | null>(null);
  const [markers, setMarkers] = useState<{ timestamp: number; name: string }[]>([]);

  const handleAddMarker = () => {
    const inputEl = document.getElementById("marker-input") as HTMLInputElement;
    if (inputEl && sineWaveSum.current) {
      const ts = sineWaveSum.current.timestamp;
      const name = inputEl.value;
      setMarkers((prev) => [...prev, { timestamp: ts, name }]);
      const tsStr = new Date(ts).toLocaleTimeString("en-GB", { hour12: false });
      setMarker(`${name} @ ${tsStr}`);
    } else {
      setMarker("No data");
    }
  };

  function createMarkerElement(
    timestamp: number,
    value: number,
    name: string,
    startTime: number,
    endTime: number,
    graphWidth: number,
    graphHeight: number,
  ) {
    const ratio = (timestamp - startTime) / (endTime - startTime);
    const xPos = Math.min(Math.max(ratio, 0), 1) * graphWidth;
    const yPos = graphHeight - value;


    const line = document.createElement("div");
    line.style.position = "absolute";
    line.style.left = `${xPos}px`;
    line.style.top = `${yPos}px`;
    line.style.height = `${value}px`;
    line.style.width = "2px";
    line.style.background = "black";
    line.className = "vertical-marker";

    const label = document.createElement("div");
    label.textContent = name;
    label.style.position = "absolute";
    label.style.left = `${xPos}px`;
    label.style.top = `${yPos - 16}px`;
    label.style.transform = "translateX(-50%)";
    label.style.color = "black";
    label.style.fontSize = "12px";
    label.style.padding = "0 2px";
    label.style.whiteSpace = "nowrap";
    label.className = "marker-label";

    return { line, label };
  }

  React.useEffect(() => {
    if (!graph1Ref.current || !sineWaveSum.current) return;
    const graphEl = graph1Ref.current;
    const graphWidth = graphEl.clientWidth;
    const graphHeight = graphEl.clientHeight;

    // Remove previous markers and labels
    graphEl.querySelectorAll(".vertical-marker, .marker-label").forEach((el) => el.remove());

    const endTime = sineWaveSum.current.timestamp;
    const startTime = endTime - (singleGraphConfig.defaultTimeWindow as number);
    const graphMin = -1; // TODO: do it in general case not hardcord
    const graphMax = 1; // TODO: do it in general case not hardcord

    markers.forEach(({ timestamp, name }) => {
      const closest = sineWaveSum.long.values
        .filter((v): v is TimeSeriesValue => v !== null)
        .reduce((prev, curr) =>
          Math.abs(curr.timestamp - timestamp) < Math.abs(prev.timestamp - timestamp) ? curr : prev
        );
      const valueY = ((closest.value - graphMin) / (graphMax - graphMin)) * graphHeight;
      const { line, label } = createMarkerElement(
        timestamp,
        valueY,
        name,
        startTime,
        endTime,
        graphWidth,
        graphHeight,
      );

      graphEl.appendChild(line);
      graphEl.appendChild(label);
    });
  }, [markers, sineWaveSum.current]);

  const config: GraphConfig = {
    title: "Sine Wave",
    defaultTimeWindow: 30 * 60 * 1000,
    exportFilename: "sine_wave_data",
    showLegend: true,
    lines: [],
  };

  // Create inverted sine wave (Sine Wave 2)
  const offsetValues: (TimeSeriesValue | null)[] = sineWaveSum.long.values.map(
    (v) =>
      v !== null ? { value: v.value * -1, timestamp: v.timestamp } : null,
  );

  const series: Series = {
    values: offsetValues,
    index: sineWaveSum.long.index,
    size: sineWaveSum.long.size,
    lastTimestamp: sineWaveSum.long.lastTimestamp,
    timeWindow: sineWaveSum.long.timeWindow,
    sampleInterval: sineWaveSum.long.sampleInterval,
    validCount: sineWaveSum.long.validCount,
  };

  const currentValue: TimeSeriesValue | null =
    sineWaveSum.current !== null
      ? {
          value: sineWaveSum.current.value * -1,
          timestamp: sineWaveSum.current.timestamp,
        }
      : null;

  const sineWave2: TimeSeries = {
    current: currentValue,
    long: series,
    short: sineWaveSum.short,
  };

  const combinedData = [
    {
      newData: sineWaveSum,
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
    newData: sineWaveSum,
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
        <div ref={graph1Ref} className="relative">
          <AutoSyncedBigGraph
            syncHook={syncHook}
            newData={singleData}
            config={singleGraphConfig}
            unit={"mm"}
            renderValue={(value) => value.toFixed(3)}
            graphId="single-graph1"
          />
        </div>
        <div className="flex gap-2">
          Add data marker
          <input
            id="marker-input"
            type="text"
            defaultValue="mymarker1"
            className="border px-2 py-1"
          />
          <button onClick={handleAddMarker} className="px-3 py-1 bg-gray-200">
            Add
          </button>
          <p id="marker-output">{marker ?? "No data"}</p>
        </div>
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
