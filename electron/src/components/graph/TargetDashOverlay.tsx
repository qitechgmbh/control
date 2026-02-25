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

const PATH_RECALC_INTERVAL_MS = 120;

function buildSteppedPath(
  u: uPlot,
  xData: number[],
  yData: Array<number | null>,
): string {
  if (xData.length < 2) {
    return "";
  }

  const parts: string[] = [];
  let started = false;
  let prevX = 0;
  let prevY = 0;

  for (let i = 0; i < xData.length; i++) {
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
  const clipPathId = useId().replace(/:/g, "");

  const [lines, setLines] = useState<OverlayLine[]>([]);
  const [clipRect, setClipRect] = useState<{
    x: number;
    y: number;
    width: number;
    height: number;
  } | null>(null);

  useEffect(() => {
    if (targetMeta.length === 0) {
      setLines([]);
      setClipRect(null);
    }
  }, [targetMeta]);

  useEffect(() => {
    const recalc = () => {
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
          const yData = u.data[meta.dataIndex] as Array<number | null> | undefined;
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

      // Keep previous valid lines if uPlot is in a transient state and no path
      // can be built for this tick; this prevents visible flicker.
      if (nextLines.length > 0) {
        setLines(nextLines);
        setClipRect({
          x: u.bbox.left,
          y: u.bbox.top,
          width: u.bbox.width,
          height: u.bbox.height,
        });
      }
    };

    recalc();

    if (targetMeta.length === 0) {
      return;
    }

    const recalcIntervalId = window.setInterval(recalc, PATH_RECALC_INTERVAL_MS);

    return () => {
      window.clearInterval(recalcIntervalId);
    };
  }, [uplotRef, targetMeta]);

  if (lines.length === 0) {
    return null;
  }

  return (
    <svg className="pointer-events-none absolute inset-0 h-full w-full" aria-hidden>
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
