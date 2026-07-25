import React, { useEffect, useRef, useCallback, useMemo } from "react";
import uPlot from "uplot";
import "uplot/dist/uPlot.min.css";
import {
  Series,
  getSeriesMinMax,
  seriesToUPlotData,
  TimeSeries,
  TimeSeriesValue,
} from "@/lib/timeseries";

type MiniGraphProps = {
  newData: TimeSeries | null;
  width: number;
  renderValue?: (value: number) => string;
  sampleInterval?: number;
  valuePrecision?: number;
};

const HEIGHT = 64;

function roundValue(value: number, precision: number | undefined): number {
  if (precision === undefined) {
    return value;
  }
  const factor = 10 ** precision;
  return Math.round(value * factor) / factor;
}

function downsampleSeries(
  series: Series,
  sampleInterval: number | undefined,
  valuePrecision: number | undefined,
): Series {
  if (sampleInterval === undefined && valuePrecision === undefined) {
    return series;
  }

  const [timestamps, values] = seriesToUPlotData(series);
  const filteredValues: TimeSeriesValue[] = [];
  let lastTimestamp = Number.NEGATIVE_INFINITY;

  timestamps.forEach((timestamp, index) => {
    if (
      sampleInterval !== undefined &&
      timestamp - lastTimestamp < sampleInterval &&
      index !== timestamps.length - 1
    ) {
      return;
    }

    filteredValues.push({
      timestamp,
      value: roundValue(values[index], valuePrecision),
    });
    lastTimestamp = timestamp;
  });

  return {
    ...series,
    values: filteredValues,
    index: filteredValues.length,
    size: filteredValues.length,
    lastTimestamp: filteredValues.at(-1)?.timestamp ?? series.lastTimestamp,
    validCount: filteredValues.length,
  };
}

export function MiniGraph({
  newData,
  width,
  renderValue,
  sampleInterval,
  valuePrecision,
}: MiniGraphProps) {
  const divRef = useRef<HTMLDivElement | null>(null);
  const uplotRef = useRef<uPlot | null>(null);

  // Performance tracking refs
  const lastUpdateTimestamp = useRef<number>(0);
  const lastDataHash = useRef<string>("");
  const lastMinMax = useRef<{ min: number; max: number }>({ min: 0, max: 0 });
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

  const shortSeries = useMemo(() => {
    if (!newData?.short) {
      return null;
    }
    return downsampleSeries(newData.short, sampleInterval, valuePrecision);
  }, [newData?.short, sampleInterval, valuePrecision]);

  // Ultra-efficient update function
  const updateChart = useCallback(() => {
    if (!uplotRef.current || !shortSeries || !newData?.current) {
      pendingUpdate.current = false;
      return;
    }

    const cur = newData.current;

    // Skip if no new data
    if (cur.timestamp <= lastUpdateTimestamp.current) {
      pendingUpdate.current = false;
      return;
    }

    const short = shortSeries;
    const timeWindow = short.timeWindow;

    // Get data
    const [timestamps, values] = seriesToUPlotData(short);

    if (timestamps.length === 0) {
      pendingUpdate.current = false;
      return;
    }

    // Check if data actually changed using hash
    const dataHash = hashData(timestamps, values);
    if (dataHash === lastDataHash.current) {
      pendingUpdate.current = false;
      return;
    }

    // Get min/max and check if scales need updating
    const { min: minY, max: maxY } = getSeriesMinMax(short);
    const scalesChanged =
      minY !== lastMinMax.current.min || maxY !== lastMinMax.current.max;

    // Update tracking vars
    lastUpdateTimestamp.current = cur.timestamp;
    lastDataHash.current = dataHash;
    lastMinMax.current = { min: minY, max: maxY };

    const range = maxY - minY || 1;
    const cutoff = cur.timestamp - timeWindow;

    // Batch all updates to minimize redraws
    uplotRef.current.batch(() => {
      // Always update data and x-scale (time moves forward)
      uplotRef.current!.setData([timestamps, values]);
      uplotRef.current!.setScale("x", { min: cutoff, max: cur.timestamp });

      // Only update y-scale if min/max changed
      if (scalesChanged) {
        uplotRef.current!.setScale("y", {
          min: minY - range * 0.1,
          max: maxY + range * 0.1,
        });
      }
    });

    pendingUpdate.current = false;
  }, [newData?.current, shortSeries, hashData]);

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
    if (!divRef.current || !shortSeries?.timeWindow || isInitialized.current)
      return;

    const short = shortSeries;
    const timeWindow = short.timeWindow;

    // Get initial data
    const [allTimestamps, allValues] = seriesToUPlotData(short);
    const { min: minY, max: maxY } = getSeriesMinMax(short);
    const range = maxY - minY || 1;

    // Initialize tracking
    const dataHash = hashData(allTimestamps, allValues);
    lastDataHash.current = dataHash;
    lastMinMax.current = { min: minY, max: maxY };

    const now = Date.now();
    const latestTimestamp =
      allTimestamps.length > 0 ? allTimestamps[allTimestamps.length - 1] : now;
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
          min: minY - range * 0.1,
          max: maxY + range * 0.1,
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
      [allTimestamps, allValues],
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
  }, [width, shortSeries?.timeWindow, hashData, tickFormatter]);

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
