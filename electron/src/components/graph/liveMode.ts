import { useCallback, useRef } from "react";
import uPlot from "uplot";
import { seriesToUPlotData } from "@/lib/timeseries";
import { buildUPlotData, animateNewPoint, AnimationRefs } from "./animation";
import {
  GraphConfig,
  BigGraphProps,
  SeriesData,
  LiveModeHandlers,
} from "./types";

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

function getAllSeriesData(data: BigGraphProps["newData"]): number[][] {
  const normalized = normalizeDataSeries(data);
  return normalized
    .filter((series) => series.newData !== null)
    .map((series) => {
      if (!series.newData?.long) return [];
      const [, values] = seriesToUPlotData(series.newData.long);
      return values;
    })
    .filter((values) => values.length > 0);
}

export function useLiveMode({
  newData,
  uplotRef,
  config,
  animationRefs,
  viewMode,
  selectedTimeWindow,
  startTimeRef,
  updateYAxisScale,
  lastProcessedCountRef,
  chartCreatedRef,
}: {
  newData: BigGraphProps["newData"];
  uplotRef: React.RefObject<uPlot | null>;
  config: GraphConfig;
  animationRefs: AnimationRefs;
  viewMode: "default" | "all" | "manual";
  selectedTimeWindow: number | "all";
  startTimeRef: React.RefObject<number | null>;
  updateYAxisScale: (
    timestamps: number[],
    values: number[],
    xMin?: number,
    xMax?: number,
  ) => void;
  lastProcessedCountRef: React.RefObject<number>;
  chartCreatedRef: React.RefObject<boolean>;
}): LiveModeHandlers {
  const localStartTime = useRef(startTimeRef.current);
  const localanimationData = useRef(animationRefs.lastRenderedData.current);
  const localProcessedCount = useRef(lastProcessedCountRef.current);

  const getCurrentLiveEndTimestamp = useCallback((): number => {
    const primaryData = getPrimarySeriesData(newData);
    if (!primaryData?.long) return Date.now();

    const [timestamps] = seriesToUPlotData(primaryData.long);
    if (timestamps.length === 0) return Date.now();

    // Use current data if available and newer, otherwise use last data point
    const lastDataTimestamp = timestamps[timestamps.length - 1];
    const currentTimestamp = primaryData.current?.timestamp;

    if (currentTimestamp && currentTimestamp > lastDataTimestamp) {
      return currentTimestamp;
    }

    return lastDataTimestamp;
  }, [newData]);

  const updateLiveData = useCallback(() => {
    const primaryData = getPrimarySeriesData(newData);
    if (!primaryData?.long || !primaryData?.current || !uplotRef.current)
      return;

    try {
      const [timestamps, values] = seriesToUPlotData(primaryData.long);
      const cur = primaryData.current;
      const liveTimestamps = [...timestamps];
      const liveValues = [...values];

      liveTimestamps.push(cur.timestamp);
      liveValues.push(cur.value);

      // Get all series data and extend with current values
      const allSeriesValues = getAllSeriesData(newData);
      const normalized = normalizeDataSeries(newData);

      // Only include additional series if we have multiple series
      let additionalSeriesValues: number[][] = [];
      if (normalized.length > 1) {
        // Extend each series with current or last known value
        additionalSeriesValues = allSeriesValues
          .slice(1)
          .map((seriesValues, index) => {
            const seriesIndex = index + 1; // Account for primary series
            const seriesData = normalized[seriesIndex]?.newData;
            const currentValue =
              seriesData?.current?.value ??
              (seriesValues.length > 0
                ? seriesValues[seriesValues.length - 1]
                : 0);
            return [...seriesValues, currentValue];
          });
      }

      const liveData = buildUPlotData(
        liveTimestamps,
        liveValues,
        undefined,
        animationRefs.realPointsCount,
        config,
        additionalSeriesValues,
      );

      uplotRef.current.setData(liveData);

      const latestTimestamp = liveTimestamps[liveTimestamps.length - 1];

      if (viewMode === "default") {
        let xMin, xMax;

        if (selectedTimeWindow === "all") {
          const fullStart = startTimeRef.current ?? liveTimestamps[0];
          xMin = fullStart;
          xMax = latestTimestamp;
        } else {
          xMin = latestTimestamp - (selectedTimeWindow as number);
          xMax = latestTimestamp;
        }

        uplotRef.current.batch(() => {
          uplotRef.current!.setScale("x", { min: xMin, max: xMax });
          updateYAxisScale(liveTimestamps, liveValues, xMin, xMax);
        });
      } else if (viewMode === "all") {
        const fullStart = startTimeRef.current ?? liveTimestamps[0];
        uplotRef.current.batch(() => {
          uplotRef.current!.setScale("x", {
            min: fullStart,
            max: latestTimestamp,
          });
          updateYAxisScale(
            liveTimestamps,
            liveValues,
            fullStart,
            latestTimestamp,
          );
        });
      }

      if (startTimeRef.current === null && liveTimestamps.length > 0) {
        localStartTime.current = liveTimestamps[0];
      }
    } catch (error) {
      console.warn("Error in updateLiveData:", error);
    }
  }, [
    newData,
    uplotRef,
    config,
    animationRefs,
    viewMode,
    selectedTimeWindow,
    startTimeRef,
    updateYAxisScale,
  ]);

  const handleLiveTimeWindow = useCallback(
    (timeWindow: number | "all") => {
      if (!uplotRef.current) return;

      const primaryData = getPrimarySeriesData(newData);
      if (!primaryData?.long) return;

      try {
        const [timestamps, values] = seriesToUPlotData(primaryData.long);

        if (timeWindow === "all") {
          // Live mode: show all data up to the latest timestamp
          const fullStart =
            timestamps.length > 0
              ? timestamps[0]
              : Date.now() - 24 * 60 * 60 * 1000;
          const fullEnd =
            timestamps.length > 0
              ? timestamps[timestamps.length - 1]
              : Date.now();

          uplotRef.current.batch(() => {
            uplotRef.current!.setScale("x", { min: fullStart, max: fullEnd });
            updateYAxisScale(timestamps, values, fullStart, fullEnd);
          });
        } else {
          // Handle specific time window in live mode
          const latestTimestamp =
            timestamps.length > 0
              ? timestamps[timestamps.length - 1]
              : Date.now();
          const viewStart = latestTimestamp - timeWindow;

          uplotRef.current.batch(() => {
            uplotRef.current!.setScale("x", {
              min: viewStart,
              max: latestTimestamp,
            });
            updateYAxisScale(timestamps, values, viewStart, latestTimestamp);
          });
        }

        localanimationData.current = { timestamps, values };
        localProcessedCount.current = timestamps.length;
      } catch (error) {
        console.warn("Error in handleLiveTimeWindow:", error);
      }
    },
    [uplotRef, newData, updateYAxisScale, animationRefs, lastProcessedCountRef],
  );

  const processNewHistoricalData = useCallback(() => {
    if (
      !uplotRef.current ||
      !chartCreatedRef.current ||
      viewMode === "manual"
    ) {
      return;
    }

    const primaryData = getPrimarySeriesData(newData);
    if (!primaryData?.long) return;

    try {
      const [timestamps, values] = seriesToUPlotData(primaryData.long);
      if (timestamps.length === 0) return;

      // Check if we need to process new data
      if (timestamps.length <= lastProcessedCountRef.current) {
        return;
      }

      const currentData = animationRefs.lastRenderedData.current;
      const targetData = { timestamps, values };

      // Create function to get all series data for animation
      const getAllSeriesDataForAnimation = () => {
        const allSeriesValues = getAllSeriesData(newData);
        return allSeriesValues.slice(1); // Skip primary series as it's already included
      };

      if (targetData.timestamps.length > currentData.timestamps.length) {
        const maxAnimatableLength = Math.min(
          targetData.timestamps.length,
          currentData.timestamps.length + 1,
        );

        const limitedTargetData = {
          timestamps: targetData.timestamps.slice(0, maxAnimatableLength),
          values: targetData.values.slice(0, maxAnimatableLength),
        };

        animateNewPoint(
          currentData,
          limitedTargetData,
          animationRefs,
          uplotRef,
          true, // isLiveMode
          viewMode,
          selectedTimeWindow,
          startTimeRef,
          config,
          updateYAxisScale,
          getAllSeriesDataForAnimation,
        );
      } else if (
        targetData.timestamps.length === currentData.timestamps.length
      ) {
        let hasChanges = false;
        for (let i = 0; i < targetData.values.length; i++) {
          if (Math.abs(targetData.values[i] - currentData.values[i]) > 0.001) {
            hasChanges = true;
            break;
          }
        }

        if (hasChanges) {
          const allSeriesValues = getAllSeriesData(newData);
          const additionalSeriesValues = allSeriesValues.slice(1); // Skip primary series

          const uData = buildUPlotData(
            timestamps,
            values,
            undefined,
            animationRefs.realPointsCount,
            config,
            additionalSeriesValues,
          );

          uplotRef.current.setData(uData);
          localanimationData.current = { timestamps, values };

          const lastTimestamp = timestamps[timestamps.length - 1] ?? 0;
          if (viewMode === "default") {
            let xMin: number | undefined, xMax: number | undefined;

            if (selectedTimeWindow === "all") {
              const fullStart = startTimeRef.current ?? timestamps[0];
              xMin = fullStart;
              xMax = lastTimestamp;
            } else {
              xMin = lastTimestamp - (selectedTimeWindow as number);
              xMax = lastTimestamp;
            }

            uplotRef.current.batch(() => {
              uplotRef.current!.setScale("x", { min: xMin, max: xMax });
              updateYAxisScale(timestamps, values, xMin, xMax);
            });
          } else if (viewMode === "all") {
            const fullStart = startTimeRef.current ?? timestamps[0];
            uplotRef.current.batch(() => {
              uplotRef.current!.setScale("x", {
                min: fullStart,
                max: lastTimestamp,
              });
              updateYAxisScale(timestamps, values, fullStart, lastTimestamp);
            });
          }
        }
      }
    } catch (error) {
      console.warn("Error in processNewHistoricalData:", error);
    }
  }, [
    uplotRef,
    newData,
    chartCreatedRef,
    viewMode,
    animationRefs,
    lastProcessedCountRef,
    selectedTimeWindow,
    startTimeRef,
    config,
    updateYAxisScale,
  ]);

  return {
    getCurrentLiveEndTimestamp,
    updateLiveData,
    handleLiveTimeWindow,
    processNewHistoricalData,
  };
}
export type { LiveModeHandlers };
