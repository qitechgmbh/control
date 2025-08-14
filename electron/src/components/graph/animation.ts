import { useRef } from "react";
import uPlot from "uplot";
import { POINT_ANIMATION_DURATION } from "./constants";
import {
  AnimationRefs,
  AnimationState,
  BigGraphProps,
  SeriesData,
} from "./types";

export function useAnimationRefs(): AnimationRefs {
  const animationFrameRef = useRef<number | null>(null);
  const animationStateRef = useRef<AnimationState>({
    isAnimating: false,
    startTime: 0,
    fromValue: 0,
    toValue: 0,
    fromTimestamp: 0,
    toTimestamp: 0,
    targetIndex: 0,
  });
  const lastRenderedDataRef = useRef<{
    timestamps: number[];
    values: number[];
  }>({ timestamps: [], values: [] });
  const realPointsCountRef = useRef(0);

  return {
    animationFrame: animationFrameRef,
    animationState: animationStateRef,
    lastRenderedData: lastRenderedDataRef,
    realPointsCount: realPointsCountRef,
  };
}

export function lerp(start: number, end: number, t: number): number {
  return start + (end - start) * t;
}

export function stopAnimations(refs: AnimationRefs): void {
  if (refs.animationFrame.current) {
    cancelAnimationFrame(refs.animationFrame.current);
    refs.animationFrame.current = null;
  }
  refs.animationState.current.isAnimating = false;
}

export function buildUPlotData(
  timestamps: number[],
  values: number[],
  realPointsCount: number | undefined,
  realPointsCountRef: React.RefObject<number>,
  config: { lines?: Array<{ show?: boolean; value: number }> },
  allSeriesData?: number[][],
): uPlot.AlignedData {
  const uData: uPlot.AlignedData = [timestamps];

  // Add primary series (for animation)
  uData.push(values);

  // Add additional series if provided
  if (allSeriesData) {
    allSeriesData.forEach((seriesValues) => {
      uData.push(seriesValues);
    });
  }

  if (realPointsCount !== undefined) {
    realPointsCountRef.current = realPointsCount;
  }

  // Add config lines
  config.lines?.forEach((line) => {
    if (line.show !== false) {
      uData.push(timestamps.map(() => line.value));
    }
  });

  return uData;
}

export function animateNewPoint(
  currentData: { timestamps: number[]; values: number[] },
  targetData: { timestamps: number[]; values: number[] },
  refs: AnimationRefs,
  uplotRef: React.RefObject<uPlot | null>,
  isLiveMode: boolean,
  viewMode: string,
  selectedTimeWindow: number | "all",
  startTimeRef: React.RefObject<number | null>,
  config: { lines?: Array<{ show?: boolean; value: number }> },
  updateYAxisScale: (xMin?: number, xMax?: number) => void, // Updated signature
  getAllSeriesData?: () => number[][],
): void {
  if (targetData.timestamps.length <= currentData.timestamps.length) {
    return;
  }
  if (!isLiveMode || viewMode === "manual") {
    return;
  }

  const newIndex = currentData.timestamps.length;

  const prevValue =
    currentData.values[newIndex - 1] ?? targetData.values[newIndex];
  const prevTimestamp =
    currentData.timestamps[newIndex - 1] ?? targetData.timestamps[newIndex];
  const newValue = targetData.values[newIndex];
  const newTimestamp = targetData.timestamps[newIndex];

  if (refs.animationFrame.current) {
    cancelAnimationFrame(refs.animationFrame.current);
    refs.animationFrame.current = null;
  }

  refs.animationState.current = {
    isAnimating: true,
    startTime: performance.now(),
    fromValue: prevValue,
    toValue: newValue,
    fromTimestamp: prevTimestamp,
    toTimestamp: newTimestamp,
    targetIndex: newIndex,
  };

  const animate = (currentTime: number) => {
    if (
      !uplotRef.current ||
      !refs.animationState.current.isAnimating ||
      !isLiveMode
    )
      return;

    const elapsed = currentTime - refs.animationState.current.startTime;
    const progress = Math.min(elapsed / POINT_ANIMATION_DURATION, 1);

    const animatedTimestamps = [...currentData.timestamps];
    const animatedValues = [...currentData.values];

    if (progress < 1) {
      const interpolatedTimestamp = lerp(
        refs.animationState.current.fromTimestamp,
        refs.animationState.current.toTimestamp,
        progress,
      );
      const interpolatedValue = lerp(
        refs.animationState.current.fromValue,
        refs.animationState.current.toValue,
        progress,
      );

      animatedTimestamps.push(interpolatedTimestamp);
      animatedValues.push(interpolatedValue);
    } else {
      animatedTimestamps.push(refs.animationState.current.toTimestamp);
      animatedValues.push(refs.animationState.current.toValue);

      refs.lastRenderedData.current = {
        timestamps: [...animatedTimestamps],
        values: [...animatedValues],
      };
      refs.realPointsCount.current = animatedTimestamps.length;
      refs.animationState.current.isAnimating = false;
    }

    // Get all series data for complete uPlot update
    const allSeriesData = getAllSeriesData ? getAllSeriesData() : undefined;

    const animatedUData = buildUPlotData(
      animatedTimestamps,
      animatedValues,
      animatedTimestamps.length,
      refs.realPointsCount,
      config,
      allSeriesData,
    );

    uplotRef.current.setData(animatedUData);

    if (isLiveMode && animatedTimestamps.length > 0) {
      const latestTimestamp = animatedTimestamps[animatedTimestamps.length - 1];

      if (viewMode === "default") {
        let xMin: number | undefined, xMax: number | undefined;

        if (selectedTimeWindow === "all") {
          const fullStart = startTimeRef.current ?? animatedTimestamps[0];
          xMin = fullStart;
          xMax = latestTimestamp;
        } else {
          xMin = latestTimestamp - (selectedTimeWindow as number);
          xMax = latestTimestamp;
        }

        uplotRef.current.setScale("x", { min: xMin, max: xMax });
        updateYAxisScale(xMin, xMax); // Updated call
      } else if (viewMode === "all") {
        const fullStart = startTimeRef.current ?? animatedTimestamps[0];
        uplotRef.current.setScale("x", {
          min: fullStart,
          max: latestTimestamp,
        });
        updateYAxisScale(fullStart, latestTimestamp); // Updated call
      }
    }

    if (progress < 1) {
      refs.animationFrame.current = requestAnimationFrame(animate);
    } else {
      if (targetData.timestamps.length > animatedTimestamps.length) {
        animateNewPoint(
          { timestamps: animatedTimestamps, values: animatedValues },
          targetData,
          refs,
          uplotRef,
          isLiveMode,
          viewMode,
          selectedTimeWindow,
          startTimeRef,
          config,
          updateYAxisScale,
          getAllSeriesData,
        );
      }
    }
  };

  refs.animationFrame.current = requestAnimationFrame(animate);
}

// Helper function to normalize data to array format
export function normalizeDataSeries(
  data: BigGraphProps["newData"],
): SeriesData[] {
  if (Array.isArray(data)) {
    return data;
  }
  return [data];
}

// Helper function to get primary series for display value
export function getPrimarySeries(
  data: BigGraphProps["newData"],
): SeriesData | null {
  const normalized = normalizeDataSeries(data);
  return normalized.find((series) => series.newData !== null) || null;
}

export function getPrimarySeriesData(data: BigGraphProps["newData"]) {
  const normalized = normalizeDataSeries(data);
  const primarySeries = normalized.find((series) => series.newData !== null);
  return primarySeries?.newData || null;
}

// Helper function to format value for display
export function formatDisplayValue(
  value: number | undefined | null,
  renderValue?: (value: number) => string,
): string {
  if (value === undefined || value === null) return "N/A";
  return renderValue ? renderValue(value) : value.toFixed(2);
}
