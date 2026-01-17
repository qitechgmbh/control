import React, { useState, useRef, useCallback, useEffect } from "react";
import {
  AutoSyncedBigGraph,
  useGraphSync,
  type GraphConfig,
} from "@/components/graph";
import { TimeSeries, TimeSeriesValue } from "@/lib/timeseries";
import { Unit } from "@/control/units";

type TimeSeriesData = {
  newData: TimeSeries | null;
  title?: string;
  color?: string;
  lines?: any[];
};

type GraphWithMarkerControlsProps = {
  syncHook: ReturnType<typeof useGraphSync>;
  newData: TimeSeriesData | TimeSeriesData[];
  config: GraphConfig;
  unit?: Unit;
  renderValue?: (value: number) => string;
  graphId: string;
  currentTimeSeries: TimeSeries | null;
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
  // Calculate the position of the timestamp
  const ratio = (timestamp - startTime) / (endTime - startTime);
  const xPos = Math.min(Math.max(ratio, 0), 1) * graphWidth;
  const yPos = graphHeight - value;

  const line = document.createElement("div");
  line.style.position = "absolute";
  line.style.left = `${xPos}px`;
  line.style.top = `${yPos}px`;
  line.style.height = `${value}px`;
  line.style.width = "2px";
  line.style.background = "rgba(0, 0, 0, 0.5)";
  line.className = "vertical-marker";

  const label = document.createElement("div");
  label.textContent = name;
  label.style.position = "absolute";
  label.style.left = `${xPos}px`;
  label.style.top = `${yPos - 20}px`;
  label.style.transform = "translateX(-50%)";
  label.style.color = "black";
  label.style.padding = "2px 4px";
  label.style.fontSize = "12px";
  label.style.whiteSpace = "nowrap";
  label.className = "marker-label";

  return { line, label };
}

export function GraphWithMarkerControls({
  syncHook,
  newData,
  config,
  unit,
  renderValue,
  graphId,
  currentTimeSeries,
}: GraphWithMarkerControlsProps) {
  const graphWrapperRef = useRef<HTMLDivElement | null>(null);
  const [markerName, setMarkerName] = useState("");
  const [markers, setMarkers] = useState<
    { timestamp: number; name: string; value: number }[]
  >([]);
  const [statusMessage, setStatusMessage] = useState<string | null>(null);

  const dynamicMarkerLines = markers.map((marker, index) => ({
    type: "user_marker" as const,
    value: marker.value,
    label: marker.name,
    color: "#ff0000",
    width: 2,
    show: true,
    markerTimestamp: marker.timestamp,
  }));

  // Time Tick for forcing marker redraw
  const [timeTick, setTimeTick] = useState(0);

  // Set interval to force redraw the marker effect frequently (e.g., every 50ms)
  useEffect(() => {
    if (!currentTimeSeries?.current) return;
    const intervalId = setInterval(() => {
      setTimeTick((prev) => prev + 1);
    }, 50);
    return () => clearInterval(intervalId);
  }, [currentTimeSeries?.current]);

  const handleAddMarker = useCallback(() => {
    if (currentTimeSeries?.current && markerName.trim()) {
      const ts = currentTimeSeries.current.timestamp;
      const val = currentTimeSeries.current.value;
      const name = markerName.trim();

      setMarkers((prev) => [...prev, { timestamp: ts, name, value: val }]);
    }
  }, [currentTimeSeries, markerName]);

  // Marker Drawing Effect
  useEffect(() => {
    if (!graphWrapperRef.current || !currentTimeSeries?.current) return;

    const graphEl = graphWrapperRef.current;
    // Find chart container via uPlot canvas (canvas is always a direct child)
    const canvas = graphEl.querySelector("canvas");
    if (!canvas) return;
    const chartContainer = canvas.parentElement;
    if (!chartContainer) return;

    const graphWidth = chartContainer.clientWidth;
    const graphHeight = chartContainer.clientHeight;

    const overlayContainer = chartContainer.parentElement;
    if (!overlayContainer) return;

    // Remove previous markers and labels from the overlay container
    overlayContainer
      .querySelectorAll(".vertical-marker, .marker-label")
      .forEach((el) => el.remove());

    // Get the visible time window
    const currentTimeWindow = syncHook.controlProps.timeWindow;
    const defaultDuration = config.defaultTimeWindow as number;
    const validTimeWindowMs =
      (typeof currentTimeWindow === "number" && currentTimeWindow) ||
      defaultDuration || // Fallback to config default
      30 * 60 * 1000; // Final fallback (30 minutes)

    const endTime = currentTimeSeries.current.timestamp;
    const startTime = endTime - validTimeWindowMs;

    // Calculate Y-axis scale from visible data (similar to createChart.ts)
    const visibleValues: number[] = [];

    // Collect values from the time series in the visible time window
    currentTimeSeries.long.values
      .filter((v): v is TimeSeriesValue => v !== null)
      .forEach((v) => {
        if (v.timestamp >= startTime && v.timestamp <= endTime) {
          visibleValues.push(v.value);
        }
      });

    // Include config lines in the scale calculation
    config.lines?.forEach((line) => {
      if (line.show !== false) {
        visibleValues.push(line.value);
      }
    });

    // Calculate min/max with 10% padding (matching createChart.ts behavior)
    let graphMin: number, graphMax: number;
    if (visibleValues.length > 0) {
      const minY = Math.min(...visibleValues);
      const maxY = Math.max(...visibleValues);
      const range = maxY - minY || Math.abs(maxY) * 0.1 || 1;
      graphMin = minY - range * 0.1;
      graphMax = maxY + range * 0.1;
    } else {
      // Fallback if no data is available
      graphMin = -1;
      graphMax = 1;
    }

    markers.forEach(({ timestamp, name }) => {
      if (timestamp >= startTime && timestamp <= endTime) {
        // Find the data point closest to the marker timestamp to get the correct Y-value
        const validValues = currentTimeSeries.long.values.filter(
          (v): v is TimeSeriesValue => v !== null,
        );
        if (validValues.length === 0) return;

        const closest = validValues.reduce((prev, curr) =>
          Math.abs(curr.timestamp - timestamp) <
          Math.abs(prev.timestamp - timestamp)
            ? curr
            : prev,
        );

        // Calculate the Y-position in pixels from the bottom of the chart area
        const normalizedValue =
          (closest.value - graphMin) / (graphMax - graphMin);
        const valueY = normalizedValue * graphHeight;

        const { line, label } = createMarkerElement(
          timestamp,
          valueY,
          name,
          startTime,
          endTime,
          graphWidth,
          graphHeight,
        );

        overlayContainer.appendChild(line);
        overlayContainer.appendChild(label);
      }
    });
  }, [
    markers,
    currentTimeSeries,
    timeTick,
    config.defaultTimeWindow,
    syncHook.controlProps.timeWindow,
  ]);

  const finalConfig = {
    ...config,
    lines: [...(config.lines || []), ...dynamicMarkerLines],
  };

  return (
    <div className="flex flex-col gap-2">
      <div ref={graphWrapperRef} className="relative">
        {/* Render the core chart component */}
        <AutoSyncedBigGraph
          syncHook={syncHook}
          newData={newData}
          config={finalConfig}
          unit={unit}
          renderValue={renderValue}
          graphId={graphId}
        />
      </div>

      {/* Marker Input and Button */}
      <div className="flex items-center gap-2">
        <span className="font-medium">Add Marker:</span>
        <input
          type="text"
          placeholder={`Marker for ${config.title}`}
          value={markerName}
          onChange={(e) => setMarkerName(e.target.value)}
          className="rounded border px-2 py-1"
        />
        <button
          onClick={handleAddMarker}
          className="rounded bg-gray-200 px-3 py-1 hover:bg-gray-300"
          disabled={!currentTimeSeries?.current}
        >
          Add
        </button>
        <p className="ml-4 text-sm text-gray-600">{statusMessage ?? ""}</p>
      </div>
    </div>
  );
}
