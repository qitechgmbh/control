import React, { useState, useRef, useEffect } from "react";
import type uPlot from "uplot";
import {
  AutoSyncedBigGraph,
  useGraphSync,
  type GraphConfig,
} from "@/components/graph";
import { TimeSeries, TimeSeriesValue } from "@/lib/timeseries";
import { Unit } from "@/control/units";
import { useMarkerManager } from "./useMarkerManager";
import { useMarkerContext } from "./MarkerContext";

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

const MARKER_HIT_WIDTH = 28;

const LABEL_TOP_OFFSET = 30;

function applyLabelHighlight(label: HTMLDivElement, highlighted: boolean) {
  if (highlighted) {
    label.style.zIndex = "20";
    label.style.boxShadow = "0 4px 12px rgba(0,0,0,0.2)";
    label.style.borderColor = "rgb(59 130 246)";
  } else {
    label.style.zIndex = "";
    label.style.boxShadow = "0 2px 6px rgba(0,0,0,0.15)";
    label.style.borderColor = "rgba(0,0,0,0.12)";
  }
}

function unhighlightAllLabels(container: HTMLElement) {
  container.querySelectorAll<HTMLDivElement>(".marker-label").forEach((el) => {
    applyLabelHighlight(el, false);
  });
}

// Apply or restore overlay styles (avoids React Compiler "mutates immutable" on inline DOM writes)
function setOverlayOverflow(
  container: HTMLElement,
  parent: HTMLElement | null,
  visible: boolean,
  previous: { overflow: string; position: string; parentOverflow: string },
) {
  if (visible) {
    container.style.overflow = "visible";
    container.style.position = "relative";
    if (parent) parent.style.overflow = "visible";
  } else {
    container.style.overflow = previous.overflow;
    container.style.position = previous.position;
    if (parent) parent.style.overflow = previous.parentOverflow;
  }
}

// Duration (ms) to hold pointer on marker before it is deleted
const LONG_PRESS_DELETE_MS = 5000;

// Build marker DOM: one wrapper (hit area) for hover/tap, contains line, point, label.
// Optional onLongPress: called after LONG_PRESS_DELETE_MS hold (e.g. to delete marker).
function createMarkerElement(
  u: uPlot,
  overlayRect: DOMRect,
  timestamp: number,
  name: string,
  value: number,
  color?: string,
  onLongPress?: () => void,
): HTMLDivElement {
  const plotRect = u.rect;
  const plotLeftInOverlay = plotRect.left - overlayRect.left;
  const plotTopInOverlay = plotRect.top - overlayRect.top;
  const plotHeight = plotRect.height;
  const plotWidth = plotRect.width;

  let xInPlot = u.valToPos(timestamp, "x", false);
  xInPlot = Math.max(0, Math.min(plotWidth, xInPlot));
  const xPos = plotLeftInOverlay + xInPlot;
  let yInPlot = u.valToPos(value, "y", false);
  yInPlot = Math.max(0, Math.min(plotHeight, yInPlot));

  const lineColor = color || "rgba(0, 0, 0, 0.5)";
  const pointColor = color || "rgba(0, 0, 0, 0.8)";
  const half = MARKER_HIT_WIDTH / 2;

  const wrapperTop = plotTopInOverlay - LABEL_TOP_OFFSET;
  const plotStartInWrapper = LABEL_TOP_OFFSET;

  const wrapper = document.createElement("div");
  wrapper.style.position = "absolute";
  wrapper.style.left = `${xPos - half}px`;
  wrapper.style.top = `${wrapperTop}px`;
  wrapper.style.width = `${MARKER_HIT_WIDTH}px`;
  wrapper.style.height = `${plotHeight + 50 + plotStartInWrapper}px`;
  wrapper.style.cursor = "pointer";
  wrapper.style.touchAction = "manipulation";
  wrapper.style.zIndex = "10";
  wrapper.className = "marker-wrapper";

  const line = document.createElement("div");
  line.style.position = "absolute";
  line.style.left = `${half - 1}px`;
  line.style.top = `${plotStartInWrapper}px`;
  line.style.height = `${plotHeight}px`;
  line.style.width = "2px";
  line.style.background = lineColor;
  line.style.pointerEvents = "none";
  line.title = name;
  line.className = "vertical-marker";

  const point = document.createElement("div");
  point.style.position = "absolute";
  point.style.left = `${half}px`;
  point.style.top = `${plotStartInWrapper + yInPlot}px`;
  point.style.width = "8px";
  point.style.height = "8px";
  point.style.borderRadius = "50%";
  point.style.background = pointColor;
  point.style.transform = "translate(-50%, -50%)";
  point.style.border = "2px solid white";
  point.style.pointerEvents = "none";
  point.title = name;
  point.className = "marker-point";

  const label = document.createElement("div");
  label.textContent = name;
  label.title = name;
  label.style.position = "absolute";
  label.style.left = `${half}px`;
  label.style.top = "6px";
  label.style.transform = "translateX(-50%)";
  label.style.color = "rgb(30 41 59)";
  label.style.padding = "4px 8px";
  label.style.fontSize = "12px";
  label.style.fontWeight = "600";
  label.style.whiteSpace = "nowrap";
  label.style.maxWidth = "160px";
  label.style.overflow = "hidden";
  label.style.textOverflow = "ellipsis";
  label.style.background = "rgba(255, 255, 255, 1)";
  label.style.borderRadius = "6px";
  label.style.boxShadow = "0 2px 8px rgba(0,0,0,0.2)";
  label.style.border = "1px solid rgba(0,0,0,0.2)";
  label.style.pointerEvents = "none";
  label.style.transition = "box-shadow 0.15s ease, border-color 0.15s ease";
  label.style.zIndex = "1";
  label.className = "marker-label";

  const highlight = () => applyLabelHighlight(label, true);
  const unhighlight = () => applyLabelHighlight(label, false);

  wrapper.addEventListener("mouseenter", highlight);
  wrapper.addEventListener("mouseleave", unhighlight);
  wrapper.addEventListener("click", () => highlight());

  // Long-press: hold 5s on marker to delete it (timer on pointerdown, clear on up/leave/cancel)
  let longPressTimer: ReturnType<typeof setTimeout> | null = null;
  const clearLongPressTimer = () => {
    if (longPressTimer !== null) {
      clearTimeout(longPressTimer);
      longPressTimer = null;
    }
  };
  wrapper.addEventListener("pointerdown", () => {
    if (!onLongPress) return;
    clearLongPressTimer();
    longPressTimer = setTimeout(() => {
      longPressTimer = null;
      onLongPress(); // removes this marker
    }, LONG_PRESS_DELETE_MS);
  });
  wrapper.addEventListener("pointerup", clearLongPressTimer);
  wrapper.addEventListener("pointerleave", clearLongPressTimer);
  wrapper.addEventListener("pointercancel", clearLongPressTimer);

  wrapper.appendChild(line);
  wrapper.appendChild(point);
  wrapper.appendChild(label);
  return wrapper;
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
  uplotRefOut,
}: GraphWithMarkerControlsProps & {
  graphWrapperRef: React.RefObject<HTMLDivElement | null>;
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
    if (curr?.timestamp != null) {
      setCurrentTimestamp(curr.timestamp);
      setCurrentValue(curr.value);
    } else {
      setCurrentValue(null);
    }
  }, [
    currentTimeSeries?.current?.timestamp,
    currentTimeSeries?.current?.value,
    setCurrentTimestamp,
    setCurrentValue,
  ]);

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

  // Marker Drawing Effect: use uPlot instance for exact valToPos so point sits on the curve
  useEffect(() => {
    const u = uplotRefOut.current;
    if (!u?.root?.parentElement) return;

    u.syncRect();

    const overlayContainer = u.root.parentElement;
    const overlayParent = overlayContainer.parentElement;

    const previous = {
      overflow: overlayContainer.style.overflow,
      position: overlayContainer.style.position,
      parentOverflow: overlayParent?.style.overflow ?? "",
    };
    setOverlayOverflow(overlayContainer, overlayParent, true, previous);

    const overlayRect = overlayContainer.getBoundingClientRect();

    // Only show markers within the visible time window (saved markers stay in store for export)
    const xMin = u.scales.x?.min ?? -Infinity;
    const xMax = u.scales.x?.max ?? Infinity;
    const visibleMarkers = markers.filter(
      (m) => m.timestamp >= xMin && m.timestamp <= xMax,
    );

    const wrappers: HTMLDivElement[] = visibleMarkers.map(
      ({ timestamp, name, value, color }) => {
        let markerValue = value;
        if (markerValue === undefined && currentTimeSeries) {
          const validValues = currentTimeSeries.long.values.filter(
            (v): v is TimeSeriesValue => v !== null,
          );
          if (validValues.length > 0) {
            // Use linear interpolation between surrounding points for stable positioning
            // (avoids "closest" flipping between adjacent points which causes jumping)
            const sorted = [...validValues].sort(
              (a, b) => a.timestamp - b.timestamp,
            );
            const after = sorted.find((p) => p.timestamp >= timestamp);
            const before = [...sorted]
              .reverse()
              .find((p) => p.timestamp <= timestamp);
            if (after && before) {
              if (after.timestamp === before.timestamp) {
                markerValue = after.value;
              } else {
                const t =
                  (timestamp - before.timestamp) /
                  (after.timestamp - before.timestamp);
                markerValue = before.value + t * (after.value - before.value);
              }
            } else if (after) {
              markerValue = after.value;
            } else if (before) {
              markerValue = before.value;
            }
          }
        }
        if (markerValue === undefined) {
          const yScale = u.scales.y;
          markerValue =
            yScale?.min != null && yScale?.max != null
              ? (yScale.min + yScale.max) / 2
              : 0;
        }

        // Pass remove callback so long-press (5s) on this marker deletes it
        return createMarkerElement(
          u,
          overlayRect,
          timestamp,
          name,
          markerValue,
          color,
          () => markerManager.removeMarker(timestamp),
        );
      },
    );

    overlayContainer
      .querySelectorAll(".marker-wrapper")
      .forEach((el) => el.remove());

    wrappers.forEach((wrapper) => overlayContainer.appendChild(wrapper));

    const onOverlayClick = (e: MouseEvent) => {
      const target = e.target as HTMLElement;
      if (!target.closest(".marker-wrapper")) {
        unhighlightAllLabels(overlayContainer);
      }
    };
    overlayContainer.addEventListener("click", onOverlayClick, true);

    return () => {
      overlayContainer.removeEventListener("click", onOverlayClick, true);
      setOverlayOverflow(overlayContainer, overlayParent, false, previous);
    };
  }, [markers, currentTimeSeries, timeTick, uplotRefOut, graphWrapperRef]);

  // Use original config without adding marker lines (markers are overlay elements)
  const finalConfig = config;

  return (
    <div className="flex flex-col gap-2">
      <div ref={graphWrapperRef} className="relative">
        <AutoSyncedBigGraph
          syncHook={syncHook}
          newData={newData}
          config={finalConfig}
          unit={unit}
          renderValue={renderValue}
          graphId={graphId}
          uplotRefOut={uplotRefOut}
        />
      </div>
    </div>
  );
}

export function GraphWithMarkerControls(props: GraphWithMarkerControlsProps) {
  const graphWrapperRef = useRef<HTMLDivElement | null>(null);
  const uplotRefOut = useRef<uPlot | null>(null);

  return (
    <GraphWithMarkerControlsContent
      {...props}
      graphWrapperRef={graphWrapperRef}
      uplotRefOut={uplotRefOut}
    />
  );
}
