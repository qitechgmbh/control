import React, { useEffect, useId, useMemo, useState } from "react";
import uPlot from "uplot";
import { TimeSeries } from "@/lib/timeseries";
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

function getPlotBounds(plot: uPlot): PlotBounds {
  const dpr = window.devicePixelRatio || 1;
  const left = plot.bbox.left / dpr;
  const top = plot.bbox.top / dpr;
  const width = plot.bbox.width / dpr;
  const height = plot.bbox.height / dpr;

  return {
    left,
    top,
    width,
    height,
    bottom: top + height,
  };
}

function getMarkerBottom(bounds: PlotBounds): number {
  return bounds.bottom + MARKER_BOTTOM_EXTENSION_PX;
}

function buildClipRect(plot: uPlot): ClipRect {
  const bounds = getPlotBounds(plot);
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
  plot: uPlot,
  markers: Marker[],
  _currentTimeSeries: TimeSeries | null,
): MarkerPosition[] {
  const bounds = getPlotBounds(plot);
  const markerBottom = getMarkerBottom(bounds);
  const lineTop = bounds.top + MARKER_TOP_OFFSET_PX;
  const labelY = lineTop - MARKER_LABEL_OFFSET_PX;
  const xMin = plot.scales.x?.min ?? -Infinity;
  const xMax = plot.scales.x?.max ?? Infinity;

  return markers
    .filter((marker) => marker.timestamp >= xMin && marker.timestamp <= xMax)
    .map((marker) => {
      const markerCenterY = lineTop + (markerBottom - lineTop) / 2;

      return {
        key: `${marker.timestamp}-${marker.name}`,
        name: marker.name,
        color: marker.color || "rgba(0, 0, 0, 0.7)",
        x: Math.max(
          bounds.left,
          Math.min(
            bounds.left + bounds.width,
            plot.valToPos(marker.timestamp, "x", false),
          ),
        ),
        y: markerCenterY,
        lineTop,
        plotTop: bounds.top,
        plotBottom: markerBottom,
        labelY,
      };
    });
}

export function MarkerOverlay({
  uplotRef,
  markers,
  currentTimeSeries,
}: {
  uplotRef: React.RefObject<uPlot | null>;
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
    let bootstrapRafId: number | null = null;
    let recalcScheduled = false;
    let hooksAttached = false;
    let resizeObserver: ResizeObserver | null = null;
    const removeHookFns: Array<() => void> = [];

    const recalc = () => {
      recalcScheduled = false;
      const plot = uplotRef.current;
      if (!plot || stableMarkers.length === 0) {
        return;
      }

      plot.syncRect();
      const nextPositions = buildMarkerPositions(
        plot,
        stableMarkers,
        currentTimeSeries,
      );

      setPositions((prev) =>
        areMarkerPositionsEqual(prev, nextPositions) ? prev : nextPositions,
      );

      const nextClipRect = buildClipRect(plot);

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

    const addHook = (plot: any, hookName: string, fn: () => void) => {
      const hooks = plot.hooks?.[hookName];
      if (!Array.isArray(hooks)) return;
      hooks.push(fn);
      removeHookFns.push(() => {
        const idx = hooks.indexOf(fn);
        if (idx >= 0) hooks.splice(idx, 1);
      });
    };

    const attachHooks = () => {
      if (hooksAttached) return true;
      const plot = uplotRef.current as any;
      if (!plot) return false;

      addHook(plot, "setScale", scheduleRecalc);
      addHook(plot, "setData", scheduleRecalc);

      const rootElement = plot.root as HTMLElement | undefined;
      if (rootElement && typeof ResizeObserver !== "undefined") {
        resizeObserver = new ResizeObserver(() => scheduleRecalc());
        resizeObserver.observe(rootElement);
      }

      hooksAttached = true;
      scheduleRecalc();
      return true;
    };

    if (!attachHooks()) {
      const tryAttach = () => {
        if (attachHooks()) return;
        bootstrapRafId = window.requestAnimationFrame(tryAttach);
      };
      bootstrapRafId = window.requestAnimationFrame(tryAttach);
    }

    const onWindowResize = () => scheduleRecalc();
    window.addEventListener("resize", onWindowResize);

    return () => {
      if (rafId !== null) {
        window.cancelAnimationFrame(rafId);
      }
      if (bootstrapRafId !== null) {
        window.cancelAnimationFrame(bootstrapRafId);
      }
      removeHookFns.forEach((fn) => fn());
      window.removeEventListener("resize", onWindowResize);
      resizeObserver?.disconnect();
    };
  }, [uplotRef, stableMarkers, currentTimeSeries]);

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
