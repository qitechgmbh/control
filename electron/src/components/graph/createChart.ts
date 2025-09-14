import uPlot from "uplot";
import { seriesToUPlotData, TimeSeries } from "@/lib/timeseries";
import {
  createEventHandlers,
  attachEventHandlers,
  HandlerCallbacks,
} from "./handlers";
import { BigGraphProps, CreateChartParams } from "./types";
import { normalizeDataSeries } from "./animation";

export function createChart({
  containerRef,
  uplotRef,
  newData,
  config,
  colors,
  renderValue,
  viewMode,
  selectedTimeWindow,
  isLiveMode,
  startTimeRef,
  manualScaleRef,
  animationRefs,
  handlerRefs,
  graphId,
  syncGraph,
  getHistoricalEndTimestamp,
  updateYAxisScale,
  setViewMode,
  setIsLiveMode,
  setCursorValue,
  setCursorValues,
  visibleSeries,
  showFromTimestamp,
}: CreateChartParams): (() => void) | undefined {
  if (!containerRef.current) return;

  // Get ALL series from original data
  const allOriginalSeries = getAllTimeSeries(newData);
  if (allOriginalSeries.length === 0) return;

  // Use the first available series for timing calculations (visible or not)
  const primarySeries = allOriginalSeries[0].series;
  const [timestamps, primaryValues] = seriesToUPlotData(primarySeries.long);
  if (timestamps.length === 0) return;

  if (startTimeRef.current === null && timestamps.length > 0) {
    startTimeRef.current = timestamps[0];
  }

  // Build uPlot data for ALL series (including hidden ones)
  const uPlotData: uPlot.AlignedData = [timestamps as any];

  allOriginalSeries.forEach(({ series }) => {
    const [, values] = seriesToUPlotData(series.long);
    uPlotData.push(values as any);
  });

  // Add config lines data
  config.lines?.forEach((line) => {
    if (line.show !== false) {
      const lineData = new Array(timestamps.length).fill(line.value);
      uPlotData.push(lineData as any);
    }
  });

  const fullStart = startTimeRef.current ?? timestamps[0] ?? 0;

  let initialMin: number, initialMax: number;
  if (viewMode === "manual" && manualScaleRef.current) {
    initialMin = manualScaleRef.current.x.min;
    initialMax = manualScaleRef.current.x.max;
  } else if (selectedTimeWindow === "all") {
    initialMin = showFromTimestamp ?? fullStart;
    initialMax = getHistoricalEndTimestamp();
  } else {
    const endTimestamp = getHistoricalEndTimestamp();

    if (isLiveMode) {
      const defaultViewStart = Math.max(
        endTimestamp - (selectedTimeWindow as number),
        fullStart,
      );
      initialMin = showFromTimestamp ?? defaultViewStart;
    } else {
      initialMin =
        showFromTimestamp ?? endTimestamp - (selectedTimeWindow as number);
    }

    initialMax = endTimestamp;
  }

  // Calculate initial Y range from ALL series (not just visible ones)
  // This prevents scale changes when toggling visibility
  const initialAllValues: number[] = [];
  allOriginalSeries.forEach(({ series }) => {
    const [seriesTimestamps, seriesValues] = seriesToUPlotData(series.long);
    for (let i = 0; i < seriesTimestamps.length; i++) {
      if (
        seriesTimestamps[i] >= initialMin &&
        seriesTimestamps[i] <= initialMax
      ) {
        initialAllValues.push(seriesValues[i]);
      }
    }
  });

  config.lines?.forEach((line) => {
    if (line.show !== false) {
      initialAllValues.push(line.value);
    }
  });

  if (initialAllValues.length === 0) {
    allOriginalSeries.forEach(({ series }) => {
      const [, values] = seriesToUPlotData(series.long);
      initialAllValues.push(...values);
    });
    config.lines?.forEach((line) => {
      if (line.show !== false) {
        initialAllValues.push(line.value);
      }
    });
  }

  let initialYMin: number, initialYMax: number;
  if (initialAllValues.length > 0) {
    const minY = Math.min(...initialAllValues);
    const maxY = Math.max(...initialAllValues);
    const range = maxY - minY || Math.abs(maxY) * 0.1 || 1;

    initialYMin = minY - range * 0.1;
    initialYMax = maxY + range * 0.1;
  } else {
    initialYMin = -1;
    initialYMax = 1;
  }

  const rect = containerRef.current.getBoundingClientRect();
  const width = rect.width;
  const height = Math.min(rect.height, window.innerHeight * 0.5);

  // Build series configuration for ALL series (but control visibility)
  const seriesConfig: uPlot.Series[] = [{ label: "Time" }];

  // Add ALL data series with visibility control
  const defaultColors = ["#3b82f6", "#ef4444", "#10b981", "#f59e0b", "#8b5cf6"];
  allOriginalSeries.forEach(({ title, color }, index) => {
    const isVisible = !visibleSeries || visibleSeries[index];

    seriesConfig.push({
      label: title || `Series ${index + 1}`,
      stroke: color || defaultColors[index % defaultColors.length],
      width: 2,
      spanGaps: true,
      show: isVisible, // Control visibility here
      points: {
        show: (_u, _seriesIdx, dataIdx) => {
          return isVisible && dataIdx < animationRefs.realPointsCount.current;
        },
        size: 6,
        stroke: color || defaultColors[index % defaultColors.length],
        fill: color || defaultColors[index % defaultColors.length],
        width: 2,
      },
    });
  });

  // Add config lines
  config.lines?.forEach((line) => {
    if (line.show !== false) {
      seriesConfig.push({
        label: line.label,
        stroke: line.color,
        width: line.width ?? 1,
        dash: line.dash ?? (line.type === "threshold" ? [5, 5] : undefined),
        show: true,
        points: { show: false },
      });
    }
  });

  // Always destroy existing chart before creating new one
  if (uplotRef.current) {
    uplotRef.current.destroy();
    uplotRef.current = null;
  }

  // Create new chart with initial visibility settings
  uplotRef.current = new uPlot(
    {
      width,
      height,
      padding: [-10, 20, -10, 20],
      cursor: {
        show: true,
        x: true,
        y: true,
        drag: {
          x: true,
          y: false,
          setScale: true,
        },
        sync: { key: "myCursor" },
      },
      legend: {
        show: config.showLegend ?? allOriginalSeries.length > 1,
      },
      hooks: {
        setScale: [
          (u) => {
            if (handlerRefs.isUserZoomingRef.current) {
              const xScale = u.scales.x;
              if (xScale.min !== undefined && xScale.max !== undefined) {
                updateYAxisScale(xScale.min, xScale.max);

                manualScaleRef.current = {
                  x: { min: xScale.min ?? 0, max: xScale.max ?? 0 },
                  y: {
                    min: u.scales.y?.min ?? 0,
                    max: u.scales.y?.max ?? 1,
                  },
                };

                setViewMode("manual");
                setIsLiveMode(false);

                if (syncGraph?.onZoomChange) {
                  syncGraph.onZoomChange(graphId, {
                    min: xScale.min ?? 0,
                    max: xScale.max ?? 0,
                  });
                }

                if (syncGraph?.onViewModeChange) {
                  syncGraph.onViewModeChange(graphId, "manual", false);
                }
              }
              handlerRefs.isUserZoomingRef.current = false;
            }
          },
        ],
        setCursor: [
          (u) => {
            if (typeof u.cursor.idx === "number") {
              const timestamp = u.data[0][u.cursor.idx];

              // Handle single series case
              if (allOriginalSeries.length === 1) {
                if (u.data[1] && u.data[1][u.cursor.idx] !== undefined) {
                  const value = u.data[1][u.cursor.idx];
                  const cur = allOriginalSeries[0]?.series?.current;

                  const isNearCurrent =
                    cur &&
                    timestamp !== undefined &&
                    Math.abs(timestamp - cur.timestamp) < 1000;

                  const displayValue = isNearCurrent ? cur.value : value;
                  setCursorValue(displayValue ?? null);
                } else {
                  setCursorValue(null);
                }
              }
              // Handle multiple series case
              else if (setCursorValues && visibleSeries) {
                // Create cursor values array for ALL original series
                const cursorValuesArray: (number | null)[] = new Array(
                  allOriginalSeries.length,
                ).fill(null);

                // Since we include ALL series in uPlot data, indices match directly
                allOriginalSeries.forEach((originalSeries, originalIndex) => {
                  const dataIndex = originalIndex + 1; // +1 because 0 is timestamps

                  if (
                    u.data[dataIndex] &&
                    u.data[dataIndex][u.cursor.idx!] !== undefined
                  ) {
                    const value = u.data[dataIndex][u.cursor.idx!];
                    const cur = originalSeries.series?.current;

                    const isNearCurrent =
                      cur &&
                      timestamp !== undefined &&
                      Math.abs(timestamp - cur.timestamp) < 1000;

                    const displayValue = isNearCurrent ? cur.value : value;
                    cursorValuesArray[originalIndex] = displayValue ?? null;
                  }
                });

                setCursorValues(cursorValuesArray);

                // Set primary cursor value for backward compatibility
                const firstVisibleValue = cursorValuesArray.find(
                  (v) => v !== null,
                );
                setCursorValue(firstVisibleValue ?? null);
              }
            } else {
              setCursorValue(null);
              if (setCursorValues) {
                setCursorValues(new Array(allOriginalSeries.length).fill(null));
              }
            }
          },
        ],
      },
      scales: {
        x: {
          time: true,
          auto: true,
          min: initialMin,
          max: initialMax,
          distr: 1,
        },
        y: {
          auto: false,
          min: initialYMin,
          max: initialYMax,
        },
      },

      axes: [
        {
          stroke: colors.axis,
          labelSize: 12,
          labelFont: "Inter, system-ui, sans-serif",
          grid: { stroke: colors.grid, width: 1 },
          space: 60,
          values: (u, ticks) => {
            const xScale = u.scales.x;
            if (
              !xScale ||
              xScale.min === undefined ||
              xScale.max === undefined
            ) {
              return ticks.map((ts) => {
                const date = new Date(ts);
                return date.toLocaleTimeString(undefined, {
                  hour12: false,
                  hour: "2-digit",
                  minute: "2-digit",
                  second: "2-digit",
                });
              });
            }

            const timeRange = xScale.max - xScale.min;

            return ticks.map((ts) => {
              const date = new Date(ts);

              if (timeRange <= 30 * 1000) {
                return date.toLocaleTimeString(undefined, {
                  hour12: false,
                  hour: "2-digit",
                  minute: "2-digit",
                  second: "2-digit",
                });
              } else if (timeRange <= 5 * 60 * 1000) {
                return date.toLocaleTimeString(undefined, {
                  hour12: false,
                  hour: "2-digit",
                  minute: "2-digit",
                  second: "2-digit",
                });
              } else if (timeRange <= 60 * 60 * 1000) {
                return date.toLocaleTimeString(undefined, {
                  hour12: false,
                  hour: "2-digit",
                  minute: "2-digit",
                });
              } else if (timeRange <= 24 * 60 * 60 * 1000) {
                return date.toLocaleTimeString(undefined, {
                  hour12: false,
                  hour: "2-digit",
                  minute: "2-digit",
                });
              } else {
                return date.toLocaleString(undefined, {
                  month: "short",
                  day: "numeric",
                  hour: "2-digit",
                  minute: "2-digit",
                  hour12: false,
                });
              }
            });
          },
          splits: (_u, _axisIdx, scaleMin, scaleMax) => {
            const timeRange = scaleMax - scaleMin;
            const ticks: number[] = [];

            if (timeRange <= 10 * 1000) {
              const startTime = Math.ceil(scaleMin / 1000) * 1000;
              for (let t = startTime; t <= scaleMax; t += 1000) {
                if (t >= scaleMin) ticks.push(t);
              }
            } else if (timeRange <= 30 * 1000) {
              const startTime = Math.ceil(scaleMin / 5000) * 5000;
              for (let t = startTime; t <= scaleMax; t += 5000) {
                if (t >= scaleMin) ticks.push(t);
              }
            } else if (timeRange <= 60 * 1000) {
              const startTime = Math.ceil(scaleMin / 10000) * 10000;
              for (let t = startTime; t <= scaleMax; t += 10000) {
                if (t >= scaleMin) ticks.push(t);
              }
            } else if (timeRange <= 5 * 60 * 1000) {
              const startTime = Math.ceil(scaleMin / 30000) * 30000;
              for (let t = startTime; t <= scaleMax; t += 30000) {
                if (t >= scaleMin) ticks.push(t);
              }
            } else if (timeRange <= 10 * 60 * 1000) {
              const startTime = Math.ceil(scaleMin / 60000) * 60000;
              for (let t = startTime; t <= scaleMax; t += 60000) {
                if (t >= scaleMin) ticks.push(t);
              }
            } else if (timeRange <= 30 * 60 * 1000) {
              const startTime = Math.ceil(scaleMin / 120000) * 120000;
              for (let t = startTime; t <= scaleMax; t += 120000) {
                if (t >= scaleMin) ticks.push(t);
              }
            } else if (timeRange <= 60 * 60 * 1000) {
              const startTime = Math.ceil(scaleMin / 300000) * 300000;
              for (let t = startTime; t <= scaleMax; t += 300000) {
                if (t >= scaleMin) ticks.push(t);
              }
            } else if (timeRange <= 6 * 60 * 60 * 1000) {
              const startTime = Math.ceil(scaleMin / 1800000) * 1800000;
              for (let t = startTime; t <= scaleMax; t += 1800000) {
                if (t >= scaleMin) ticks.push(t);
              }
            } else {
              const startTime = Math.ceil(scaleMin / 3600000) * 3600000;
              for (let t = startTime; t <= scaleMax; t += 3600000) {
                if (t >= scaleMin) ticks.push(t);
              }
            }

            return ticks;
          },
        },
        {
          stroke: colors.axis,
          labelSize: 12,
          labelFont: "Inter, system-ui, sans-serif",
          grid: { stroke: colors.grid, width: 1 },
          side: 1,
          space: 60,
          values: (_u, ticks): (string | number | null)[] => {
            if (renderValue) {
              const renderedValues = ticks.map(renderValue);
              const uniqueValues = new Set(renderedValues);

              if (uniqueValues.size === renderedValues.length) {
                return renderedValues as (string | number | null)[];
              }
            }

            const precision = 0;
            const maxPrecision = 4;

            for (let p = precision; p <= maxPrecision; p++) {
              const formattedValues = ticks.map((v) => v.toFixed(p));
              const uniqueFormatted = new Set(formattedValues);

              if (uniqueFormatted.size === formattedValues.length) {
                return formattedValues;
              }
            }

            return ticks.map((v) => v.toFixed(maxPrecision));
          },
        },
      ],

      series: seriesConfig,
    },
    uPlotData,
    containerRef.current,
  );

  // Create handler callbacks
  const handlerCallbacks: HandlerCallbacks = {
    updateYAxisScale,
    setViewMode,
    setIsLiveMode,
    onZoomChange: syncGraph?.onZoomChange,
    onViewModeChange: syncGraph?.onViewModeChange,
  };

  // Create and attach event handlers
  const handlers = createEventHandlers(
    containerRef,
    uplotRef,
    handlerRefs,
    handlerCallbacks,
    newData,
    config,
    graphId,
    manualScaleRef,
    width,
  );

  const cleanup = attachEventHandlers(containerRef.current, handlers);

  animationRefs.lastRenderedData.current = {
    timestamps: [...timestamps],
    values: [...primaryValues],
  };

  return cleanup;
}

// Helper function to get all valid TimeSeries from DataSeries
export function getAllTimeSeries(
  data: BigGraphProps["newData"],
): Array<{ series: TimeSeries; title?: string; color?: string }> {
  const normalized = normalizeDataSeries(data);
  return normalized
    .filter((series) => series.newData !== null)
    .map((series) => ({
      series: series.newData!,
      title: series.title,
      color: series.color,
    }));
}
