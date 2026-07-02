/* eslint-disable react-compiler/react-compiler */
import { useRef, useCallback } from "react";
import type { IChartApi } from "lightweight-charts";
import { seriesToUPlotData } from "@/lib/timeseries";
import { getPrimarySeriesData } from "./dataHelpers";
import { setVisibleRangeSilently, setYAutoScale } from "./createChart";
import { BigGraphProps, HistoricalModeHandlers, SwitchOrigin } from "./types";

export function useHistoricalMode({
  newData,
  chartRef,
  suppressRangeEventRef,
  getCurrentLiveEndTimestamp,
  lastProcessedCountRef,
  manualScaleRef,
  syncHistoricalFreezeTimestamp,
}: {
  newData: BigGraphProps["newData"];
  chartRef: React.RefObject<IChartApi | null>;
  suppressRangeEventRef: React.RefObject<boolean>;
  getCurrentLiveEndTimestamp: () => number;
  lastProcessedCountRef: React.RefObject<number>;
  manualScaleRef: React.RefObject<{
    x: { min: number; max: number };
  } | null>;
  syncHistoricalFreezeTimestamp?: number | null;
}): HistoricalModeHandlers {
  const localHistoricalFreezeTimestampRef = useRef<number | null>(null);

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
      if (!chartRef.current) return;

      const primaryData = getPrimarySeriesData(newData);
      if (!primaryData?.long) return;

      const [timestamps] = seriesToUPlotData(primaryData.long);
      if (timestamps.length === 0) return;

      try {
        const endTimestamp = getHistoricalEndTimestamp();
        const startTimestamp =
          timeWindow === "all"
            ? Math.min(...timestamps)
            : endTimestamp - timeWindow;

        setVisibleRangeSilently(chartRef.current, suppressRangeEventRef, {
          min: startTimestamp,
          max: endTimestamp,
        });

        manualScaleRef.current = {
          x: { min: startTimestamp, max: endTimestamp },
        };
      } catch (error) {
        console.warn("Error in handleHistoricalTimeWindow:", error);
      }
    },
    [
      chartRef,
      newData,
      manualScaleRef,
      getHistoricalEndTimestamp,
      suppressRangeEventRef,
    ],
  );

  // Switch to historical mode
  const switchToHistoricalMode = useCallback(
    (origin?: SwitchOrigin) => {
      // "gesture" origin: manualScaleRef was already set by the chart's own
      // range-change subscription at the moment of the zoom (or will shortly
      // be set by the incoming xRange sync for sibling graphs) — leave it.
      if (origin !== "gesture") {
        manualScaleRef.current = null;
      }

      captureHistoricalFreezeTimestamp();
      lastProcessedCountRef.current = 0;

      if (chartRef.current) {
        setYAutoScale(chartRef.current, false);
      }
    },
    [
      captureHistoricalFreezeTimestamp,
      lastProcessedCountRef,
      manualScaleRef,
      chartRef,
    ],
  );

  // Switch back to live mode
  const switchToLiveMode = useCallback(() => {
    localHistoricalFreezeTimestampRef.current = null;
    manualScaleRef.current = null;
    lastProcessedCountRef.current = 0;

    if (chartRef.current) {
      setYAutoScale(chartRef.current, true);

      const primaryData = getPrimarySeriesData(newData);
      if (primaryData?.long) {
        try {
          const [timestamps] = seriesToUPlotData(primaryData.long);
          if (timestamps.length > 0) {
            const currentLiveEnd = getCurrentLiveEndTimestamp();
            const recentStart = currentLiveEnd - 30 * 60 * 1000; // Last 30 minutes

            setVisibleRangeSilently(chartRef.current, suppressRangeEventRef, {
              min: recentStart,
              max: currentLiveEnd,
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
    chartRef,
    newData,
    getCurrentLiveEndTimestamp,
    suppressRangeEventRef,
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
