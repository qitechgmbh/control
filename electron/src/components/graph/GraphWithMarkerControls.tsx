import React, { useState, useRef, useCallback, useEffect } from "react";
import {
  AutoSyncedBigGraph,
  useGraphSync,
  type GraphConfig,
} from "@/components/graph";
import { TimeSeries, TimeSeriesValue } from "@/lib/timeseries";
import { Unit } from "@/control/units";
import { useMarkerManager } from "./useMarkerManager";
import { MarkerProvider, useMarkerContext } from "./MarkerContext";

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
  machineId?: string;
  markers?: Array<{
    timestamp: number;
    name: string;
    value?: number;
    color?: string;
  }>;
};

function createMarkerElement(
  timestamp: number,
  name: string,
  value: number,
  graphMin: number,
  graphMax: number,
  startTime: number,
  endTime: number,
  graphWidth: number,
  graphHeight: number,
  color?: string,
) {
  // Calculate the X position of the timestamp
  const ratio = (timestamp - startTime) / (endTime - startTime);
  const xPos = Math.min(Math.max(ratio, 0), 1) * graphWidth;

  // Calculate the Y position of the value (from bottom of graph)
  const normalizedValue = (value - graphMin) / (graphMax - graphMin);
  const valueY = graphHeight - normalizedValue * graphHeight;

  // Create vertical line that spans full height (shows time position)
  const line = document.createElement("div");
  line.style.position = "absolute";
  line.style.left = `${xPos}px`;
  line.style.top = "0px";
  line.style.height = `${graphHeight}px`;
  line.style.width = "2px";
  // Use custom color if provided, otherwise default gray
  const lineColor = color || "rgba(0, 0, 0, 0.5)";
  line.style.background = lineColor;
  line.className = "vertical-marker";

  // Create a point at the actual data value position
  const point = document.createElement("div");
  point.style.position = "absolute";
  point.style.left = `${xPos}px`;
  point.style.top = `${valueY}px`;
  point.style.width = "8px";
  point.style.height = "8px";
  point.style.borderRadius = "50%";
  // Use custom color if provided, otherwise default black
  const pointColor = color || "rgba(0, 0, 0, 0.8)";
  point.style.background = pointColor;
  point.style.transform = "translate(-50%, -50%)";
  point.style.border = "2px solid white";
  point.className = "marker-point";

  const label = document.createElement("div");
  label.textContent = name;
  label.style.position = "absolute";
  label.style.left = `${xPos}px`;
  label.style.top = `${graphHeight + 5}px`;
  label.style.transform = "translateX(-50%)";
  label.style.color = "black";
  label.style.padding = "2px 4px";
  label.style.fontSize = "12px";
  label.style.whiteSpace = "nowrap";
  label.className = "marker-label";

  return { line, point, label };
}

function GraphWithMarkerControlsContent({
  syncHook,
  newData,
  config,
  unit,
  renderValue,
  graphId,
  currentTimeSeries,
  machineId: providedMachineId,
  markers: providedMarkers,
  graphWrapperRef,
}: GraphWithMarkerControlsProps & {
  graphWrapperRef: React.RefObject<HTMLDivElement | null>;
}) {
  const { setMachineId, setCurrentTimestamp } = useMarkerContext();

  // Auto-detect machineId from graphId if not provided (extract base name)
  // e.g., "pressure-graph" -> "pressure", "extruder-graphs" -> "extruder-graphs"
  const machineId = providedMachineId || graphId.split("-")[0] || "default";

  // Update context with machineId and current timestamp
  useEffect(() => {
    setMachineId(machineId);
  }, [machineId, setMachineId]);

  useEffect(() => {
    if (currentTimeSeries?.current?.timestamp) {
      setCurrentTimestamp(currentTimeSeries.current.timestamp);
    }
  }, [currentTimeSeries?.current?.timestamp, setCurrentTimestamp]);

  // Use provided markers or load from marker manager
  const markerManager = useMarkerManager(machineId);
  const markers = providedMarkers || markerManager.markers;

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

    // Remove previous markers, points and labels from the overlay container
    overlayContainer
      .querySelectorAll(".vertical-marker, .marker-point, .marker-label")
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

    markers.forEach(({ timestamp, name, value, color }) => {
      if (timestamp >= startTime && timestamp <= endTime) {
        // Find the data point closest to the marker timestamp to get the correct Y-value
        // If value is not provided, use the closest data point
        let markerValue = value;
        if (markerValue === undefined && currentTimeSeries) {
          const validValues = currentTimeSeries.long.values.filter(
            (v): v is TimeSeriesValue => v !== null,
          );
          if (validValues.length > 0) {
            const closest = validValues.reduce((prev, curr) =>
              Math.abs(curr.timestamp - timestamp) <
              Math.abs(prev.timestamp - timestamp)
                ? curr
                : prev,
            );
            markerValue = closest.value;
          }
        }

        // Use a default value if still undefined
        if (markerValue === undefined) {
          markerValue = (graphMin + graphMax) / 2;
        }

        // Create marker element (full height line + point at data value)
        const { line, point, label } = createMarkerElement(
          timestamp,
          name,
          markerValue,
          graphMin,
          graphMax,
          startTime,
          endTime,
          graphWidth,
          graphHeight,
          color,
        );

        overlayContainer.appendChild(line);
        overlayContainer.appendChild(point);
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

  // Use original config without adding marker lines (markers are overlay elements)
  const finalConfig = config;

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
    </div>
  );
}

export function GraphWithMarkerControls(props: GraphWithMarkerControlsProps) {
  const graphWrapperRef = useRef<HTMLDivElement | null>(null);

  // Use context if available (from SyncedFloatingControlPanel), otherwise work without it
  return (
    <GraphWithMarkerControlsContent
      {...props}
      graphWrapperRef={graphWrapperRef}
    />
  );
}
