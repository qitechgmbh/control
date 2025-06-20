import uPlot from "uplot";
import { seriesToUPlotData } from "@/lib/timeseries";
import { buildUPlotData } from "./animation";
import {
  createEventHandlers,
  attachEventHandlers,
  HandlerRefs,
  HandlerCallbacks,
} from "./handlers";
import { BigGraphProps } from "./types";
import { AnimationRefs } from "./animation";

export interface CreateChartParams {
  containerRef: React.RefObject<HTMLDivElement | null>; // Changed this line
  uplotRef: React.RefObject<uPlot | null>;
  newData: NonNullable<BigGraphProps["newData"]>;
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
  if (!containerRef.current || !newData?.long) return;

  const [timestamps, values] = seriesToUPlotData(newData.long);
  if (timestamps.length === 0) return;

  if (startTimeRef.current === null && timestamps.length > 0) {
    startTimeRef.current = timestamps[0];
  }

  const uData = buildUPlotData(
    timestamps,
    values,
    undefined,
    animationRefs.realPointsCount,
    config,
  );

  const fullStart = startTimeRef.current ?? timestamps[0] ?? 0;

  let initialMin: number, initialMax: number;
  if (viewMode === "manual" && manualScaleRef.current) {
    initialMin = manualScaleRef.current.x.min;
    initialMax = manualScaleRef.current.x.max;
  } else if (selectedTimeWindow === "all") {
    initialMin = fullStart;
    initialMax = getHistoricalEndTimestamp();
  } else {
    // Remove Math.max constraint for historical mode
    const endTimestamp = getHistoricalEndTimestamp();

    if (isLiveMode) {
      // Live mode: constrain to available data
      const defaultViewStart = Math.max(
        endTimestamp - (selectedTimeWindow as number),
        fullStart,
      );
      initialMin = defaultViewStart;
    } else {
      // Historical mode: show full time window regardless of available data
      initialMin = endTimestamp - (selectedTimeWindow as number);
    }

    initialMax = endTimestamp;
  }

  // Calculate initial Y range
  const initialVisibleValues: number[] = [];
  for (let i = 0; i < timestamps.length; i++) {
    if (timestamps[i] >= initialMin && timestamps[i] <= initialMax) {
      initialVisibleValues.push(values[i]);
    }
  }

  config.lines?.forEach((line) => {
    if (line.show !== false) {
      initialVisibleValues.push(line.value);
    }
  });

  if (initialVisibleValues.length === 0) {
    initialVisibleValues.push(...values);
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

  const seriesConfig: uPlot.Series[] = [
    { label: "Time" },
    {
      label: "Value",
      stroke: colors.primary,
      width: 2,
      spanGaps: true,
      points: {
        show: (_u, _seriesIdx, dataIdx) => {
          return dataIdx < animationRefs.realPointsCount.current;
        },
        size: 6,
        stroke: colors.primary,
        fill: colors.primary,
        width: 2,
      },
    },
  ];

  config.lines?.forEach((line) => {
    if (line.show !== false) {
      seriesConfig.push({
        label: line.label,
        stroke: line.color,
        width: line.width ?? 1,
        dash: line.dash ?? (line.type === "threshold" ? [5, 5] : undefined),
        show: true,
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
        show: false,
      },
      hooks: {
        setScale: [
          (u) => {
            if (handlerRefs.isUserZoomingRef.current) {
              const xScale = u.scales.x;
              if (xScale.min !== undefined && xScale.max !== undefined) {
                const [timestamps, values] = seriesToUPlotData(newData.long);
                updateYAxisScale(timestamps, values, xScale.min, xScale.max);

                manualScaleRef.current = {
                  x: { min: xScale.min ?? 0, max: xScale.max ?? 0 },
                  y: {
                    min: u.scales.y?.min ?? 0,
                    max: u.scales.y?.max ?? 1,
                  },
                };

                setViewMode("manual");
                setIsLiveMode(false);

                // Prop-based sync
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
              const value = u.data[1][u.cursor.idx];
              const cur = newData?.current;

              const isNearCurrent =
                cur &&
                timestamp !== undefined &&
                Math.abs(timestamp - cur.timestamp) < 1000;

              const displayValue = isNearCurrent ? cur.value : value;
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
                  // undefined uses system locale
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
                // For very short ranges, show seconds
                return date.toLocaleTimeString(undefined, {
                  hour12: false,
                  hour: "2-digit",
                  minute: "2-digit",
                  second: "2-digit",
                });
              } else if (timeRange <= 5 * 60 * 1000) {
                // For 5 minute ranges, show seconds
                return date.toLocaleTimeString(undefined, {
                  hour12: false,
                  hour: "2-digit",
                  minute: "2-digit",
                  second: "2-digit",
                });
              } else if (timeRange <= 60 * 60 * 1000) {
                // For hour ranges, show minutes
                return date.toLocaleTimeString(undefined, {
                  hour12: false,
                  hour: "2-digit",
                  minute: "2-digit",
                });
              } else if (timeRange <= 24 * 60 * 60 * 1000) {
                // For day ranges, show hours and minutes
                return date.toLocaleTimeString(undefined, {
                  hour12: false,
                  hour: "2-digit",
                  minute: "2-digit",
                });
              } else {
                // For longer ranges, show date and time
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
          splits: (
            _u,
            _axisIdx,
            scaleMin,
            scaleMax,
            _foundIncr,
            _foundSpace,
          ) => {
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
    uData,
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
    values: [...values],
  };

  // Return cleanup function
  return cleanup;
}
