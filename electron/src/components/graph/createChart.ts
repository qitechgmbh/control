import uPlot from "uplot";
import { seriesToUPlotData, TimeSeries } from "@/lib/timeseries";
import {
  createEventHandlers,
  attachEventHandlers,
  HandlerRefs,
  HandlerCallbacks,
} from "./handlers";
import { BigGraphProps, SeriesData } from "./types";
import { AnimationRefs } from "./animation";

export interface CreateChartParams {
  containerRef: React.RefObject<HTMLDivElement | null>;
  uplotRef: React.RefObject<uPlot | null>;
  newData: BigGraphProps["newData"];
  config: BigGraphProps["config"];
  colors: {
    primary: string;
    grid: string;
    axis: string;
    background: string;
  };
  renderValue?: (value: number) => string;
  viewMode: "default" | "all" | "manual";
  selectedTimeWindow: number | "all";
  isLiveMode: boolean;
  startTimeRef: React.RefObject<number | null>;
  manualScaleRef: React.RefObject<{
    x: { min: number; max: number };
    y: { min: number; max: number };
  } | null>;
  animationRefs: AnimationRefs;
  handlerRefs: HandlerRefs;
  graphId: string;
  syncGraph?: BigGraphProps["syncGraph"];
  getHistoricalEndTimestamp: () => number;
  updateYAxisScale: (
    timestamps: number[],
    values: number[],
    xMin?: number,
    xMax?: number,
  ) => void;
  setViewMode: React.Dispatch<
    React.SetStateAction<"default" | "all" | "manual">
  >;
  setIsLiveMode: React.Dispatch<React.SetStateAction<boolean>>;
  setCursorValue: React.Dispatch<React.SetStateAction<number | null>>;
}

// Helper function to normalize data to array format
function normalizeDataSeries(data: BigGraphProps["newData"]): SeriesData[] {
  if (Array.isArray(data)) {
    return data;
  }
  return [data];
}

// Helper function to get all valid TimeSeries from DataSeries
function getAllTimeSeries(
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

// Helper function to create unified timestamp array and align all series data
function alignSeriesData(
  allSeries: Array<{ series: TimeSeries; title?: string; color?: string }>,
  renderValue?: (value: number) => string,
): {
  timestamps: number[];
  alignedValues: number[][];
} {
  if (allSeries.length === 0) {
    return { timestamps: [], alignedValues: [] };
  }

  // Get all unique timestamps from all series
  const allTimestamps = new Set<number>();
  const seriesData: Array<{ timestamps: number[]; values: number[] }> = [];

  allSeries.forEach(({ series }) => {
    const [timestamps, values] = seriesToUPlotData(series.long);

    // Apply renderValue formatting if provided
    const processedValues = renderValue
      ? values.map((v) => parseFloat(renderValue(v)))
      : values;

    seriesData.push({ timestamps, values: processedValues });
    timestamps.forEach((ts) => allTimestamps.add(ts));
  });

  // Sort timestamps
  const sortedTimestamps = Array.from(allTimestamps).sort((a, b) => a - b);

  // Align all series to the unified timestamp array
  const alignedValues: number[][] = [];

  seriesData.forEach(({ timestamps, values }) => {
    const alignedSeries: number[] = [];

    for (const targetTimestamp of sortedTimestamps) {
      const index = timestamps.indexOf(targetTimestamp);
      if (index !== -1) {
        alignedSeries.push(values[index]);
      } else {
        // Use null for missing data points - uPlot will handle gaps
        alignedSeries.push(null as any);
      }
    }

    alignedValues.push(alignedSeries);
  });

  return {
    timestamps: sortedTimestamps,
    alignedValues,
  };
}

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
}: CreateChartParams): (() => void) | undefined {
  if (!containerRef.current) return;

  const allSeries = getAllTimeSeries(newData);
  if (allSeries.length === 0) return;

  // Align all series data to unified timestamps
  const { timestamps, alignedValues } = alignSeriesData(allSeries, renderValue);
  if (timestamps.length === 0) return;

  if (startTimeRef.current === null && timestamps.length > 0) {
    startTimeRef.current = timestamps[0];
  }

  // Build uPlot data with aligned series
  const uPlotData: uPlot.AlignedData = [timestamps as any];
  alignedValues.forEach((values) => {
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
    initialMin = fullStart;
    initialMax = getHistoricalEndTimestamp();
  } else {
    const endTimestamp = getHistoricalEndTimestamp();

    if (isLiveMode) {
      const defaultViewStart = Math.max(
        endTimestamp - (selectedTimeWindow as number),
        fullStart,
      );
      initialMin = defaultViewStart;
    } else {
      initialMin = endTimestamp - (selectedTimeWindow as number);
    }

    initialMax = endTimestamp;
  }

  // Calculate initial Y range from all aligned series
  const initialVisibleValues: number[] = [];

  for (let i = 0; i < timestamps.length; i++) {
    if (timestamps[i] >= initialMin && timestamps[i] <= initialMax) {
      alignedValues.forEach((seriesValues) => {
        if (seriesValues[i] !== null && seriesValues[i] !== undefined) {
          initialVisibleValues.push(seriesValues[i]);
        }
      });
    }
  }

  config.lines?.forEach((line) => {
    if (line.show !== false) {
      initialVisibleValues.push(line.value);
    }
  });

  if (initialVisibleValues.length === 0) {
    alignedValues.forEach((seriesValues) => {
      seriesValues.forEach((value) => {
        if (value !== null && value !== undefined) {
          initialVisibleValues.push(value);
        }
      });
    });
    config.lines?.forEach((line) => {
      if (line.show !== false) {
        initialVisibleValues.push(line.value);
      }
    });
  }

  let initialYMin: number, initialYMax: number;
  if (initialVisibleValues.length > 0) {
    const minY = Math.min(...initialVisibleValues);
    const maxY = Math.max(...initialVisibleValues);
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

  // Build series configuration
  const seriesConfig: uPlot.Series[] = [{ label: "Time" }];

  // Add data series
  const defaultColors = ["#3b82f6", "#ef4444", "#10b981", "#f59e0b", "#8b5cf6"];
  allSeries.forEach(({ title, color }, index) => {
    seriesConfig.push({
      label: title || `Series ${index + 1}`,
      stroke: color || defaultColors[index % defaultColors.length],
      width: 2,
      spanGaps: true,
      show: true,
      points: {
        show: (_u, _seriesIdx, dataIdx) => {
          return dataIdx < animationRefs.realPointsCount.current;
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

  if (uplotRef.current) {
    uplotRef.current.destroy();
    uplotRef.current = null;
  }

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
        show: config.showLegend ?? allSeries.length > 1,
      },
      hooks: {
        setScale: [
          (u) => {
            if (handlerRefs.isUserZoomingRef.current) {
              const xScale = u.scales.x;
              if (xScale.min !== undefined && xScale.max !== undefined) {
                // Update Y axis based on all visible series using aligned data
                const allVisibleValues: number[] = [];

                for (let i = 0; i < timestamps.length; i++) {
                  if (
                    timestamps[i] >= xScale.min! &&
                    timestamps[i] <= xScale.max!
                  ) {
                    alignedValues.forEach((seriesValues) => {
                      if (
                        seriesValues[i] !== null &&
                        seriesValues[i] !== undefined
                      ) {
                        allVisibleValues.push(seriesValues[i]);
                      }
                    });
                  }
                }

                if (allVisibleValues.length > 0) {
                  const minY = Math.min(...allVisibleValues);
                  const maxY = Math.max(...allVisibleValues);
                  const range = maxY - minY || Math.abs(maxY) * 0.1 || 1;

                  u.setScale("y", {
                    min: minY - range * 0.1,
                    max: maxY + range * 0.1,
                  });
                }

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
            if (
              typeof u.cursor.idx === "number" &&
              u.data[1] &&
              u.data[1][u.cursor.idx] !== undefined
            ) {
              const timestamp = u.data[0][u.cursor.idx];
              const value = u.data[1][u.cursor.idx]; // Use first series for cursor value
              const cur = allSeries[0]?.series?.current;

              const isNearCurrent =
                cur &&
                timestamp !== undefined &&
                Math.abs(timestamp - cur.timestamp) < 1000;

              // Apply renderValue to current value if using it
              const processedCurrentValue =
                cur && renderValue
                  ? parseFloat(renderValue(cur.value))
                  : cur?.value;

              const displayValue = isNearCurrent
                ? processedCurrentValue
                : value;
              setCursorValue(displayValue ?? null);
            } else {
              setCursorValue(null);
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
          values: (_u, ticks) => {
            if (renderValue) {
              const renderedValues = ticks.map(renderValue);
              const uniqueValues = new Set(renderedValues);

              if (uniqueValues.size === renderedValues.length) {
                return renderedValues;
              }
            }

            const precision = 0;
            const maxPrecision = 2;

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
    values: alignedValues.length > 0 ? [...alignedValues[0]] : [],
  };

  return cleanup;
}
