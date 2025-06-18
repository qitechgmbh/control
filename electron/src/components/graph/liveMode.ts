import { useCallback, useRef } from "react";
import uPlot from "uplot";
import { seriesToUPlotData } from "@/lib/timeseries";
import { buildUPlotData, animateNewPoint, AnimationRefs } from "./animation";
import { GraphConfig } from "./types";
export interface LiveModeHandlers {
  getCurrentLiveEndTimestamp: () => number;
  updateLiveData: () => void;
  handleLiveTimeWindow: (timeWindow: number | "all") => void;
  processNewHistoricalData: () => void;
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
  newData: any;
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
    if (!newData?.long) return Date.now();

    const [timestamps] = seriesToUPlotData(newData.long);
    if (timestamps.length === 0) return Date.now();

    // Use current data if available and newer, otherwise use last data point
    const lastDataTimestamp = timestamps[timestamps.length - 1];
    const currentTimestamp = newData.current?.timestamp;

    if (currentTimestamp && currentTimestamp > lastDataTimestamp) {
      return currentTimestamp;
    }

    return lastDataTimestamp;
  }, [newData]);

  const updateLiveData = useCallback(() => {
    if (!newData?.long || !newData?.current || !uplotRef.current) return;

    const [timestamps, values] = seriesToUPlotData(newData.long);
    const cur = newData.current;
    const liveTimestamps = [...timestamps];
    const liveValues = [...values];

    liveTimestamps.push(cur.timestamp);
    liveValues.push(cur.value);

    const liveData = buildUPlotData(
      liveTimestamps,
      liveValues,
      undefined,
      animationRefs.realPointsCount,
      config,
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
      if (!uplotRef.current || !newData?.long) return;

      const [timestamps, values] = seriesToUPlotData(newData.long);

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

        uplotRef.current.setScale("x", { min: fullStart, max: fullEnd });
        updateYAxisScale(timestamps, values, fullStart, fullEnd);
      } else {
        // Handle specific time window in live mode
        const latestTimestamp =
          timestamps.length > 0
            ? timestamps[timestamps.length - 1]
            : Date.now();
        const viewStart = latestTimestamp - timeWindow;

        uplotRef.current.setScale("x", {
          min: viewStart,
          max: latestTimestamp,
        });
        updateYAxisScale(timestamps, values, viewStart, latestTimestamp);
      }
      localanimationData.current = { timestamps, values };
      localProcessedCount.current = timestamps.length;
    },
    [uplotRef, newData, updateYAxisScale, animationRefs, lastProcessedCountRef],
  );

  const processNewHistoricalData = useCallback(() => {
    if (
      !uplotRef.current ||
      !newData?.long ||
      !chartCreatedRef.current ||
      viewMode === "manual"
    ) {
      return;
    }

    const [timestamps, values] = seriesToUPlotData(newData.long);
    if (timestamps.length === 0) return;

    if (timestamps.length <= lastProcessedCountRef.current) {
      return;
    }

    const currentData = animationRefs.lastRenderedData.current;
    const targetData = { timestamps, values };

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
      );
    } else if (targetData.timestamps.length === currentData.timestamps.length) {
      let hasChanges = false;
      for (let i = 0; i < targetData.values.length; i++) {
        if (Math.abs(targetData.values[i] - currentData.values[i]) > 0.001) {
          hasChanges = true;
          break;
        }
      }

      if (hasChanges) {
        const uData = buildUPlotData(
          timestamps,
          values,
          undefined,
          animationRefs.realPointsCount,
          config,
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

          uplotRef.current.setScale("x", { min: xMin, max: xMax });
          updateYAxisScale(timestamps, values, xMin, xMax);
        } else if (viewMode === "all") {
          const fullStart = startTimeRef.current ?? timestamps[0];
          uplotRef.current.setScale("x", {
            min: fullStart,
            max: lastTimestamp,
          });
          updateYAxisScale(timestamps, values, fullStart, lastTimestamp);
        }
      }
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
