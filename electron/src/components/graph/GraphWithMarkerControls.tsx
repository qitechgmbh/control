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
  const [markers, setMarkers] = useState<{ timestamp: number; name: string; value: number }[]>([])
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
        setTimeTick(prev => prev + 1);
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
    // The BigGraph component is the first child (the one with the actual chart)
    // TODO: Find a better way to do this
    const chartContainer = graphEl.querySelector(".h-\\[50vh\\] > div > div.flex-1 > div");
    if (!chartContainer) return; 

    const graphWidth = chartContainer.clientWidth;
    const graphHeight = chartContainer.clientHeight;
    
    const overlayContainer = chartContainer.parentElement;
    if (!overlayContainer) return;

    // Remove previous markers and labels from the overlay container
    overlayContainer.querySelectorAll(".vertical-marker, .marker-label").forEach((el) => el.remove());

    // Get the visible time window
    const currentTimeWindow = syncHook.controlProps.timeWindow;  
    const defaultDuration = config.defaultTimeWindow as number;
    const validTimeWindowMs = 
        (typeof currentTimeWindow === 'number' && currentTimeWindow) || 
        defaultDuration || // Fallback to config default
        (30 * 60 * 1000); // Final fallback (30 minutes)
        
    const endTime = currentTimeSeries.current.timestamp; 
    const startTime = endTime - validTimeWindowMs; 

    // Assuming the graph's fixed Y-scale is from -1 to 1 based on the sine wave example
    const graphMin = -1; 
    const graphMax = 1; 
    // TODO: For real-world graphs (like Winder), you might need to read the actual min/max scale 
    // from the uPlot instance or define a safe range if the data is unconstrained.

    markers.forEach(({ timestamp, name }) => {
      if (timestamp >= startTime && timestamp <= endTime) {
          // Find the data point closest to the marker timestamp to get the correct Y-value
          const closest = currentTimeSeries.long.values
              .filter((v): v is TimeSeriesValue => v !== null)
              .reduce((prev, curr) =>
                Math.abs(curr.timestamp - timestamp) < Math.abs(prev.timestamp - timestamp) ? curr : prev
              );
          if (!closest) return; 

          // Calculate the Y-position in pixels from the bottom of the chart area
          const normalizedValue = (closest.value - graphMin) / (graphMax - graphMin);
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
  }, [markers, currentTimeSeries, timeTick, config.defaultTimeWindow, syncHook.controlProps.timeWindow]);

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
      <div className="flex gap-2 items-center">
        <span className="font-medium">Add Marker:</span>
        <input
          type="text"
          placeholder={`Marker for ${config.title}`}
          value={markerName}
          onChange={(e) => setMarkerName(e.target.value)}
          className="border px-2 py-1 rounded"
        />
        <button 
          onClick={handleAddMarker} 
          className="px-3 py-1 bg-gray-200 rounded hover:bg-gray-300"
          disabled={!currentTimeSeries?.current}
        >
          Add
        </button>
        <p className="text-sm text-gray-600 ml-4">{statusMessage ?? ""}</p>
      </div>
    </div>
  );
}