import type { Chart, Plugin } from "chart.js";
import type { Marker } from "@/stores/markerStore";

/** Horizontal reference lines (thresholds) that also extend the y range. */
export type HLine = {
  value: number;
  color: string;
  dash?: number[];
  width?: number;
  label?: string;
};

/** Per-chart options for the custom plugins, stored on the chart instance. */
export type OverlayState = {
  hLines: HLine[];
  markers: Marker[];
  /** Value of each marker in this chart (first visible series), for the dot. */
  markerValue: (timestamp: number) => number | null;
  /** Crosshair x (ms) from this chart or a synced chart. */
  crosshairX: number | null;
};

const overlays = new WeakMap<Chart, OverlayState>();

export function setOverlayState(chart: Chart, state: OverlayState) {
  overlays.set(chart, state);
}

export function getOverlayState(chart: Chart): OverlayState | undefined {
  return overlays.get(chart);
}

/** Includes horizontal reference lines in the y range so they never fall off-screen. */
export const hLinesRangePlugin: Plugin<"line"> = {
  id: "qitechHLinesRange",
  afterDataLimits(chart, args) {
    const scale = args.scale;
    if (scale.axis !== "y") return;
    const state = overlays.get(chart);
    if (!state) return;
    for (const line of state.hLines) {
      if (!Number.isFinite(line.value)) continue;
      if (line.value < scale.min) scale.min = line.value;
      if (line.value > scale.max) scale.max = line.value;
    }
  },
};

/** Draws threshold lines, user markers and the (synced) crosshair on top of the data. */
export const overlayPlugin: Plugin<"line"> = {
  id: "qitechOverlay",
  afterDatasetsDraw(chart) {
    const state = overlays.get(chart);
    if (!state) return;
    const { ctx, chartArea } = chart;
    const x = chart.scales.x;
    const y = chart.scales.y;
    if (!x || !y) return;

    ctx.save();
    ctx.beginPath();
    ctx.rect(chartArea.left, chartArea.top, chartArea.width, chartArea.height);
    ctx.clip();

    // Threshold lines
    for (const line of state.hLines) {
      const py = y.getPixelForValue(line.value);
      if (py < chartArea.top || py > chartArea.bottom) continue;
      ctx.strokeStyle = line.color;
      ctx.lineWidth = line.width ?? 1;
      ctx.setLineDash(line.dash ?? [5, 5]);
      ctx.beginPath();
      ctx.moveTo(chartArea.left, py);
      ctx.lineTo(chartArea.right, py);
      ctx.stroke();
    }
    ctx.setLineDash([]);

    // User markers
    ctx.font = "600 12px system-ui, sans-serif";
    ctx.textBaseline = "top";
    for (const marker of state.markers) {
      const px = x.getPixelForValue(marker.timestamp);
      if (px < chartArea.left || px > chartArea.right) continue;
      const color = marker.color ?? "#111827";
      ctx.strokeStyle = color;
      ctx.lineWidth = 1.5;
      ctx.setLineDash([4, 3]);
      ctx.beginPath();
      ctx.moveTo(px, chartArea.top);
      ctx.lineTo(px, chartArea.bottom);
      ctx.stroke();
      ctx.setLineDash([]);

      const value = state.markerValue(marker.timestamp);
      if (value !== null) {
        const py = y.getPixelForValue(value);
        ctx.fillStyle = color;
        ctx.beginPath();
        ctx.arc(px, py, 4, 0, Math.PI * 2);
        ctx.fill();
      }

      const label = marker.name;
      const w = ctx.measureText(label).width + 8;
      const lx = Math.min(px + 4, chartArea.right - w);
      ctx.fillStyle = "rgba(255,255,255,0.9)";
      ctx.fillRect(lx, chartArea.top + 2, w, 16);
      ctx.fillStyle = color;
      ctx.fillText(label, lx + 4, chartArea.top + 4);
    }

    // Crosshair
    if (state.crosshairX !== null) {
      const px = x.getPixelForValue(state.crosshairX);
      if (px >= chartArea.left && px <= chartArea.right) {
        ctx.strokeStyle = "rgba(100,116,139,0.8)";
        ctx.lineWidth = 1;
        ctx.beginPath();
        ctx.moveTo(px, chartArea.top);
        ctx.lineTo(px, chartArea.bottom);
        ctx.stroke();
      }
    }

    ctx.restore();
  },
};

// --- cursor sync between charts of one group ---

type CursorListener = (x: number | null, source: string) => void;
const cursorGroups = new Map<string, Set<CursorListener>>();

export function subscribeCursor(group: string, listener: CursorListener) {
  let set = cursorGroups.get(group);
  if (!set) {
    set = new Set();
    cursorGroups.set(group, set);
  }
  set.add(listener);
  return () => {
    set!.delete(listener);
    if (set!.size === 0) cursorGroups.delete(group);
  };
}

export function publishCursor(group: string, x: number | null, source: string) {
  cursorGroups.get(group)?.forEach((listener) => listener(x, source));
}
