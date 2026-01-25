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
  name: string,
  value: number,
  graphMin: number,
  graphMax: number,
  startTime: number,
  endTime: number,
  graphWidth: number,
  graphHeight: number,
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
  line.style.background = "rgba(0, 0, 0, 0.5)";
  line.className = "vertical-marker";

  // Create a point at the actual data value position
  const point = document.createElement("div");
  point.style.position = "absolute";
  point.style.left = `${xPos}px`;
  point.style.top = `${valueY}px`;
  point.style.width = "8px";
  point.style.height = "8px";
  point.style.borderRadius = "50%";
  point.style.background = "rgba(0, 0, 0, 0.8)";
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
  
  // Load markers from localStorage on mount and clean up old ones
  const loadMarkersFromStorage = useCallback((): {
    timestamp: number;
    name: string;
    value: number;
  }[] => {
    try {
      const storageKey = `graph-markers-${graphId}`;
      const stored = localStorage.getItem(storageKey);
      if (stored) {
        const allMarkers: {
          timestamp: number;
          name: string;
          value: number;
        }[] = JSON.parse(stored);
        
        // Remove markers older than 7 days to save storage space
        const sevenDaysAgo = Date.now() - 7 * 24 * 60 * 60 * 1000;
        const recentMarkers = allMarkers.filter(
          (marker) => marker.timestamp >= sevenDaysAgo,
        );
        
        // Limit to max 100 markers per graph to prevent storage bloat
        const maxMarkers = 100;
        const limitedMarkers =
          recentMarkers.length > maxMarkers
            ? recentMarkers.slice(-maxMarkers)
            : recentMarkers;
        
        // Save cleaned markers back if we removed any
        if (limitedMarkers.length !== allMarkers.length) {
          localStorage.setItem(storageKey, JSON.stringify(limitedMarkers));
        }
        
        return limitedMarkers;
      }
    } catch (error) {
      console.warn("Failed to load markers from localStorage:", error);
    }
    return [];
  }, [graphId]);

  const [markers, setMarkers] = useState<
    { timestamp: number; name: string; value: number }[]
  >(loadMarkersFromStorage);
  const [statusMessage, setStatusMessage] = useState<string | null>(null);

  // Save markers to localStorage whenever they change, with limits
  useEffect(() => {
    try {
      const storageKey = `graph-markers-${graphId}`;
      
      // Limit to max 100 markers per graph
      const maxMarkers = 100;
      const markersToSave =
        markers.length > maxMarkers ? markers.slice(-maxMarkers) : markers;
      
      localStorage.setItem(storageKey, JSON.stringify(markersToSave));
    } catch (error) {
      console.warn("Failed to save markers to localStorage:", error);
    }
  }, [markers, graphId]);

  // Markers are rendered as overlay elements, not as graph lines

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

    markers.forEach(({ timestamp, name, value }) => {
      if (timestamp >= startTime && timestamp <= endTime) {
        // Create marker element (full height line + point at data value)
        const { line, point, label } = createMarkerElement(
          timestamp,
          name,
          value,
          graphMin,
          graphMax,
          startTime,
          endTime,
          graphWidth,
          graphHeight,
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
