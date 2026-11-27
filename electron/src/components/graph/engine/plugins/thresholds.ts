import type { Plugin } from "chart.js";

/** A horizontal reference line (threshold, tolerance). */
export type HLine = {
  value: number;
  color: string;
  dash?: number[];
  width?: number;
};

/**
 * Draws horizontal reference lines and extends the y range so they never fall
 * off-screen.
 */
export function thresholdsPlugin(getLines: () => HLine[]): Plugin<"line"> {
  return {
    id: "qitechThresholds",

    afterDataLimits(_chart, args) {
      const scale = args.scale;
      if (scale.axis !== "y") return;
      for (const line of getLines()) {
        if (!Number.isFinite(line.value)) continue;
        if (line.value < scale.min) scale.min = line.value;
        if (line.value > scale.max) scale.max = line.value;
      }
    },

    afterDatasetsDraw(chart) {
      const lines = getLines();
      const y = chart.scales.y;
      if (lines.length === 0 || !y) return;
      const { ctx, chartArea } = chart;

      ctx.save();
      ctx.beginPath();
      ctx.rect(
        chartArea.left,
        chartArea.top,
        chartArea.width,
        chartArea.height,
      );
      ctx.clip();
      for (const line of lines) {
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
      ctx.restore();
    },
  };
}
