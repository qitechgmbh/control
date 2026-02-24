/**
 * MiniGraph — compact inline sparkline for control page cards.
 *
 * 64 px tall, no cursor, no legend, no x-axis.
 * Y-axis grid shows on the right side.
 * Driven by a rAF loop reading directly from a SeriesBuffer — no reactive
 * subscriptions, no intermediate array copies on every event.
 *
 * Props:
 *  - getBuffer: () => SeriesBuffer  called each frame to read the ring buffer
 *  - timeWindow: number  seconds of history to display (default: 10)
 *  - stroke: CSS colour string (default: var(--accent) / #6c8cff)
 */

import uPlot from "uplot";
import { onMount, onCleanup } from "solid-js";
import type { SeriesBuffer } from "../namespaces/mock1";

const HEIGHT = 64;

type Props = {
  getBuffer: () => SeriesBuffer;
  timeWindow?: number;
  stroke?: string;
};

export default function MiniGraph(props: Props) {
  let container!: HTMLDivElement;
  let uplot: uPlot | null = null;
  let rafId = 0;
  let pendingWidth = 0;

  function buildOpts(width: number): uPlot.Options {
    return {
      width,
      height: HEIGHT,
      padding: [4, 0, 4, 0],
      cursor: { show: false },
      legend: { show: false },
      scales: {
        x: { time: true },
        y: { auto: false },
      },
      axes: [
        { show: false },
        {
          side: 1, // right side
          size: 40,
          stroke: "#717898",
          grid: { stroke: "#2e3148", width: 0.5 },
          ticks: { stroke: "#2e3148", width: 0.5 },
          values: (_u, ticks) => ticks.map((v) => v.toFixed(2)),
        },
      ],
      series: [
        {},
        {
          stroke: props.stroke ?? "#6c8cff",
          width: 1.5,
          spanGaps: true,
          points: { show: false },
        },
      ],
    };
  }

  function frame() {
    if (!uplot) { rafId = requestAnimationFrame(frame); return; }

    if (pendingWidth > 0) {
      uplot.setSize({ width: pendingWidth, height: HEIGHT });
      pendingWidth = 0;
    }

    const buf = props.getBuffer();
    const { times, values, length, head } = buf;
    if (length === 0) { rafId = requestAnimationFrame(frame); return; }

    const bufSize = times.length;
    const isFull = length === bufSize;
    const startIdx = isFull ? head : 0;
    const tMax = times[(head - 1 + bufSize) % bufSize];
    const tMin = tMax - (props.timeWindow ?? 10);

    // Count entries in window
    let count = 0;
    for (let i = 0; i < length; i++) {
      if (times[(startIdx + i) % bufSize] >= tMin) count++;
    }

    const outT = new Float64Array(count);
    const outV = new Float64Array(count);
    let out = 0;
    let minY = Infinity, maxY = -Infinity;
    for (let i = 0; i < length; i++) {
      const idx = (startIdx + i) % bufSize;
      const t = times[idx];
      if (t >= tMin) {
        outT[out] = t;
        outV[out] = values[idx];
        if (values[idx] < minY) minY = values[idx];
        if (values[idx] > maxY) maxY = values[idx];
        out++;
      }
    }

    if (count === 0) { rafId = requestAnimationFrame(frame); return; }

    const range = maxY - minY || 1;
    const pad = range * 0.15;

    uplot.batch(() => {
      uplot!.setData([outT, outV], false);
      uplot!.setScale("x", { min: tMin, max: tMax });
      uplot!.setScale("y", { min: minY - pad, max: maxY + pad });
    });

    rafId = requestAnimationFrame(frame);
  }

  onMount(() => {
    const ro = new ResizeObserver(([entry]) => {
      const w = Math.floor(entry.contentRect.width);
      if (w === 0) return;
      if (!uplot) {
        uplot = new uPlot(buildOpts(w), [new Float64Array(0), new Float64Array(0)], container);
        rafId = requestAnimationFrame(frame);
      } else {
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

  return <div ref={container} class="minigraph-wrap" />;
}
