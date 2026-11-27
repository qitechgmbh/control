import type { Plugin } from "chart.js";

// --- cursor sync between the charts of one group ---

type CursorListener = (x: number | null) => void;
const cursorGroups = new Map<string, Set<CursorListener>>();

/** Listens for the cursor of any chart in `group`. Returns the unsubscribe function. */
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

function publishCursor(group: string, x: number | null) {
  cursorGroups.get(group)?.forEach((listener) => listener(x));
}

// --- plugin ---

export type CrosshairOptions = {
  /** Charts of one group share the crosshair. */
  group: string;
  /** Crosshair position (ms) to draw, from this chart or a synced one. */
  getX: () => number | null;
};

/**
 * Publishes the pointer position over this chart to its group, and draws the group's
 * crosshair. Touch shows the crosshair only while the finger is down.
 */
export function crosshairPlugin({
  group,
  getX,
}: CrosshairOptions): Plugin<"line"> {
  return {
    id: "qitechCrosshair",

    afterEvent(chart, args) {
      const e = args.event;
      const nativeType = (e.native as Event | null)?.type;
      if (
        e.type === "mouseout" ||
        nativeType === "touchend" ||
        nativeType === "touchcancel"
      ) {
        publishCursor(group, null);
        return;
      }
      // Chart.js maps touchstart/touchmove to mousedown/mousemove.
      if (e.type !== "mousemove" && e.type !== "mousedown") return;
      const area = chart.chartArea;
      if (
        e.x === null ||
        e.y === null ||
        e.x < area.left ||
        e.x > area.right ||
        e.y < area.top ||
        e.y > area.bottom
      ) {
        publishCursor(group, null);
        return;
      }
      publishCursor(group, chart.scales.x.getValueForPixel(e.x) ?? null);
    },

    afterDatasetsDraw(chart) {
      const value = getX();
      const x = chart.scales.x;
      if (value === null || !x) return;
      const { ctx, chartArea } = chart;
      const px = x.getPixelForValue(value);
      if (px < chartArea.left || px > chartArea.right) return;

      ctx.save();
      ctx.strokeStyle = "rgba(100,116,139,0.8)";
      ctx.lineWidth = 1;
      ctx.beginPath();
      ctx.moveTo(px, chartArea.top);
      ctx.lineTo(px, chartArea.bottom);
      ctx.stroke();
      ctx.restore();
    },
  };
}
