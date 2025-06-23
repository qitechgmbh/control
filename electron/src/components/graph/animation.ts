import { useRef } from "react";
import uPlot from "uplot";
import { POINT_ANIMATION_DURATION } from "./constants";

export interface AnimationState {
  isAnimating: boolean;
  startTime: number;
  fromValue: number;
  toValue: number;
  fromTimestamp: number;
  toTimestamp: number;
  targetIndex: number;
}

export interface AnimationRefs {
  animationFrame: React.RefObject<number | null>;
  animationState: React.RefObject<AnimationState>;
  lastRenderedData: React.RefObject<{
    timestamps: number[];
    values: number[];
  }>;
  realPointsCount: React.RefObject<number>;
}

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
): uPlot.AlignedData {
  const uData: uPlot.AlignedData = [timestamps, values];

  if (realPointsCount !== undefined) {
    realPointsCountRef.current = realPointsCount;
  }

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
  updateYAxisScale: (
    timestamps: number[],
    values: number[],
    xMin?: number,
    xMax?: number,
  ) => void,
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

    const animatedUData = buildUPlotData(
      animatedTimestamps,
      animatedValues,
      animatedTimestamps.length,
      refs.realPointsCount,
      config,
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
        updateYAxisScale(animatedTimestamps, animatedValues, xMin, xMax);
      } else if (viewMode === "all") {
        const fullStart = startTimeRef.current ?? animatedTimestamps[0];
        uplotRef.current.setScale("x", {
          min: fullStart,
          max: latestTimestamp,
        });
        updateYAxisScale(
          animatedTimestamps,
          animatedValues,
          fullStart,
          latestTimestamp,
        );
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
        );
      }
    }
  };

  refs.animationFrame.current = requestAnimationFrame(animate);
}
