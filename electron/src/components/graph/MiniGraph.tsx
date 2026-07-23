import React, { useEffect, useRef, useCallback, useMemo } from "react";
import uPlot from "uplot";
import "uplot/dist/uPlot.min.css";
import { seriesToUPlotData, TimeSeries } from "@/lib/timeseries";

type MiniGraphProps = {
  newData: TimeSeries | null;
  width: number;
  renderValue?: (value: number) => string;
};

const HEIGHT = 64;
const MIN_UPDATE_INTERVAL_MS = 1000;
const MAX_RENDER_POINTS = 32;
const GRAPH_SMOOTHING_WINDOW = 3;
const SCALE_PADDING_RATIO = 0.25;
const SCALE_SHRINK_THRESHOLD = 4;

function quantizeValue(value: number, renderValue?: (value: number) => string) {
  if (!renderValue) {
    return Math.round(value * 10) / 10;
  }

  const rendered = renderValue(value);
  const numeric = Number(rendered);
  return Number.isFinite(numeric) ? numeric : value;
}

function getMinMax(values: number[]): { min: number; max: number } {
  if (values.length === 0) {
    return { min: 0, max: 0 };
  }

  return {
    min: Math.min(...values),
    max: Math.max(...values),
  };
}

function paddedScale(min: number, max: number): { min: number; max: number } {
  const range = max - min || 1;
  return {
    min: min - range * SCALE_PADDING_RATIO,
    max: max + range * SCALE_PADDING_RATIO,
  };
}

function shouldUpdateScale(
  current: { min: number; max: number },
  next: { min: number; max: number },
): boolean {
  if (next.min < current.min || next.max > current.max) {
    return true;
  }

  const nextRange = next.max - next.min || 1;
  const currentRange = current.max - current.min || 1;
  return currentRange > nextRange * SCALE_SHRINK_THRESHOLD;
}

function smoothValues(values: number[]): number[] {
  if (values.length <= 2 || GRAPH_SMOOTHING_WINDOW <= 1) {
    return values;
  }

  const halfWindow = Math.floor(GRAPH_SMOOTHING_WINDOW / 2);
  return values.map((_, index) => {
    const start = Math.max(0, index - halfWindow);
    const end = Math.min(values.length, index + halfWindow + 1);
    let sum = 0;

    for (let i = start; i < end; i++) {
      sum += values[i];
    }

    return sum / (end - start);
  });
}

function compactSeries(
  timestamps: number[],
  values: number[],
): [number[], number[]] {
  if (timestamps.length <= MAX_RENDER_POINTS) {
    return [timestamps, values];
  }

  const bucketSize = Math.ceil(timestamps.length / MAX_RENDER_POINTS);
  const compactTimestamps: number[] = [];
  const compactValues: number[] = [];

  for (let start = 0; start < timestamps.length; start += bucketSize) {
    const end = Math.min(start + bucketSize, timestamps.length);
    let sum = 0;

    for (let i = start; i < end; i++) {
      sum += values[i];
    }

    compactTimestamps.push(timestamps[end - 1]);
    compactValues.push(sum / (end - start));
  }

  return [compactTimestamps, compactValues];
}

export function MiniGraph({ newData, width, renderValue }: MiniGraphProps) {
  const divRef = useRef<HTMLDivElement | null>(null);
  const uplotRef = useRef<uPlot | null>(null);

  // Performance tracking refs
  const lastUpdateTimestamp = useRef<number>(0);
  const lastDataHash = useRef<string>("");
  const yScale = useRef<{ min: number; max: number }>({ min: 0, max: 0 });
  const rafId = useRef<number>(0);
  const isInitialized = useRef<boolean>(false);
  const pendingUpdate = useRef<boolean>(false);

  // Fast hash function for data change detection
  const hashData = useCallback(
    (timestamps: number[], values: number[]): string => {
      if (timestamps.length === 0) return "";
      // Hash only first, last, and length for performance
      return `${timestamps[0]}-${timestamps[timestamps.length - 1]}-${timestamps.length}-${values[values.length - 1]}`;
    },
    [],
  );

  // Memoize the tick formatter to avoid recreating on every render
  const tickFormatter = useMemo(() => {
    return renderValue
      ? (u: uPlot, ticks: number[]) => ticks.map((v) => renderValue(v))
      : (u: uPlot, ticks: number[]) => ticks.map((v) => v.toFixed(1));
  }, [renderValue]);

  // Ultra-efficient update function
  const updateChart = useCallback(() => {
    if (!uplotRef.current || !newData?.short || !newData?.current) {
      pendingUpdate.current = false;
      return;
    }

    const cur = newData.current;

    // Skip if no new data
    if (cur.timestamp <= lastUpdateTimestamp.current) {
      pendingUpdate.current = false;
      return;
    }

    if (cur.timestamp - lastUpdateTimestamp.current < MIN_UPDATE_INTERVAL_MS) {
      pendingUpdate.current = false;
      return;
    }

    const short = newData.short;
    const timeWindow = short.timeWindow;

    // Get data
    const [timestamps, rawValues] = seriesToUPlotData(short);
    const values = rawValues.map((value) => quantizeValue(value, renderValue));
    const [plotTimestamps, compactValues] = compactSeries(timestamps, values);
    const plotValues = smoothValues(compactValues);

    if (plotTimestamps.length === 0) {
      pendingUpdate.current = false;
      return;
    }

    // Check if data actually changed using hash
    const dataHash = hashData(plotTimestamps, plotValues);
    if (dataHash === lastDataHash.current) {
      pendingUpdate.current = false;
      return;
    }

    // Get min/max and check if scales need updating
    const { min: minY, max: maxY } = getMinMax(plotValues);
    const nextScale = paddedScale(minY, maxY);
    const scalesChanged = shouldUpdateScale(yScale.current, nextScale);

    // Update tracking vars
    lastUpdateTimestamp.current = cur.timestamp;
    lastDataHash.current = dataHash;
    const cutoff = cur.timestamp - timeWindow;

    // Batch all updates to minimize redraws
    uplotRef.current.batch(() => {
      // Always update data and x-scale (time moves forward)
      uplotRef.current!.setData([plotTimestamps, plotValues]);
      uplotRef.current!.setScale("x", { min: cutoff, max: cur.timestamp });

      // Only update y-scale if min/max changed
      if (scalesChanged) {
        yScale.current = nextScale;
        uplotRef.current!.setScale("y", nextScale);
      }
    });

    pendingUpdate.current = false;
  }, [newData, hashData, renderValue]);

  // RAF-throttled update scheduler
  const scheduleUpdate = useCallback(() => {
    if (pendingUpdate.current || !isInitialized.current) return;

    pendingUpdate.current = true;

    if (rafId.current) {
      cancelAnimationFrame(rafId.current);
    }

    rafId.current = requestAnimationFrame(updateChart);
  }, [updateChart]);

  // Initialize chart only once
  useEffect(() => {
    if (!divRef.current || !newData?.short?.timeWindow || isInitialized.current)
      return;

    const short = newData.short;
    const timeWindow = short.timeWindow;

    // Get initial data
    const [allTimestamps, rawAllValues] = seriesToUPlotData(short);
    const allValues = rawAllValues.map((value) =>
      quantizeValue(value, renderValue),
    );
    const [plotTimestamps, compactValues] = compactSeries(
      allTimestamps,
      allValues,
    );
    const plotValues = smoothValues(compactValues);
    const { min: minY, max: maxY } = getMinMax(plotValues);
    const initialScale = paddedScale(minY, maxY);

    // Initialize tracking
    const dataHash = hashData(plotTimestamps, plotValues);
    lastDataHash.current = dataHash;
    yScale.current = initialScale;

    const now = Date.now();
    const latestTimestamp =
      plotTimestamps.length > 0
        ? plotTimestamps[plotTimestamps.length - 1]
        : now;
    const cutoff = latestTimestamp - timeWindow;

    const opts: uPlot.Options = {
      width,
      height: HEIGHT,
      padding: [4, 0, 4, 0],
      cursor: { show: false },
      legend: { show: false },
      scales: {
        x: {
          time: true,
          min: cutoff,
          max: latestTimestamp,
        },
        y: {
          auto: false,
          min: initialScale.min,
          max: initialScale.max,
        },
      },
      axes: [
        { show: false },
        {
          side: 1,
          grid: { stroke: "#ccc", width: 0.5 },
          ticks: { stroke: "#ccc", width: 0.5 },
          values: tickFormatter, // Use the memoized formatter
        },
      ],
      series: [
        {},
        {
          stroke: "black",
          width: 2,
          spanGaps: true,
          points: { show: false }, // Hide the dots/points
        },
      ],
    };

    uplotRef.current = new uPlot(
      opts,
      [plotTimestamps, plotValues],
      divRef.current,
    );
    isInitialized.current = true;

    return () => {
      if (rafId.current) {
        cancelAnimationFrame(rafId.current);
      }
      uplotRef.current?.destroy();
      uplotRef.current = null;
      isInitialized.current = false;
      pendingUpdate.current = false;
    };
  }, [width, newData?.short?.timeWindow, hashData, tickFormatter, renderValue]);

  // Trigger updates only when timestamp changes
  useEffect(() => {
    scheduleUpdate();
  }, [newData?.current?.timestamp, scheduleUpdate]);

  // Efficient width handling
  useEffect(() => {
    if (!uplotRef.current || !isInitialized.current) return;
    uplotRef.current.setSize({ width, height: HEIGHT });
  }, [width]);

  return (
    <div
      ref={divRef}
      style={{
        width: "100%",
        height: HEIGHT,
        overflow: "hidden",
      }}
    />
  );
}
