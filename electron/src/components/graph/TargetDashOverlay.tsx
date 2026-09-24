import React, { useEffect, useMemo, useRef, useState } from "react";
import uPlot from "uplot";
import { BigGraphProps, GraphConfig } from "./types";
import { getAllTimeSeries } from "./createChart";

type OverlayLine = {
  key: string;
  color: string;
  width: number;
  dash: number[];
  dashOffset: number;
  d: string;
};

// Position and size of uPlot's plot area, in CSS pixels relative to the overlay's
// containing block. The SVG is placed exactly on this rect, which makes the SVG's
// coordinate space identical to the one valToPos(..., false) reports in.
type PlotRect = {
  x: number;
  y: number;
  width: number;
  height: number;
};

function buildDashedLine(
  plot: uPlot,
  xData: number[],
  yData: Array<number | null>,
  plotWidth: number,
): string {
  if (xData.length < 2) {
    return "";
  }

  const parts: string[] = [];
  let started = false;
  let prevX = 0;
  let prevY = 0;

  // valToPos(..., false) returns CSS pixels relative to the PLOT AREA, not to the
  // chart root: uPlot passes an offset of 0 instead of plotLftCss/plotTopCss on this
  // branch. The SVG is positioned on the plot area for exactly that reason, so these
  // values can be used as-is. Using canvasPixels=true instead would return device
  // pixels and misplace everything by a factor of devicePixelRatio.
  for (let i = 0; i < xData.length; i++) {
    const value = yData[i];
    if (value === null || value === undefined) continue;

    const x = plot.valToPos(xData[i], "x", false);
    const y = plot.valToPos(value, "y", false);

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

  // Extend the last step to the right edge of the plot area so the target line
  // reaches the same boundary as the data series drawn by uPlot.
  if (started && plotWidth > prevX) {
    parts.push(`L ${plotWidth} ${prevY}`);
  }

  return parts.join(" ");
}

function getHistoricalDashTargets(
  data: BigGraphProps["newData"],
  config: GraphConfig,
): Array<{
  dataIndex: number;
  dash: number[];
  color: string;
  width: number;
}> {
  const allOriginalSeries = getAllTimeSeries(data);
  const firstConfigLineDataIndex = 1 + allOriginalSeries.length;
  let visibleLineIndex = 0;

  const targets: Array<{
    dataIndex: number;
    dash: number[];
    color: string;
    width: number;
  }> = [];

  config.lines?.forEach((line) => {
    if (line.show === false) return;

    const dash = line.dash ?? (line.type === "threshold" ? [5, 5] : undefined);
    const isHistoricalDashedTarget =
      line.type === "target" && !!line.targetSeries && !!dash?.length;

    if (isHistoricalDashedTarget) {
      targets.push({
        dataIndex: firstConfigLineDataIndex + visibleLineIndex,
        dash: dash!,
        color: line.color,
        width: line.width ?? 1,
      });
    }

    visibleLineIndex++;
  });

  return targets;
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

// getBoundingClientRect returns fractional values, so compare with sub-pixel
// tolerance to avoid re-rendering on layout noise.
function isSamePlotRect(a: PlotRect | null, b: PlotRect): boolean {
  const EPSILON = 0.01;
  return (
    !!a &&
    Math.abs(a.x - b.x) < EPSILON &&
    Math.abs(a.y - b.y) < EPSILON &&
    Math.abs(a.width - b.width) < EPSILON &&
    Math.abs(a.height - b.height) < EPSILON
  );
}

export function TargetDashOverlay({
  uplotRef,
  newData,
  config,
}: {
  uplotRef: React.RefObject<uPlot | null>;
  newData: BigGraphProps["newData"];
  config: GraphConfig;
}) {
  const targetMeta = useMemo(
    () => getHistoricalDashTargets(newData, config),
    [newData, config],
  );
  const svgRef = useRef<SVGSVGElement | null>(null);

  const [lines, setLines] = useState<OverlayLine[]>([]);
  const [plotRect, setPlotRect] = useState<PlotRect | null>(null);

  useEffect(() => {
    if (targetMeta.length === 0) {
      setLines([]);
      setPlotRect(null);
    }
  }, [targetMeta]);

  useEffect(() => {
    let rafId: number | null = null;
    let bootstrapRafId: number | null = null;
    let recalcScheduled = false;
    let hooksAttached = false;
    const removeHookFns: Array<() => void> = [];
    let resizeObserver: ResizeObserver | null = null;

    const recalc = () => {
      recalcScheduled = false;
      const plot = uplotRef.current;
      if (!plot || targetMeta.length === 0) {
        return;
      }

      const xData = plot.data[0] as number[] | undefined;
      if (!xData || xData.length < 2) {
        return;
      }

      // Measure uPlot's plot area (the .u-over box) in the coordinate space the
      // absolutely positioned SVG is laid out in: offsets are relative to the
      // containing block's padding box, hence the clientLeft/clientTop correction.
      const host = svgRef.current?.parentElement;
      if (!host) {
        return;
      }
      const overRect = plot.over.getBoundingClientRect();
      const hostRect = host.getBoundingClientRect();
      const nextPlotRect: PlotRect = {
        x: overRect.left - hostRect.left - host.clientLeft,
        y: overRect.top - hostRect.top - host.clientTop,
        width: overRect.width,
        height: overRect.height,
      };

      const nextLines = targetMeta
        .map((meta, index) => {
          const yData = plot.data[meta.dataIndex] as
            | Array<number | null>
            | undefined;
          if (!yData || yData.length < 2) return null;

          const d = buildDashedLine(plot, xData, yData, nextPlotRect.width);
          if (!d) return null;

          return {
            key: `${meta.dataIndex}-${index}`,
            color: meta.color,
            width: meta.width,
            dash: meta.dash,
            dashOffset: 0,
            d,
          } as OverlayLine;
        })
        .filter((line): line is OverlayLine => !!line);

      setLines((prev) =>
        areOverlayLinesEqual(prev, nextLines) ? prev : nextLines,
      );
      setPlotRect((prev) =>
        isSamePlotRect(prev, nextPlotRect) ? prev : nextPlotRect,
      );
    };

    const scheduleRecalc = () => {
      if (recalcScheduled) return;
      recalcScheduled = true;
      rafId = window.requestAnimationFrame(recalc);
    };

    scheduleRecalc();

    if (targetMeta.length === 0) {
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

    const onWindowResize = () => scheduleRecalc();
    window.addEventListener("resize", onWindowResize);

    const attachHooks = (): boolean => {
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
  }, [uplotRef, targetMeta]);

  // The SVG stays mounted even with nothing to draw: recalc measures the plot area
  // relative to this element's parent, so the ref has to be live on the first pass.
  // Sizing it to the plot area makes valToPos(..., false) directly usable as path
  // coordinates, and makes the SVG itself clip overflowing target lines.
  return (
    <svg
      ref={svgRef}
      className="pointer-events-none absolute"
      style={{
        left: plotRect?.x ?? 0,
        top: plotRect?.y ?? 0,
        width: plotRect?.width ?? 0,
        height: plotRect?.height ?? 0,
      }}
      aria-hidden
    >
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
    </svg>
  );
}
