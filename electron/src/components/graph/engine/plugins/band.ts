import type { Chart, Plugin } from "chart.js";
import type { BandPoint } from "../points";

/** Opacity of the min/max band under a series line. */
const BAND_ALPHA = 0.18;

/** A line dataset that may carry the min/max band of its buckets. */
export type BandDataset = { band?: BandPoint[]; borderColor?: unknown };

/**
 * Draws each series' min/max band (`dataset.band`) as a faint area in the series colour,
 * under the lines. Only the mean line is a Chart.js dataset, so the band never changes
 * the y range: spikes stay visible without making the axis jump.
 */
export function bandPlugin(): Plugin<"line"> {
  return {
    id: "qitechBand",

    beforeDatasetsDraw(chart) {
      const x = chart.scales.x;
      const y = chart.scales.y;
      if (!x || !y) return;
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
      ctx.globalAlpha = BAND_ALPHA;

      chart.data.datasets.forEach((dataset, i) => {
        const { band, borderColor } = dataset as BandDataset;
        if (!band?.length || !chart.isDatasetVisible(i)) return;
        if (typeof borderColor === "string") ctx.fillStyle = borderColor;
        drawBand(chart, band, x.min, x.max);
      });

      ctx.restore();
    },
  };
}

/** Fills the band between `lo` and `hi`; a point with `lo: null` is a gap. */
function drawBand(chart: Chart, band: BandPoint[], min: number, max: number) {
  const { ctx } = chart;
  const x = chart.scales.x;
  const y = chart.scales.y;

  // One point beyond each edge so the area reaches the border.
  let start = lowerBound(band, min);
  if (start > 0) start--;
  let end = lowerBound(band, max);
  if (end < band.length) end++;

  let segStart = -1;
  const fill = (from: number, to: number) => {
    if (to - from < 2) return;
    ctx.beginPath();
    for (let k = from; k < to; k++) {
      ctx.lineTo(
        x.getPixelForValue(band[k].x),
        y.getPixelForValue(band[k].hi!),
      );
    }
    for (let k = to - 1; k >= from; k--) {
      ctx.lineTo(
        x.getPixelForValue(band[k].x),
        y.getPixelForValue(band[k].lo!),
      );
    }
    ctx.closePath();
    ctx.fill();
  };

  for (let k = start; k < end; k++) {
    if (band[k].lo === null) {
      if (segStart >= 0) fill(segStart, k);
      segStart = -1;
    } else if (segStart < 0) {
      segStart = k;
    }
  }
  if (segStart >= 0) fill(segStart, end);
}

function lowerBound(band: BandPoint[], x: number): number {
  let lo = 0;
  let hi = band.length;
  while (lo < hi) {
    const mid = (lo + hi) >>> 1;
    if (band[mid].x < x) lo = mid + 1;
    else hi = mid;
  }
  return lo;
}
