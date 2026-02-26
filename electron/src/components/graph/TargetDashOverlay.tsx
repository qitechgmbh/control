import React, { useEffect, useMemo, useState, useId } from "react";
import uPlot from "uplot";
import { BigGraphProps, GraphConfig } from "./types";
import { getAllTimeSeries } from "./createChart";

type OverlayLine = {
  key: string;
  color: string;
  width: number;
  dash: number[];
  d: string;
};

type ClipRect = {
  x: number;
  y: number;
  width: number;
  height: number;
};

function findFirstGe(arr: number[], value: number): number {
  let lo = 0;
  let hi = arr.length - 1;
  let ans = arr.length;

  while (lo <= hi) {
    const mid = (lo + hi) >> 1;
    if (arr[mid] >= value) {
      ans = mid;
      hi = mid - 1;
    } else {
      lo = mid + 1;
    }
  }

  return ans;
}

function findLastLe(arr: number[], value: number): number {
  let lo = 0;
  let hi = arr.length - 1;
  let ans = -1;

  while (lo <= hi) {
    const mid = (lo + hi) >> 1;
    if (arr[mid] <= value) {
      ans = mid;
      lo = mid + 1;
    } else {
      hi = mid - 1;
    }
  }

  return ans;
}

function buildSteppedPath(
  u: uPlot,
  xData: number[],
  yData: Array<number | null>,
): string {
  const xMin = u.scales.x?.min;
  const xMax = u.scales.x?.max;

  if (xData.length < 2 || xMin === undefined || xMax === undefined) {
    return "";
  }

  const firstVisibleIdx = Math.max(0, findFirstGe(xData, xMin) - 1);
  const lastVisibleIdx = Math.min(
    xData.length - 1,
    findLastLe(xData, xMax) + 1,
  );

  if (lastVisibleIdx < firstVisibleIdx) {
    return "";
  }

  const parts: string[] = [];
  let started = false;
  let prevX = 0;
  let prevY = 0;

  for (let i = firstVisibleIdx; i <= lastVisibleIdx; i++) {
    const value = yData[i];
    if (value === null || value === undefined) continue;

    const x = u.valToPos(xData[i], "x", true);
    const y = u.valToPos(value, "y", true);

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
    const isHistoricalDashedTarget = !!line.targetSeries && !!dash?.length;

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
  uplotRef,
  newData,
  config,
  selectedTimeWindow,
  isLiveMode,
}: {
  uplotRef: React.RefObject<uPlot | null>;
  newData: BigGraphProps["newData"];
  config: GraphConfig;
  selectedTimeWindow: number | "all";
  isLiveMode: boolean;
}) {
  const targetMeta = useMemo(
    () => getHistoricalDashTargets(newData, config),
    [newData, config],
  );
  const clipPathId = useId().replace(/:/g, "");

  const [lines, setLines] = useState<OverlayLine[]>([]);
  const [clipRect, setClipRect] = useState<ClipRect | null>(null);

  useEffect(() => {
    if (targetMeta.length === 0) {
      setLines([]);
      setClipRect(null);
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
      const u = uplotRef.current;
      if (!u || targetMeta.length === 0) {
        return;
      }

      const xData = u.data[0] as number[] | undefined;
      if (!xData || xData.length < 2) {
        return;
      }

      const nextLines = targetMeta
        .map((meta, index) => {
          const yData = u.data[meta.dataIndex] as
            | Array<number | null>
            | undefined;
          if (!yData || yData.length < 2) return null;

          const d = buildSteppedPath(u, xData, yData);
          if (!d) return null;

          return {
            key: `${meta.dataIndex}-${index}`,
            color: meta.color,
            width: meta.width,
            dash: meta.dash,
            d,
          } as OverlayLine;
        })
        .filter((line): line is OverlayLine => !!line);

      if (nextLines.length > 0) {
        setLines((prev) =>
          areOverlayLinesEqual(prev, nextLines) ? prev : nextLines,
        );
        const nextClipRect: ClipRect = {
          x: u.bbox.left,
          y: u.bbox.top,
          width: u.bbox.width,
          height: u.bbox.height,
        };
        setClipRect((prev) =>
          isSameClipRect(prev, nextClipRect) ? prev : nextClipRect,
        );
      }
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

    const addHook = (u: any, hookName: string, fn: () => void) => {
      const hooks = u.hooks?.[hookName];
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
      const u = uplotRef.current as any;
      if (!u) return false;

      addHook(u, "draw", scheduleRecalc);
      addHook(u, "setScale", scheduleRecalc);
      addHook(u, "setData", scheduleRecalc);

      const rootElement = u.root as HTMLElement | undefined;
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
  }, [uplotRef, targetMeta, isLiveMode, selectedTimeWindow]);

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
              strokeDashoffset={0}
            />
          );
        })}
      </g>
    </svg>
  );
}
