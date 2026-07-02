/* eslint-disable react-compiler/react-compiler */
import { useCallback } from "react";
import type { IChartApi } from "lightweight-charts";
import {
  seriesToUPlotData,
  alignTargetSeriesToTimestamps,
} from "@/lib/timeseries";
import { getPrimarySeriesData, msToTime } from "./dataHelpers";
import { getAllTimeSeries, setVisibleRangeSilently } from "./createChart";
import { BigGraphProps, LiveModeHandlers, SeriesRefs } from "./types";

export function useLiveMode({
  newData,
  chartRef,
  seriesRefs,
  viewMode,
  selectedTimeWindow,
  startTimeRef,
  suppressRangeEventRef,
  lastProcessedCountRef,
  chartCreatedRef,
}: {
  newData: BigGraphProps["newData"];
  chartRef: React.RefObject<IChartApi | null>;
  seriesRefs: React.RefObject<SeriesRefs>;
  viewMode: "default" | "all" | "manual";
  selectedTimeWindow: number | "all";
  startTimeRef: React.RefObject<number | null>;
  suppressRangeEventRef: React.RefObject<boolean>;
  lastProcessedCountRef: React.RefObject<number>;
  chartCreatedRef: React.RefObject<boolean>;
}): LiveModeHandlers {
  // Get the latest timestamp for live mode, considering current and historical data
  const getCurrentLiveEndTimestamp = useCallback((): number => {
    const primaryData = getPrimarySeriesData(newData);
    if (!primaryData?.long) return Date.now();

    const [timestamps] = seriesToUPlotData(primaryData.long);
    if (timestamps.length === 0) return Date.now();

    const lastDataTimestamp = timestamps[timestamps.length - 1];
    const currentTimestamp = primaryData.current?.timestamp;

    return currentTimestamp && currentTimestamp > lastDataTimestamp
      ? currentTimestamp
      : lastDataTimestamp;
  }, [newData]);

  // Push the latest sub-second sample onto every series as a single
  // incremental point — replaces the old full-array setData()-per-tick.
  const updateLiveData = useCallback(() => {
    if (!chartRef.current || !seriesRefs.current) return;

    const allOriginalSeries = getAllTimeSeries(newData);
    if (allOriginalSeries.length === 0) return;

    const cur = allOriginalSeries[0].series.current;
    if (!cur) return;

    try {
      const latestTimestamp = cur.timestamp;
      const time = msToTime(latestTimestamp);

      seriesRefs.current.dataSeries.forEach((series, index) => {
        const seriesData = allOriginalSeries[index]?.series;
        if (!seriesData) return;
        const [, values] = seriesToUPlotData(seriesData.long);
        const value =
          seriesData.current?.value ??
          (values.length > 0 ? values[values.length - 1] : undefined);
        if (value === undefined) return;
        series.update({ time, value });
      });

      seriesRefs.current.lineSeries.forEach(({ api, line }) => {
        const targetSeries =
          line.type === "target" ? line.targetSeries : undefined;
        const value = targetSeries
          ? alignTargetSeriesToTimestamps(
              targetSeries,
              [latestTimestamp],
              line.value,
            )[0]
          : line.value;
        api.update({ time, value });
      });

      if (viewMode === "default") {
        let xMin: number, xMax: number;
        if (selectedTimeWindow === "all") {
          xMin = startTimeRef.current ?? latestTimestamp;
          xMax = latestTimestamp;
        } else {
          xMin = latestTimestamp - (selectedTimeWindow as number);
          xMax = latestTimestamp;
        }
        setVisibleRangeSilently(chartRef.current, suppressRangeEventRef, {
          min: xMin,
          max: xMax,
        });
      } else if (viewMode === "all") {
        const fullStart = startTimeRef.current ?? latestTimestamp;
        setVisibleRangeSilently(chartRef.current, suppressRangeEventRef, {
          min: fullStart,
          max: latestTimestamp,
        });
      }
    } catch (error) {
      console.warn("Error in updateLiveData:", error);
    }
  }, [
    newData,
    chartRef,
    seriesRefs,
    viewMode,
    selectedTimeWindow,
    startTimeRef,
    suppressRangeEventRef,
  ]);

  // Handle changes to the live time window
  const handleLiveTimeWindow = useCallback(
    (timeWindow: number | "all") => {
      if (!chartRef.current) return;

      const primaryData = getPrimarySeriesData(newData);
      if (!primaryData?.long) return;

      try {
        const [timestamps] = seriesToUPlotData(primaryData.long);

        if (timeWindow === "all") {
          const fullStart =
            timestamps.length > 0
              ? timestamps[0]
              : Date.now() - 24 * 60 * 60 * 1000;
          const fullEnd =
            timestamps.length > 0
              ? timestamps[timestamps.length - 1]
              : Date.now();

          setVisibleRangeSilently(chartRef.current, suppressRangeEventRef, {
            min: fullStart,
            max: fullEnd,
          });
        } else {
          const latestTimestamp =
            timestamps.length > 0
              ? timestamps[timestamps.length - 1]
              : Date.now();
          const viewStart = latestTimestamp - timeWindow;

          setVisibleRangeSilently(chartRef.current, suppressRangeEventRef, {
            min: viewStart,
            max: latestTimestamp,
          });
        }

        lastProcessedCountRef.current = timestamps.length;
      } catch (error) {
        console.warn("Error in handleLiveTimeWindow:", error);
      }
    },
    [chartRef, newData, suppressRangeEventRef, lastProcessedCountRef],
  );

  // Reconcile a burst of newly-retained historical points (e.g. after the app
  // was backgrounded and reconnects with many buffered samples at once). A
  // single new point is already reflected by updateLiveData's update() call
  // above, so this only needs to act when more than one point arrived at once.
  const processNewHistoricalData = useCallback(() => {
    if (!chartRef.current || !chartCreatedRef.current || !seriesRefs.current) {
      return;
    }

    const allOriginalSeries = getAllTimeSeries(newData);
    if (allOriginalSeries.length === 0) return;

    const primaryLong = allOriginalSeries[0].series.long;
    const currentValidCount = primaryLong.validCount;
    const previousCount = lastProcessedCountRef.current;

    if (currentValidCount - previousCount <= 1) {
      lastProcessedCountRef.current = currentValidCount;
      return;
    }

    try {
      allOriginalSeries.forEach(({ series: timeSeries }, index) => {
        const [timestamps, values] = seriesToUPlotData(timeSeries.long);
        seriesRefs.current!.dataSeries[index]?.setData(
          timestamps.map((t, i) => ({ time: msToTime(t), value: values[i] })),
        );
      });

      const [timestamps] = seriesToUPlotData(primaryLong);
      seriesRefs.current.lineSeries.forEach(({ api, line }) => {
        const targetSeries =
          line.type === "target" ? line.targetSeries : undefined;
        const lineData = targetSeries
          ? alignTargetSeriesToTimestamps(targetSeries, timestamps, line.value)
          : timestamps.map(() => line.value);
        api.setData(
          timestamps.map((t, i) => ({ time: msToTime(t), value: lineData[i] })),
        );
      });

      lastProcessedCountRef.current = currentValidCount;
    } catch (error) {
      console.warn("Error in processNewHistoricalData:", error);
    }
  }, [newData, chartRef, chartCreatedRef, seriesRefs, lastProcessedCountRef]);

  return {
    getCurrentLiveEndTimestamp,
    updateLiveData,
    handleLiveTimeWindow,
    processNewHistoricalData,
  };
}
export type { LiveModeHandlers };
