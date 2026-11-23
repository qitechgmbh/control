import type { Plugin } from "chart.js";

/** A vertical marker line at a point in time, labelled with its name. */
export type ChartMarker = {
  timestamp: number;
  name: string;
  color?: string;
};

export type MarkersState = {
  markers: ChartMarker[];
  /** Value of the chart's first visible series at a timestamp, for the dot. */
  valueAt: (timestamp: number) => number | null;
};

const DEFAULT_COLOR = "#111827";
const LABEL_HEIGHT = 16;
const LABEL_GAP = 4;
/** Label rows before labels start to overlap again. */
const LABEL_ROWS = 3;

/**
 * Draws user markers: a dashed vertical line, a dot where it crosses the first visible
 * series, and the name in a label at the top. Labels that would overlap move down a row.
 */
export function markersPlugin(getState: () => MarkersState): Plugin<"line"> {
  return {
    id: "qitechMarkers",

    afterDatasetsDraw(chart) {
      const { markers, valueAt } = getState();
      const { ctx, chartArea } = chart;
      const x = chart.scales.x;
      const y = chart.scales.y;
      if (markers.length === 0 || !x || !y) return;

      const visible = markers
        .map((marker) => ({ marker, px: x.getPixelForValue(marker.timestamp) }))
        .filter(({ px }) => px >= chartArea.left && px <= chartArea.right)
        .sort((a, b) => a.px - b.px);
      if (visible.length === 0) return;

      ctx.save();
      ctx.beginPath();
      ctx.rect(
        chartArea.left,
        chartArea.top,
        chartArea.width,
        chartArea.height,
      );
      ctx.clip();

      // Lines and dots
      for (const { marker, px } of visible) {
        const color = marker.color || DEFAULT_COLOR;
        ctx.strokeStyle = color;
        ctx.lineWidth = 1.5;
        ctx.setLineDash([4, 3]);
        ctx.beginPath();
        ctx.moveTo(px, chartArea.top);
        ctx.lineTo(px, chartArea.bottom);
        ctx.stroke();
        ctx.setLineDash([]);

        const value = valueAt(marker.timestamp);
        if (value !== null) {
          const py = y.getPixelForValue(value);
          ctx.fillStyle = color;
          ctx.strokeStyle = "#ffffff";
          ctx.lineWidth = 1.5;
          ctx.beginPath();
          ctx.arc(px, py, 4, 0, Math.PI * 2);
          ctx.fill();
          ctx.stroke();
        }
      }

      // Labels, left to right, each in the first row where it fits
      ctx.font = "600 12px system-ui, sans-serif";
      ctx.textBaseline = "middle";
      const rowEnds = new Array<number>(LABEL_ROWS).fill(-Infinity);
      for (const { marker, px } of visible) {
        const color = marker.color || DEFAULT_COLOR;
        const w = ctx.measureText(marker.name).width + 10;
        const lx = Math.max(
          chartArea.left,
          Math.min(px + 4, chartArea.right - w),
        );
        let row = rowEnds.findIndex((end) => end + LABEL_GAP <= lx);
        if (row < 0) row = rowEnds.indexOf(Math.min(...rowEnds));
        rowEnds[row] = lx + w;
        const ly = chartArea.top + 2 + row * (LABEL_HEIGHT + 2);

        ctx.fillStyle = "rgba(255,255,255,0.92)";
        ctx.fillRect(lx, ly, w, LABEL_HEIGHT);
        ctx.fillStyle = color;
        ctx.fillRect(lx, ly, 2, LABEL_HEIGHT);
        ctx.fillText(marker.name, lx + 6, ly + LABEL_HEIGHT / 2);
      }

      ctx.restore();
    },
  };
}
