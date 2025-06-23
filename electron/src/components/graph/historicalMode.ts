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
  syncHistoricalFreezeTimestamp, // ADDED: Get freeze timestamp from sync
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
  lastProcessedCountRef: React.MutableRefObject<number>;
  manualScaleRef: React.MutableRefObject<{
    x: { min: number; max: number };
    y: { min: number; max: number };
  } | null>;
  syncHistoricalFreezeTimestamp?: number | null; // ADDED: Optional sync freeze timestamp
}): HistoricalModeHandlers {
  const localHistoricalFreezeTimestampRef = useRef<number | null>(null);
  const localManualScale = useRef(manualScaleRef.current);

  const captureHistoricalFreezeTimestamp = useCallback(() => {
    // FIXED: Use sync freeze timestamp if available, otherwise capture current time
    const freezeTimestamp =
      syncHistoricalFreezeTimestamp ?? getCurrentLiveEndTimestamp();
    localHistoricalFreezeTimestampRef.current = freezeTimestamp;
    return freezeTimestamp;
  }, [getCurrentLiveEndTimestamp, syncHistoricalFreezeTimestamp]);

  const getHistoricalEndTimestamp = useCallback((): number => {
    // FIXED: Prioritize sync freeze timestamp, then local, then current live
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

  const handleHistoricalTimeWindow = useCallback(
    (timeWindow: number | "all") => {
      if (!uplotRef.current) return;

      const primaryData = getPrimarySeriesData(newData);
      if (!primaryData?.long) return;

      const [timestamps, values] = seriesToUPlotData(primaryData.long);

      if (timestamps.length === 0) return;

      try {
        // FIXED: Always use the frozen end timestamp for historical mode
        const endTimestamp = getHistoricalEndTimestamp();

        let startTimestamp: number;

        if (timeWindow === "all") {
          // Show all available historical data up to freeze point
          startTimestamp = Math.min(...timestamps);
        } else {
          // Show specific time window ending at freeze point
          startTimestamp = endTimestamp - timeWindow;
        }

        // Filter values for the visible time range
        const visibleValues: number[] = [];
        for (let i = 0; i < timestamps.length; i++) {
          if (
            timestamps[i] >= startTimestamp &&
            timestamps[i] <= endTimestamp
          ) {
            visibleValues.push(values[i]);
          }
        }

        // Calculate Y-axis range
        const minY = visibleValues.length > 0 ? Math.min(...visibleValues) : 0;
        const maxY = visibleValues.length > 0 ? Math.max(...visibleValues) : 1;
        const range = maxY - minY || Math.abs(maxY) * 0.1 || 1;

        const yRange = {
          min: minY - range * 0.1,
          max: maxY + range * 0.1,
        };

        // Apply the scale changes
        uplotRef.current.batch(() => {
          uplotRef.current!.setScale("x", {
            min: startTimestamp,
            max: endTimestamp,
          });

          uplotRef.current!.setScale("y", yRange);
        });

        // Update manual scale reference
        localManualScale.current = {
          x: { min: startTimestamp, max: endTimestamp },
          y: yRange,
        };

        manualScaleRef.current = localManualScale.current;
      } catch (error) {
        console.warn("Error in handleHistoricalTimeWindow:", error);
      }
    },
    [uplotRef, newData, manualScaleRef, getHistoricalEndTimestamp],
  );

  const switchToHistoricalMode = useCallback(() => {
    // Capture freeze timestamp when switching to historical
    captureHistoricalFreezeTimestamp();
    stopAnimations(animationRefs);

    // Reset processed count to prevent stale data issues
    const newCount = 0;
    lastProcessedCountRef.current = newCount;
  }, [captureHistoricalFreezeTimestamp, animationRefs, lastProcessedCountRef]);

  const switchToLiveMode = useCallback(() => {
    // FIXED: Clear local freeze timestamp when switching to live
    localHistoricalFreezeTimestampRef.current = null;

    // Clear manual scale to allow live mode to take over
    const newManualScale = null;
    manualScaleRef.current = newManualScale;

    // Reset processed count to force live mode to process all current data
    const newCount = 0;
    lastProcessedCountRef.current = newCount;

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
