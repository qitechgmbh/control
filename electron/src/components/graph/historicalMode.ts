import { useRef, useCallback } from "react";
import uPlot from "uplot";
import { seriesToUPlotData } from "@/lib/timeseries";
import { stopAnimations, AnimationRefs } from "./animation";
import { BigGraphProps, SeriesData, HistoricalModeHandlers } from "./types";

// Helper functions for multi-series support
function normalizeDataSeries(data: BigGraphProps["newData"]): SeriesData[] {
  if (Array.isArray(data)) {
    return data;
  }
  return [data];
}

function getPrimarySeriesData(data: BigGraphProps["newData"]) {
  const normalized = normalizeDataSeries(data);
  const primarySeries = normalized.find((series) => series.newData !== null);
  return primarySeries?.newData || null;
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
  newData: BigGraphProps["newData"];
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
  const localManualScale = useRef(manualScaleRef.current);
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
      if (!uplotRef.current) return;

      const primaryData = getPrimarySeriesData(newData);
      if (!primaryData?.long) return;

      const [timestamps, values] = seriesToUPlotData(primaryData.long);

      if (timestamps.length === 0) return;

      try {
        if (timeWindow === "all") {
          // Historical mode: show all data but end at the freeze timestamp
          const endTimestamp =
            historicalFreezeTimestampRef.current ??
            captureHistoricalFreezeTimestamp();
          const fullStart = endTimestamp - 24 * 60 * 60 * 1000;

          uplotRef.current.batch(() => {
            uplotRef.current!.setScale("x", {
              min: fullStart,
              max: endTimestamp,
            });
            updateYAxisScale(timestamps, values, fullStart, endTimestamp);
          });

          const newLocalManualScale = {
            x: { min: fullStart, max: endTimestamp },
            y: manualScaleRef.current?.y ?? {
              min: Math.min(...values),
              max: Math.max(...values),
            },
          };
          localManualScale.current = newLocalManualScale;
        } else {
          // Historical mode with specific time window
          const endTimestamp =
            historicalFreezeTimestampRef.current ??
            captureHistoricalFreezeTimestamp();
          const newViewStart = endTimestamp - timeWindow;

          const visibleValues: number[] = [];
          for (let i = 0; i < timestamps.length; i++) {
            if (
              timestamps[i] >= newViewStart &&
              timestamps[i] <= endTimestamp
            ) {
              visibleValues.push(values[i]);
            }
          }

          const minY =
            visibleValues.length > 0 ? Math.min(...visibleValues) : 0;
          const maxY =
            visibleValues.length > 0 ? Math.max(...visibleValues) : 1;
          const range = maxY - minY || Math.abs(maxY) * 0.1 || 1;

          uplotRef.current.batch(() => {
            uplotRef.current!.setScale("x", {
              min: newViewStart,
              max: endTimestamp,
            });

            uplotRef.current!.setScale("y", {
              min: minY - range * 0.1,
              max: maxY + range * 0.1,
            });
          });

          const newLocalManualScale = {
            x: { min: newViewStart, max: endTimestamp },
            y: {
              min: minY - range * 0.1,
              max: maxY + range * 0.1,
            },
          };
          localManualScale.current = newLocalManualScale;
        }
      } catch (error) {
        console.warn("Error in handleHistoricalTimeWindow:", error);
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

    // Reset processed count to prevent stale data issues
    const newCount = 0;

    localProcessedCount.current = newCount;
  }, [captureHistoricalFreezeTimestamp, animationRefs, lastProcessedCountRef]);

  const switchToLiveMode = useCallback(() => {
    // Clear freeze timestamp when switching to live
    historicalFreezeTimestampRef.current = null;

    // Clear manual scale to allow live mode to take over
    const newManualScale = null;
    localManualScale.current = newManualScale;

    // Reset processed count to force live mode to process all current data
    const newCount = 0;
    localProcessedCount.current = newCount;

    // Stop any ongoing animations
    stopAnimations(animationRefs);

    // If we have a chart and data, immediately update to current live view
    if (uplotRef.current) {
      const primaryData = getPrimarySeriesData(newData);
      if (primaryData?.long) {
        try {
          const [timestamps, values] = seriesToUPlotData(primaryData.long);
          if (timestamps.length > 0) {
            // Get current live end timestamp
            const currentLiveEnd = getCurrentLiveEndTimestamp();

            // Set to show recent data (last 30 minutes by default)
            const recentStart = currentLiveEnd - 30 * 60 * 1000;

            uplotRef.current.batch(() => {
              uplotRef.current!.setScale("x", {
                min: recentStart,
                max: currentLiveEnd,
              });

              // Update Y scale for visible data
              updateYAxisScale(timestamps, values, recentStart, currentLiveEnd);
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

  // Suppress unused variable warning
  void localManualScale;

  return {
    captureHistoricalFreezeTimestamp,
    getHistoricalEndTimestamp,
    handleHistoricalTimeWindow,
    switchToHistoricalMode,
    switchToLiveMode,
  };
}

export type { HistoricalModeHandlers };
