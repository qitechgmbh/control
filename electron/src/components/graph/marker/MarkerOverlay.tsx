import React, { useEffect, useId, useMemo, useState } from "react";
import type { IChartApi } from "lightweight-charts";
import { TimeSeries } from "@/lib/timeseries";
import { msToTime } from "../dataHelpers";
import type { Marker } from "@/stores/markerStore";

const MARKER_BOTTOM_EXTENSION_PX = 94;
const MARKER_TOP_OFFSET_PX = 96;
const MARKER_LABEL_OFFSET_PX = 12;

type MarkerPosition = {
  key: string;
  name: string;
  color: string;
  x: number;
  y: number;
  lineTop: number;
  plotTop: number;
  plotBottom: number;
  labelY: number;
};

type ClipRect = {
  x: number;
  y: number;
  width: number;
  height: number;
};

type PlotBounds = {
  left: number;
  top: number;
  width: number;
  height: number;
  bottom: number;
};

function getPlotBounds(
  chart: IChartApi,
  container: HTMLDivElement,
): PlotBounds {
  const rect = container.getBoundingClientRect();
  let priceScaleWidth = 0;
  let timeScaleHeight = 0;
  try {
    priceScaleWidth = chart.priceScale("right").width();
    timeScaleHeight = chart.timeScale().height();
  } catch {
    // Fall back to no inset if the API shape differs across versions.
  }

  return {
    left: 0,
    top: 0,
    width: Math.max(rect.width - priceScaleWidth, 0),
    height: Math.max(rect.height - timeScaleHeight, 0),
    bottom: Math.max(rect.height - timeScaleHeight, 0),
  };
}

function getMarkerBottom(bounds: PlotBounds): number {
  return bounds.bottom + MARKER_BOTTOM_EXTENSION_PX;
}

function buildClipRect(bounds: PlotBounds): ClipRect {
  const clipBottom = getMarkerBottom(bounds);

  return {
    x: bounds.left,
    y: bounds.top,
    width: bounds.width,
    height: clipBottom - bounds.top,
  };
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

function areMarkerPositionsEqual(
  a: MarkerPosition[],
  b: MarkerPosition[],
): boolean {
  if (a.length !== b.length) return false;
  for (let i = 0; i < a.length; i++) {
    if (
      a[i].key !== b[i].key ||
      a[i].name !== b[i].name ||
      a[i].color !== b[i].color ||
      a[i].x !== b[i].x ||
      a[i].y !== b[i].y ||
      a[i].lineTop !== b[i].lineTop ||
      a[i].plotTop !== b[i].plotTop ||
      a[i].plotBottom !== b[i].plotBottom ||
      a[i].labelY !== b[i].labelY
    ) {
      return false;
    }
  }
  return true;
}

function buildMarkerPositions(
  chart: IChartApi,
  container: HTMLDivElement,
  markers: Marker[],
): MarkerPosition[] {
  const bounds = getPlotBounds(chart, container);
  const markerBottom = getMarkerBottom(bounds);
  const lineTop = bounds.top + MARKER_TOP_OFFSET_PX;
  const labelY = lineTop - MARKER_LABEL_OFFSET_PX;
  const visibleRange = chart.timeScale().getVisibleRange();
  const xMin = visibleRange ? (visibleRange.from as number) * 1000 : -Infinity;
  const xMax = visibleRange ? (visibleRange.to as number) * 1000 : Infinity;

  return markers
    .filter((marker) => marker.timestamp >= xMin && marker.timestamp <= xMax)
    .map((marker) => {
      const markerCenterY = lineTop + (markerBottom - lineTop) / 2;
      const rawX = chart
        .timeScale()
        .timeToCoordinate(msToTime(marker.timestamp));
      const x = Math.max(
        bounds.left,
        Math.min(bounds.left + bounds.width, rawX ?? bounds.left),
      );

      return {
        key: `${marker.timestamp}-${marker.name}`,
        name: marker.name,
        color: marker.color || "rgba(0, 0, 0, 0.7)",
        x,
        y: markerCenterY,
        lineTop,
        plotTop: bounds.top,
        plotBottom: markerBottom,
        labelY,
      };
    });
}

export function MarkerOverlay({
  chartRef,
  containerRef,
  markers,
}: {
  chartRef: React.RefObject<IChartApi | null>;
  containerRef: React.RefObject<HTMLDivElement | null>;
  markers: Marker[];
  currentTimeSeries: TimeSeries | null;
}) {
  const clipPathId = useId().replace(/:/g, "");
  const stableMarkers = useMemo(() => markers, [markers]);
  const [positions, setPositions] = useState<MarkerPosition[]>([]);
  const [clipRect, setClipRect] = useState<ClipRect | null>(null);

  useEffect(() => {
    if (stableMarkers.length === 0) {
      setPositions([]);
      setClipRect(null);
    }
  }, [stableMarkers]);

  useEffect(() => {
    let rafId: number | null = null;
    let recalcScheduled = false;
    let resizeObserver: ResizeObserver | null = null;
    let unsubscribeRange: (() => void) | null = null;

    const recalc = () => {
      recalcScheduled = false;
      const chart = chartRef.current;
      const container = containerRef.current;
      if (!chart || !container || stableMarkers.length === 0) {
        return;
      }

      const nextPositions = buildMarkerPositions(
        chart,
        container,
        stableMarkers,
      );

      setPositions((prev) =>
        areMarkerPositionsEqual(prev, nextPositions) ? prev : nextPositions,
      );

      const nextClipRect = buildClipRect(getPlotBounds(chart, container));

      setClipRect((prev) =>
        isSameClipRect(prev, nextClipRect) ? prev : nextClipRect,
      );
    };

    const scheduleRecalc = () => {
      if (recalcScheduled) return;
      recalcScheduled = true;
      rafId = window.requestAnimationFrame(recalc);
    };

    if (stableMarkers.length === 0) {
      return;
    }

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
  }, [chartRef, containerRef, stableMarkers]);

  if (positions.length === 0 || !clipRect) {
    return null;
  }

  return (
    <svg
      className="pointer-events-none absolute inset-0 h-full w-full"
      aria-hidden
    >
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

      <g clipPath={`url(#${clipPathId})`}>
        {positions.map((marker) => (
          <g key={marker.key}>
            <line
              x1={marker.x}
              x2={marker.x}
              y1={marker.lineTop}
              y2={marker.plotBottom}
              stroke={marker.color}
              strokeWidth="2"
              opacity="0.85"
            />
            <circle
              cx={marker.x}
              cy={marker.y}
              r="4"
              fill={marker.color}
              stroke="white"
              strokeWidth="2"
            />
            <text
              x={marker.x}
              y={marker.labelY}
              textAnchor="middle"
              fontSize="12"
              fontWeight="600"
              fill="rgb(30 41 59)"
              dominantBaseline="middle"
              stroke="white"
              strokeWidth="4"
              strokeLinejoin="round"
              paintOrder="stroke"
            >
              {marker.name}
            </text>
          </g>
        ))}
      </g>
    </svg>
  );
}
