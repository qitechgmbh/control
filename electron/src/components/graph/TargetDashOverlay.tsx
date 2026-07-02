import React, { useEffect, useId, useState } from "react";
import type { IChartApi, ISeriesApi, Time } from "lightweight-charts";
import { SeriesRefs } from "./types";

type OverlayLine = {
  key: string;
  color: string;
  width: number;
  dash: number[];
  dashOffset: number;
  d: string;
};

type ClipRect = {
  x: number;
  y: number;
  width: number;
  height: number;
};

function buildDashedLine(
  chart: IChartApi,
  series: ISeriesApi<"Line">,
  points: readonly { time: Time; value: number }[],
): string {
  if (points.length < 2) {
    return "";
  }

  const parts: string[] = [];
  let started = false;
  let prevX = 0;
  let prevY = 0;

  // timeToCoordinate/priceToCoordinate already return CSS-pixel coordinates
  // relative to the pane, which aligns with the overlay SVG's own coordinate
  // space here — no device-pixel-ratio correction needed (unlike uPlot).
  for (const point of points) {
    const x = chart.timeScale().timeToCoordinate(point.time);
    const y = series.priceToCoordinate(point.value);
    if (x === null || y === null) continue;

    if (!started) {
      parts.push(`M ${x} ${y}`);
      started = true;
      prevX = x;
      prevY = y;
      continue;
    }

    if (x !== prevX) {
      parts.push(`L ${x} ${prevY}`);
    }
    if (y !== prevY) {
      parts.push(`L ${x} ${y}`);
    }

    prevX = x;
    prevY = y;
  }

  // Extend the last step to the right edge of the plot area (the current
  // visible-range's right edge maps directly to the pane's right edge) so the
  // target line reaches the same boundary as the data series.
  if (started) {
    const visibleRange = chart.timeScale().getVisibleRange();
    if (visibleRange) {
      const rightEdge = chart.timeScale().timeToCoordinate(visibleRange.to);
      if (rightEdge !== null && rightEdge > prevX) {
        parts.push(`L ${rightEdge} ${prevY}`);
      }
    }
  }

  return parts.join(" ");
}

function areDashArraysEqual(a: number[], b: number[]): boolean {
  if (a.length !== b.length) return false;
  for (let i = 0; i < a.length; i++) {
    if (a[i] !== b[i]) return false;
  }
  return true;
}

function areOverlayLinesEqual(a: OverlayLine[], b: OverlayLine[]): boolean {
  if (a.length !== b.length) return false;
  for (let i = 0; i < a.length; i++) {
    if (
      a[i].key !== b[i].key ||
      a[i].color !== b[i].color ||
      a[i].width !== b[i].width ||
      a[i].d !== b[i].d ||
      !areDashArraysEqual(a[i].dash, b[i].dash)
    ) {
      return false;
    }
  }
  return true;
}

function isSameClipRect(a: ClipRect | null, b: ClipRect): boolean {
  return (
    !!a &&
    a.x === b.x &&
    a.y === b.y &&
    a.width === b.width &&
    a.height === b.height
  );
}

export function TargetDashOverlay({
  chartRef,
  seriesRefs,
  containerRef,
}: {
  chartRef: React.RefObject<IChartApi | null>;
  seriesRefs: React.RefObject<SeriesRefs | null>;
  containerRef: React.RefObject<HTMLDivElement | null>;
}) {
  const clipPathId = useId().replace(/:/g, "");

  const [lines, setLines] = useState<OverlayLine[]>([]);
  const [clipRect, setClipRect] = useState<ClipRect | null>(null);

  useEffect(() => {
    let rafId: number | null = null;
    let recalcScheduled = false;
    let resizeObserver: ResizeObserver | null = null;
    let unsubscribeRange: (() => void) | null = null;

    const recalc = () => {
      recalcScheduled = false;
      const chart = chartRef.current;
      const refs = seriesRefs.current;
      const container = containerRef.current;
      if (!chart || !refs || !container) {
        setLines((prev) => (prev.length === 0 ? prev : []));
        return;
      }

      const overlayEntries = refs.lineSeries.filter(
        (entry) => entry.isOverlayDriven,
      );
      if (overlayEntries.length === 0) {
        setLines((prev) => (prev.length === 0 ? prev : []));
        return;
      }

      const nextLines = overlayEntries
        .map((entry, index) => {
          const points = entry.api.data() as ReadonlyArray<{
            time: Time;
            value: number;
          }>;
          if (points.length < 2) return null;

          const d = buildDashedLine(chart, entry.api, points);
          if (!d) return null;

          return {
            key: `${index}`,
            color: entry.line.color,
            width: entry.line.width ?? 1,
            dash: entry.line.dash ?? [5, 5],
            dashOffset: 0,
            d,
          } satisfies OverlayLine;
        })
        .filter((line): line is OverlayLine => !!line);

      setLines((prev) =>
        areOverlayLinesEqual(prev, nextLines) ? prev : nextLines,
      );

      const rect = container.getBoundingClientRect();
      let priceScaleWidth = 0;
      let timeScaleHeight = 0;
      try {
        priceScaleWidth = chart.priceScale("right").width();
        timeScaleHeight = chart.timeScale().height();
      } catch {
        // Fall back to no inset if the API shape differs across versions.
      }
      const nextClipRect: ClipRect = {
        x: 0,
        y: 0,
        width: Math.max(rect.width - priceScaleWidth, 0),
        height: Math.max(rect.height - timeScaleHeight, 0),
      };
      setClipRect((prev) =>
        isSameClipRect(prev, nextClipRect) ? prev : nextClipRect,
      );
    };

    const scheduleRecalc = () => {
      if (recalcScheduled) return;
      recalcScheduled = true;
      rafId = window.requestAnimationFrame(recalc);
    };

    scheduleRecalc();

    const chart = chartRef.current;
    if (chart) {
      chart.timeScale().subscribeVisibleTimeRangeChange(scheduleRecalc);
      unsubscribeRange = () =>
        chart.timeScale().unsubscribeVisibleTimeRangeChange(scheduleRecalc);
    }

    if (containerRef.current && typeof ResizeObserver !== "undefined") {
      resizeObserver = new ResizeObserver(() => scheduleRecalc());
      resizeObserver.observe(containerRef.current);
    }

    const onWindowResize = () => scheduleRecalc();
    window.addEventListener("resize", onWindowResize);

    return () => {
      if (rafId !== null) {
        window.cancelAnimationFrame(rafId);
      }
      unsubscribeRange?.();
      resizeObserver?.disconnect();
      window.removeEventListener("resize", onWindowResize);
    };
  }, [chartRef, seriesRefs, containerRef]);

  if (lines.length === 0) {
    return null;
  }

  return (
    <svg
      className="pointer-events-none absolute inset-0 h-full w-full"
      aria-hidden
    >
      {clipRect && (
        <defs>
          <clipPath id={clipPathId}>
            <rect
              x={clipRect.x}
              y={clipRect.y}
              width={clipRect.width}
              height={clipRect.height}
            />
          </clipPath>
        </defs>
      )}

      <g clipPath={clipRect ? `url(#${clipPathId})` : undefined}>
        {lines.map((line) => {
          return (
            <path
              key={line.key}
              d={line.d}
              fill="none"
              stroke={line.color}
              strokeWidth={line.width}
              strokeLinecap="butt"
              strokeDasharray={line.dash.join(" ")}
              strokeDashoffset={line.dashOffset}
            />
          );
        })}
      </g>
    </svg>
  );
}
