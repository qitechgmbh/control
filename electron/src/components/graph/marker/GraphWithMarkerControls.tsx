import React, { useRef, useEffect } from "react";
import type uPlot from "uplot";
import { AutoSyncedBigGraph } from "../SyncedComponents";
import { useGraphSync } from "../useGraphSync";
import { TimeSeries } from "@/lib/timeseries";
import { Unit } from "@/control/units";
import { type GraphConfig, GraphLine } from "../types";
import { useMarkerManager } from "./useMarkerManager";
import { useMarkerContext } from "./MarkerContext";
import { MarkerOverlay } from "./MarkerOverlay";

type TimeSeriesData = {
  newData: TimeSeries | null;
  title?: string;
  color?: string;
  lines?: GraphLine[];
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
  uplotRefOut,
}: GraphWithMarkerControlsProps & {
  uplotRefOut: React.MutableRefObject<uPlot | null>;
}) {
  const { setMachineId, setCurrentTimestamp, setCurrentValue } =
    useMarkerContext();

  // Auto-detect machineId from graphId if not provided (extract base name)
  // e.g., "pressure-graph" -> "pressure", "extruder-graphs" -> "extruder-graphs"
  const machineId = providedMachineId || graphId.split("-")[0] || "default";

  // Update context with machineId, current timestamp and value for marker creation
  useEffect(() => {
    setMachineId(machineId);
  }, [machineId, setMachineId]);

  useEffect(() => {
    const curr = currentTimeSeries?.current;
    const isLiveMode = syncHook.syncGraph.isLiveMode;

    if (!isLiveMode) {
      const updateHistoricalTimestamp = () => {
        const xMax = uplotRefOut.current?.scales.x?.max;
        if (xMax != null) {
          setCurrentTimestamp(xMax);
        } else if (curr?.timestamp != null) {
          setCurrentTimestamp(curr.timestamp);
        }
        setCurrentValue(curr?.value ?? null);
      };

      const frame = window.requestAnimationFrame(updateHistoricalTimestamp);
      return () => window.cancelAnimationFrame(frame);
    }

    if (curr?.timestamp != null) {
      setCurrentTimestamp(curr.timestamp);
      setCurrentValue(curr.value);
    } else {
      setCurrentValue(null);
    }
  }, [
    syncHook.syncGraph.isLiveMode,
    syncHook.syncGraph.historicalFreezeTimestamp,
    syncHook.syncGraph.xRange?.min,
    syncHook.syncGraph.xRange?.max,
    currentTimeSeries?.current?.timestamp,
    currentTimeSeries?.current?.value,
    setCurrentTimestamp,
    setCurrentValue,
    uplotRefOut,
  ]);

  // Use provided markers or load from marker manager
  const markerManager = useMarkerManager(machineId);
  const markers = providedMarkers || markerManager.markers;

  // Use original config without adding marker lines (markers are overlay elements)
  const finalConfig = config;

  return (
    <div className="flex flex-col gap-2">
      <div className="relative">
        <AutoSyncedBigGraph
          syncHook={syncHook}
          newData={newData}
          config={finalConfig}
          unit={unit}
          renderValue={renderValue}
          graphId={graphId}
          uplotRefOut={uplotRefOut}
        />
        <MarkerOverlay
          uplotRef={uplotRefOut}
          markers={markers}
          currentTimeSeries={currentTimeSeries}
        />
      </div>
    </div>
  );
}

export function GraphWithMarkerControls(props: GraphWithMarkerControlsProps) {
  const uplotRefOut = useRef<uPlot | null>(null);

  return (
    <GraphWithMarkerControlsContent {...props} uplotRefOut={uplotRefOut} />
  );
}
