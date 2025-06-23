import { useRef, useCallback } from "react";
import uPlot from "uplot";
import { seriesToUPlotData } from "@/lib/timeseries";
import { stopAnimations, AnimationRefs } from "./animation";

export interface HistoricalModeHandlers {
  captureHistoricalFreezeTimestamp: () => number;
  getHistoricalEndTimestamp: () => number;
  handleHistoricalTimeWindow: (timeWindow: number | "all") => void;
  switchToHistoricalMode: () => void;
  switchToLiveMode: () => void;
}

export function useHistoricalMode({
  newData,
  uplotRef,
  animationRefs,
  getCurrentLiveEndTimestamp,
  updateYAxisScale,
  lastProcessedCountRef,
  manualScaleRef,
}: {
  newData: any;
  uplotRef: React.RefObject<uPlot | null>;
  animationRefs: AnimationRefs;
  getCurrentLiveEndTimestamp: () => number;
  updateYAxisScale: (
    timestamps: number[],
    values: number[],
    xMin?: number,
    xMax?: number,
  ) => void;
  lastProcessedCountRef: React.RefObject<number>;
  manualScaleRef: React.RefObject<{
    x: { min: number; max: number };
    y: { min: number; max: number };
  } | null>;
}): HistoricalModeHandlers {
  const historicalFreezeTimestampRef = useRef<number | null>(null);
  const localRef = useRef(manualScaleRef.current);
  const localProcessedCount = useRef(lastProcessedCountRef.current);
  const captureHistoricalFreezeTimestamp = useCallback(() => {
    // Always get the current live timestamp when switching to historical
    const currentLiveEnd = getCurrentLiveEndTimestamp();
    historicalFreezeTimestampRef.current = currentLiveEnd;
    return currentLiveEnd;
  }, [getCurrentLiveEndTimestamp]);

  const getHistoricalEndTimestamp = useCallback((): number => {
    // If we have a freeze timestamp, use it
    if (historicalFreezeTimestampRef.current !== null) {
      return historicalFreezeTimestampRef.current;
    }

    // Otherwise get current live end timestamp
    return getCurrentLiveEndTimestamp();
  }, [getCurrentLiveEndTimestamp]);

  const handleHistoricalTimeWindow = useCallback(
    (timeWindow: number | "all") => {
      if (!uplotRef.current || !newData?.long) return;

      const [timestamps, values] = seriesToUPlotData(newData.long);

      if (timeWindow === "all") {
        // Historical mode: show all data but end at the freeze timestamp
        const endTimestamp =
          historicalFreezeTimestampRef.current ??
          captureHistoricalFreezeTimestamp();
        const fullStart = endTimestamp - 24 * 60 * 60 * 1000;

        uplotRef.current.setScale("x", { min: fullStart, max: endTimestamp });
        updateYAxisScale(timestamps, values, fullStart, endTimestamp);

        localRef.current = {
          x: { min: fullStart, max: endTimestamp },
          y: manualScaleRef.current?.y ?? {
            min: Math.min(...values),
            max: Math.max(...values),
          },
        };
      } else {
        // Historical mode with specific time window
        const endTimestamp =
          historicalFreezeTimestampRef.current ??
          captureHistoricalFreezeTimestamp();
        const newViewStart = endTimestamp - timeWindow;

        uplotRef.current.setScale("x", {
          min: newViewStart,
          max: endTimestamp,
        });

        const visibleValues: number[] = [];
        for (let i = 0; i < timestamps.length; i++) {
          if (timestamps[i] >= newViewStart && timestamps[i] <= endTimestamp) {
            visibleValues.push(values[i]);
          }
        }

        const minY = visibleValues.length > 0 ? Math.min(...visibleValues) : 0;
        const maxY = visibleValues.length > 0 ? Math.max(...visibleValues) : 1;

        uplotRef.current.setScale("y", {
          min: minY - (maxY - minY) * 0.1,
          max: maxY + (maxY - minY) * 0.1,
        });

        localRef.current = {
          x: { min: newViewStart, max: endTimestamp },
          y: {
            min: minY - (maxY - minY) * 0.1,
            max: maxY + (maxY - minY) * 0.1,
          },
        };
      }
    },
    [
      uplotRef,
      newData,
      updateYAxisScale,
      manualScaleRef,
      captureHistoricalFreezeTimestamp,
    ],
  );

  const switchToHistoricalMode = useCallback(() => {
    // Capture freeze timestamp when switching to historical
    captureHistoricalFreezeTimestamp();
    stopAnimations(animationRefs);
    localProcessedCount.current = 0;
    localRef.current = null;
  }, [
    captureHistoricalFreezeTimestamp,
    animationRefs,
    lastProcessedCountRef,
    manualScaleRef,
  ]);

  const switchToLiveMode = useCallback(() => {
    // Clear freeze timestamp when switching to live
    historicalFreezeTimestampRef.current = null;
    localRef.current = null;
  }, [manualScaleRef]);

  return {
    captureHistoricalFreezeTimestamp,
    getHistoricalEndTimestamp,
    handleHistoricalTimeWindow,
    switchToHistoricalMode,
    switchToLiveMode,
  };
}
