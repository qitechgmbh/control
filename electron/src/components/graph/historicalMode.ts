/* eslint-disable react-compiler/react-compiler */
import { useRef, useCallback } from "react";
import uPlot from "uplot";
import { seriesToUPlotData } from "@/lib/timeseries";
import { getPrimarySeriesData, stopAnimations } from "./animation";
import { BigGraphProps, HistoricalModeHandlers, AnimationRefs } from "./types";

export function useHistoricalMode({
  newData,
  uplotRef,
  animationRefs,
  getCurrentLiveEndTimestamp,
  updateYAxisScale,
  lastProcessedCountRef,
  manualScaleRef,
  syncHistoricalFreezeTimestamp,
}: {
  newData: BigGraphProps["newData"];
  uplotRef: React.RefObject<uPlot | null>;
  animationRefs: AnimationRefs;
  getCurrentLiveEndTimestamp: () => number;
  updateYAxisScale: (xMin?: number, xMax?: number) => void;
  lastProcessedCountRef: React.RefObject<number>;
  manualScaleRef: React.RefObject<{
    x: { min: number; max: number };
    y: { min: number; max: number };
  } | null>;
  syncHistoricalFreezeTimestamp?: number | null;
}): HistoricalModeHandlers {
  const localHistoricalFreezeTimestampRef = useRef<number | null>(null);
  const localManualScale = useRef(manualScaleRef.current);
  const isInHistoricalModeRef = useRef(false);

  // Capture the timestamp to freeze the historical view
  const captureHistoricalFreezeTimestamp = useCallback(() => {
    const freezeTimestamp =
      syncHistoricalFreezeTimestamp ?? getCurrentLiveEndTimestamp();
    localHistoricalFreezeTimestampRef.current = freezeTimestamp;
    return freezeTimestamp;
  }, [getCurrentLiveEndTimestamp, syncHistoricalFreezeTimestamp]);

  // Get the end timestamp for the historical view
  const getHistoricalEndTimestamp = useCallback((): number => {
    if (
      syncHistoricalFreezeTimestamp !== null &&
      syncHistoricalFreezeTimestamp !== undefined
    ) {
      return syncHistoricalFreezeTimestamp;
    }
    if (localHistoricalFreezeTimestampRef.current !== null) {
      return localHistoricalFreezeTimestampRef.current;
    }
    return getCurrentLiveEndTimestamp();
  }, [getCurrentLiveEndTimestamp, syncHistoricalFreezeTimestamp]);

  // Handle the time window for historical mode
  const handleHistoricalTimeWindow = useCallback(
    (timeWindow: number | "all") => {
      if (!uplotRef.current) return;

      const primaryData = getPrimarySeriesData(newData);
      if (!primaryData?.long) return;

      const [timestamps, values] = seriesToUPlotData(primaryData.long);
      if (timestamps.length === 0) return;

      try {
        const endTimestamp = getHistoricalEndTimestamp();
        let startTimestamp: number;

        if (timeWindow === "all") {
          startTimestamp = Math.min(...timestamps);
        } else {
          startTimestamp = endTimestamp - timeWindow;
        }

        // Calculate Y-axis range for the visible data
        const visibleValues: number[] = [];
        for (let i = 0; i < timestamps.length; i++) {
          if (
            timestamps[i] >= startTimestamp &&
            timestamps[i] <= endTimestamp
          ) {
            visibleValues.push(values[i]);
          }
        }

        let yMin: number, yMax: number;
        if (visibleValues.length > 0) {
          yMin = Math.min(...visibleValues);
          yMax = Math.max(...visibleValues);
          const range = yMax - yMin || Math.abs(yMax) * 0.1 || 1;
          yMin -= range * 0.1;
          yMax += range * 0.1;
        } else {
          yMin = -1;
          yMax = 1;
        }

        // Update the chart scales
        uplotRef.current.batch(() => {
          uplotRef.current!.setScale("x", {
            min: startTimestamp,
            max: endTimestamp,
          });
          uplotRef.current!.setScale("y", { min: yMin, max: yMax });
        });

        // Update manual scale references
        localManualScale.current = {
          x: { min: startTimestamp, max: endTimestamp },
          y: { min: yMin, max: yMax },
        };
        manualScaleRef.current = localManualScale.current;
      } catch (error) {
        console.warn("Error in handleHistoricalTimeWindow:", error);
      }
    },
    [uplotRef, newData, manualScaleRef, getHistoricalEndTimestamp],
  );

  // Switch to historical mode
  const switchToHistoricalMode = useCallback(() => {
    isInHistoricalModeRef.current = true;
    captureHistoricalFreezeTimestamp();
    stopAnimations(animationRefs);
    lastProcessedCountRef.current = 0; // Reset processed count
  }, [captureHistoricalFreezeTimestamp, animationRefs, lastProcessedCountRef]);

  // Switch back to live mode
  const switchToLiveMode = useCallback(() => {
    isInHistoricalModeRef.current = false;
    localHistoricalFreezeTimestampRef.current = null; // Clear freeze timestamp
    manualScaleRef.current = null; // Clear manual scale
    lastProcessedCountRef.current = 0; // Reset processed count
    stopAnimations(animationRefs);

    if (uplotRef.current) {
      const primaryData = getPrimarySeriesData(newData);
      if (primaryData?.long) {
        try {
          const [timestamps] = seriesToUPlotData(primaryData.long);
          if (timestamps.length > 0) {
            const currentLiveEnd = getCurrentLiveEndTimestamp();
            const recentStart = currentLiveEnd - 30 * 60 * 1000; // Last 30 minutes

            uplotRef.current.batch(() => {
              uplotRef.current!.setScale("x", {
                min: recentStart,
                max: currentLiveEnd,
              });
              updateYAxisScale(recentStart, currentLiveEnd);
            });
          }
        } catch (error) {
          console.warn("Error switching to live mode:", error);
        }
      }
    }
  }, [
    manualScaleRef,
    lastProcessedCountRef,
    animationRefs,
    uplotRef,
    newData,
    getCurrentLiveEndTimestamp,
    updateYAxisScale,
  ]);

  return {
    captureHistoricalFreezeTimestamp,
    getHistoricalEndTimestamp,
    handleHistoricalTimeWindow,
    switchToHistoricalMode,
    switchToLiveMode,
  };
}

export type { HistoricalModeHandlers };
