/**
 * UPlotGraph — SolidJS wrapper around uPlot with a rolling time window.
 *
 * Design principles:
 *  - The uPlot instance is driven by a rAF loop, NOT by reactive effects.
 *    This caps redraws at the display refresh rate (~60 fps) regardless of
 *    how fast incoming data arrives (e.g. 10 Hz socket events).
 *  - The caller provides a `getSeriesData` callback that returns the current
 *    ring-buffer slices.  The callback is called once per animation frame so
 *    it must be cheap (just a typed-array slice of the visible window).
 *  - `timeWindow` (seconds) controls how much history is visible.  The x-axis
 *    scale is set to [latestTimestamp - timeWindow, latestTimestamp] every
 *    frame, giving the rolling-window effect.
 *  - Resize is handled by ResizeObserver → setSize on next rAF frame.
 */

import uPlot from "uplot";
import "uplot/dist/uPlot.min.css";
import { onMount, onCleanup } from "solid-js";

export type HorizLine = {
  value: number;
  color: string;
  dash?: boolean;
  label?: string;
};

type Props = {
  /** Called each animation frame.  Return aligned data arrays (timestamps in seconds). */
  getSeriesData: () => uPlot.AlignedData;
  series: uPlot.Series[];
  unit?: string;
  height?: number;
  /** Visible time window in seconds (default: 30 s). */
  timeWindow?: number;
  lines?: HorizLine[];
};

export default function UPlotGraph(props: Props) {
  let container!: HTMLDivElement;
  let uplot: uPlot | null = null;
  let rafId = 0;
  let pendingWidth = 0;

  function buildOpts(width: number): uPlot.Options {
    const height = props.height ?? 240;
    const lines = props.lines ?? [];
    return {
      width,
      height,
      cursor: { sync: { key: "graphs" } },
      scales: {
        x: { time: true },
        y: { auto: true },
      },
      axes: [
        {
          stroke: "#717898",
          grid: { stroke: "#2e3148", width: 1 },
          ticks: { stroke: "#2e3148" },
        },
        {
          label: props.unit,
          stroke: "#717898",
          grid: { stroke: "#2e3148", width: 1 },
          ticks: { stroke: "#2e3148" },
        },
      ],
      series: [
        {},
        ...props.series,
      ],
      hooks: lines.length > 0 ? {
        drawSeries: [
          (u) => {
            const ctx = u.ctx;
            ctx.save();
            for (const line of lines) {
              const yPos = Math.round(u.valToPos(line.value, "y", true));
              ctx.beginPath();
              ctx.strokeStyle = line.color;
              ctx.lineWidth = 1.5;
              if (line.dash) ctx.setLineDash([6, 3]);
              ctx.moveTo(u.bbox.left, yPos);
              ctx.lineTo(u.bbox.left + u.bbox.width, yPos);
              ctx.stroke();
              ctx.setLineDash([]);
              if (line.label) {
                ctx.fillStyle = line.color;
                ctx.font = "11px Inter, system-ui, sans-serif";
                ctx.fillText(line.label, u.bbox.left + 4, yPos - 4);
              }
            }
            ctx.restore();
          },
        ],
      } : {},
    };
  }

  function frame() {
    if (!uplot) { rafId = requestAnimationFrame(frame); return; }

    // Apply any pending resize first
    if (pendingWidth > 0) {
      uplot.setSize({ width: pendingWidth, height: props.height ?? 240 });
      pendingWidth = 0;
    }

    const data = props.getSeriesData();
    const times = data[0] as Float64Array | number[];
    const n = times.length;

    if (n > 0) {
      const tMax = times[n - 1];
      const tMin = tMax - (props.timeWindow ?? 30);
      uplot.setData(data, false);           // false = don't auto-scale x
      uplot.setScale("x", { min: tMin, max: tMax });
    }

    rafId = requestAnimationFrame(frame);
  }

  onMount(() => {
    const ro = new ResizeObserver(([entry]) => {
      const w = Math.floor(entry.contentRect.width);
      if (w === 0) return;
      if (!uplot) {
        uplot = new uPlot(buildOpts(w), props.getSeriesData(), container);
        rafId = requestAnimationFrame(frame);
      } else {
        // Defer resize to the rAF loop to avoid mid-paint artifacts
        pendingWidth = w;
      }
    });
    ro.observe(container);

    onCleanup(() => {
      cancelAnimationFrame(rafId);
      ro.disconnect();
      uplot?.destroy();
      uplot = null;
    });
  });

  return <div ref={container} class="uplot-wrap" />;
}
