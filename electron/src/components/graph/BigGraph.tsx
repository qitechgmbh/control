import React, { useEffect, useRef, useState, useCallback } from "react";
import uPlot from "uplot";
import "uplot/dist/uPlot.min.css";
import { seriesToUPlotData } from "@/lib/timeseries";
import { renderUnitSymbol, getUnitIcon } from "@/control/units";
import { Icon } from "@/components/Icon";
import { BigGraphProps } from "./types";
import { GraphExportData } from "./excelExport";
import { POINT_ANIMATION_DURATION, DEFAULT_COLORS } from "./constants";

export function BigGraph({
  newData,
  unit,
  renderValue,
  config,
  graphId,
  syncGraph,
  onRegisterForExport,
  onUnregisterFromExport,
}: BigGraphProps & {
  onRegisterForExport?: (
    graphId: string,
    getDataFn: () => GraphExportData | null,
  ) => void;
  onUnregisterFromExport?: (graphId: string) => void;
}) {
  const containerRef = useRef<HTMLDivElement | null>(null);
  const uplotRef = useRef<uPlot | null>(null);
  const chartCreatedRef = useRef(false);

  // Initialize state from props or defaults
  const [viewMode, setViewMode] = useState<"default" | "all" | "manual">(
    syncGraph?.viewMode ?? "default",
  );
  const [isLiveMode, setIsLiveMode] = useState(syncGraph?.isLiveMode ?? true);
  const [selectedTimeWindow, setSelectedTimeWindow] = useState<number | "all">(
    syncGraph?.timeWindow ?? config.defaultTimeWindow ?? 30 * 60 * 1000,
  );

  const [cursorValue, setCursorValue] = useState<number | null>(null);
  const startTimeRef = useRef<number | null>(null);
  const manualScaleRef = useRef<{
    x: { min: number; max: number };
    y: { min: number; max: number };
  } | null>(null);
  const isUserZoomingRef = useRef(false);
  const isDraggingRef = useRef(false);
  const lastDragXRef = useRef<number | null>(null);
  const realPointsCountRef = useRef(0);
  const isPinchingRef = useRef(false);
  const lastPinchDistanceRef = useRef<number | null>(null);
  const pinchCenterRef = useRef<{ x: number; y: number } | null>(null);
  const lastProcessedCountRef = useRef(0);

  // CRITICAL FIX: Store the freeze timestamp when switching to historical mode
  const historicalFreezeTimestampRef = useRef<number | null>(null);

  // Touch direction detection
  const touchStartRef = useRef<{ x: number; y: number; time: number } | null>(
    null,
  );
  const touchDirectionRef = useRef<"horizontal" | "vertical" | "unknown">(
    "unknown",
  );

  // Point-by-point animation refs
  const animationFrameRef = useRef<number | null>(null);
  const lastRenderedDataRef = useRef<{
    timestamps: number[];
    values: number[];
  }>({ timestamps: [], values: [] });
  const animationStateRef = useRef<{
    isAnimating: boolean;
    startTime: number;
    fromValue: number;
    toValue: number;
    fromTimestamp: number;
    toTimestamp: number;
    targetIndex: number;
  }>({
    isAnimating: false,
    startTime: 0,
    fromValue: 0,
    toValue: 0,
    fromTimestamp: 0,
    toTimestamp: 0,
    targetIndex: 0,
  });

  const colors = {
    primary: config.colors?.primary ?? DEFAULT_COLORS.primary,
    grid: config.colors?.grid ?? DEFAULT_COLORS.grid,
    axis: config.colors?.axis ?? DEFAULT_COLORS.axis,
    background: config.colors?.background ?? DEFAULT_COLORS.background,
  };

  // Helper function to get current live end timestamp
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

  const captureHistoricalFreezeTimestamp = useCallback(() => {
    // Always get the current live timestamp when switching to historical
    const currentLiveEnd = getCurrentLiveEndTimestamp();
    historicalFreezeTimestampRef.current = currentLiveEnd;
    console.log(
      "🔒 Captured new freeze timestamp:",
      new Date(currentLiveEnd).toISOString(),
    );
    return currentLiveEnd;
  }, [getCurrentLiveEndTimestamp]);

  // Helper function to get historical end timestamp
  const getHistoricalEndTimestamp = useCallback((): number => {
    // If we're in historical mode and have a freeze timestamp, use it
    if (!isLiveMode && historicalFreezeTimestampRef.current !== null) {
      return historicalFreezeTimestampRef.current;
    }

    // Otherwise get current live end timestamp
    return getCurrentLiveEndTimestamp();
  }, [isLiveMode, getCurrentLiveEndTimestamp]);

  // Export registration effect
  useEffect(() => {
    if (onRegisterForExport) {
      const getExportData = (): GraphExportData | null => {
        if (!newData) return null;

        const exportData: GraphExportData = {
          config,
          data: newData,
          unit,
          renderValue,
        };

        return exportData;
      };

      onRegisterForExport(graphId, getExportData);

      return () => {
        if (onUnregisterFromExport) {
          onUnregisterFromExport(graphId);
        }
      };
    }
  }, [
    graphId,
    newData,
    config,
    unit,
    renderValue,
    onRegisterForExport,
    onUnregisterFromExport,
  ]);

  useEffect(() => {
    if (!syncGraph) return;

    let hasChanges = false;

    if (syncGraph.timeWindow !== selectedTimeWindow) {
      setSelectedTimeWindow(syncGraph.timeWindow);
      handleTimeWindowChangeInternal(syncGraph.timeWindow, true);
      hasChanges = true;
    }

    if (syncGraph.viewMode !== viewMode) {
      setViewMode(syncGraph.viewMode);
      hasChanges = true;
    }

    if (syncGraph.isLiveMode !== isLiveMode) {
      const newIsLiveMode = syncGraph.isLiveMode;

      if (isLiveMode && !newIsLiveMode) {
        // Switching live → historical: ALWAYS capture fresh timestamp
        captureHistoricalFreezeTimestamp();
        stopAnimations();
        lastProcessedCountRef.current = 0;
        manualScaleRef.current = null; // Reset manual scale
        console.log("🔄 Switched to historical mode");
      } else if (!isLiveMode && newIsLiveMode) {
        // Switching historical → live: COMPLETELY clear freeze timestamp
        historicalFreezeTimestampRef.current = null;
        manualScaleRef.current = null; // Reset manual scale
        console.log("🔄 Switched to live mode - cleared freeze timestamp");
      }

      setIsLiveMode(newIsLiveMode);
      hasChanges = true;
    }

    if (syncGraph.xRange && uplotRef.current) {
      uplotRef.current.batch(() => {
        uplotRef.current!.setScale("x", {
          min: syncGraph.xRange!.min,
          max: syncGraph.xRange!.max,
        });

        if (newData?.long) {
          const [timestamps, values] = seriesToUPlotData(newData.long);
          updateYAxisScale(
            timestamps,
            values,
            syncGraph.xRange!.min,
            syncGraph.xRange!.max,
          );
        }
      });

      manualScaleRef.current = {
        x: syncGraph.xRange,
        y: manualScaleRef.current?.y ?? { min: 0, max: 1 },
      };
      hasChanges = true;
    }

    if (hasChanges && syncGraph.viewMode === "manual") {
      setViewMode("manual");
      setIsLiveMode(false);
      stopAnimations();
      lastProcessedCountRef.current = 0;
    }
  }, [
    syncGraph?.timeWindow,
    syncGraph?.viewMode,
    syncGraph?.isLiveMode,
    syncGraph?.xRange?.min,
    syncGraph?.xRange?.max,
    selectedTimeWindow,
    viewMode,
    isLiveMode,
    captureHistoricalFreezeTimestamp,
  ]);

  const lerp = (start: number, end: number, t: number): number => {
    return start + (end - start) * t;
  };

  const animateNewPoint = (
    currentData: { timestamps: number[]; values: number[] },
    targetData: { timestamps: number[]; values: number[] },
  ) => {
    if (targetData.timestamps.length <= currentData.timestamps.length) {
      return;
    }
    if (!isLiveMode || viewMode === "manual") {
      return;
    }

    const newIndex = currentData.timestamps.length;

    const prevValue =
      currentData.values[newIndex - 1] ?? targetData.values[newIndex];
    const prevTimestamp =
      currentData.timestamps[newIndex - 1] ?? targetData.timestamps[newIndex];
    const newValue = targetData.values[newIndex];
    const newTimestamp = targetData.timestamps[newIndex];

    if (animationFrameRef.current) {
      cancelAnimationFrame(animationFrameRef.current);
      animationFrameRef.current = null;
    }

    animationStateRef.current = {
      isAnimating: true,
      startTime: performance.now(),
      fromValue: prevValue,
      toValue: newValue,
      fromTimestamp: prevTimestamp,
      toTimestamp: newTimestamp,
      targetIndex: newIndex,
    };

    const animate = (currentTime: number) => {
      if (
        !uplotRef.current ||
        !animationStateRef.current.isAnimating ||
        !isLiveMode
      )
        return;

      const elapsed = currentTime - animationStateRef.current.startTime;
      const progress = Math.min(elapsed / POINT_ANIMATION_DURATION, 1);

      const animatedTimestamps = [...currentData.timestamps];
      const animatedValues = [...currentData.values];

      if (progress < 1) {
        const interpolatedTimestamp = lerp(
          animationStateRef.current.fromTimestamp,
          animationStateRef.current.toTimestamp,
          progress,
        );
        const interpolatedValue = lerp(
          animationStateRef.current.fromValue,
          animationStateRef.current.toValue,
          progress,
        );

        animatedTimestamps.push(interpolatedTimestamp);
        animatedValues.push(interpolatedValue);
      } else {
        animatedTimestamps.push(animationStateRef.current.toTimestamp);
        animatedValues.push(animationStateRef.current.toValue);

        lastRenderedDataRef.current = {
          timestamps: [...animatedTimestamps],
          values: [...animatedValues],
        };
        realPointsCountRef.current = animatedTimestamps.length;
        animationStateRef.current.isAnimating = false;
      }

      const animatedUData = buildUPlotData(
        animatedTimestamps,
        animatedValues,
        animatedTimestamps.length,
      );
      uplotRef.current.setData(animatedUData);

      if (isLiveMode && animatedTimestamps.length > 0) {
        const latestTimestamp =
          animatedTimestamps[animatedTimestamps.length - 1];

        if (viewMode === "default") {
          let xMin: number | undefined, xMax: number | undefined;

          if (selectedTimeWindow === "all") {
            const fullStart = startTimeRef.current ?? animatedTimestamps[0];
            xMin = fullStart;
            xMax = latestTimestamp;
          } else {
            xMin = latestTimestamp - (selectedTimeWindow as number);
            xMax = latestTimestamp;
          }

          uplotRef.current.setScale("x", { min: xMin, max: xMax });
          updateYAxisScale(animatedTimestamps, animatedValues, xMin, xMax);
        } else if (viewMode === "all") {
          const fullStart = startTimeRef.current ?? animatedTimestamps[0];
          uplotRef.current.setScale("x", {
            min: fullStart,
            max: latestTimestamp,
          });
          updateYAxisScale(
            animatedTimestamps,
            animatedValues,
            fullStart,
            latestTimestamp,
          );
        }
      }

      if (progress < 1) {
        animationFrameRef.current = requestAnimationFrame(animate);
      } else {
        if (targetData.timestamps.length > animatedTimestamps.length) {
          animateNewPoint(
            { timestamps: animatedTimestamps, values: animatedValues },
            targetData,
          );
        }
      }
    };

    animationFrameRef.current = requestAnimationFrame(animate);
  };

  const stopAnimations = () => {
    if (animationFrameRef.current) {
      cancelAnimationFrame(animationFrameRef.current);
      animationFrameRef.current = null;
    }
    animationStateRef.current.isAnimating = false;
  };

  const updateYAxisScale = (
    timestamps: number[],
    values: number[],
    xMin?: number,
    xMax?: number,
  ) => {
    if (!uplotRef.current || values.length === 0) return;

    let visibleValues: number[] = [];

    if (xMin !== undefined && xMax !== undefined) {
      for (let i = 0; i < timestamps.length; i++) {
        if (timestamps[i] >= xMin && timestamps[i] <= xMax) {
          visibleValues.push(values[i]);
        }
      }
    } else {
      visibleValues = [...values];
    }

    config.lines?.forEach((line) => {
      if (line.show !== false) {
        visibleValues.push(line.value);
      }
    });

    if (visibleValues.length === 0) {
      visibleValues = values;
    }

    const minY = Math.min(...visibleValues);
    const maxY = Math.max(...visibleValues);
    const range = maxY - minY || Math.abs(maxY) * 0.1 || 1;

    const yRange = {
      min: minY - range * 0.1,
      max: maxY + range * 0.1,
    };

    uplotRef.current.batch(() => {
      uplotRef.current!.setScale("y", yRange);
    });

    if (viewMode === "manual" && manualScaleRef.current) {
      manualScaleRef.current.y = yRange;
    }
  };

  const getRightmostVisibleTimestamp = () => {
    if (!uplotRef.current || !newData?.long) return null;
    const xScale = uplotRef.current.scales.x;
    if (!xScale || xScale.max === undefined) return null;
    return xScale.max;
  };

  const buildUPlotData = (
    timestamps: number[],
    values: number[],
    realPointsCount?: number,
  ): uPlot.AlignedData => {
    const uData: uPlot.AlignedData = [timestamps, values];

    if (realPointsCount !== undefined) {
      realPointsCountRef.current = realPointsCount;
    }

    config.lines?.forEach((line) => {
      if (line.show !== false) {
        uData.push(timestamps.map(() => line.value));
      }
    });

    return uData;
  };

  const createChart = () => {
    if (!containerRef.current || !newData?.long) return;

    const [timestamps, values] = seriesToUPlotData(newData.long);
    if (timestamps.length === 0) return;

    if (startTimeRef.current === null && timestamps.length > 0) {
      startTimeRef.current = timestamps[0];
    }

    const uData = buildUPlotData(timestamps, values);

    const fullStart = startTimeRef.current ?? timestamps[0] ?? 0;

    let initialMin: number, initialMax: number;
    if (viewMode === "manual" && manualScaleRef.current) {
      initialMin = manualScaleRef.current.x.min;
      initialMax = manualScaleRef.current.x.max;
    } else if (selectedTimeWindow === "all") {
      initialMin = fullStart;
      initialMax = getHistoricalEndTimestamp();
    } else {
      // FIXED: Remove Math.max constraint for historical mode
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

    // Rest of the function remains the same...
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
            return dataIdx < realPointsCountRef.current;
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
              if (isUserZoomingRef.current) {
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
                isUserZoomingRef.current = false;
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

    // Touch handlers (keeping the same as before)
    const handleTouchStart = (e: TouchEvent) => {
      const touch = e.touches[0];
      touchStartRef.current = {
        x: touch.clientX,
        y: touch.clientY,
        time: Date.now(),
      };
      touchDirectionRef.current = "unknown";

      if (e.touches.length === 2) {
        isPinchingRef.current = true;
        isDraggingRef.current = false;
        touchDirectionRef.current = "horizontal";

        const touch1 = e.touches[0];
        const touch2 = e.touches[1];

        const distance = Math.sqrt(
          Math.pow(touch2.clientX - touch1.clientX, 2) +
            Math.pow(touch2.clientY - touch1.clientY, 2),
        );
        lastPinchDistanceRef.current = distance;

        pinchCenterRef.current = {
          x: (touch1.clientX + touch2.clientX) / 2,
          y: (touch1.clientY + touch2.clientY) / 2,
        };

        e.preventDefault();
      }
    };

    const handleTouchMove = (e: TouchEvent) => {
      if (!touchStartRef.current) return;

      if (e.touches.length === 1) {
        const touch = e.touches[0];
        const deltaX = Math.abs(touch.clientX - touchStartRef.current.x);
        const deltaY = Math.abs(touch.clientY - touchStartRef.current.y);
        const timeDelta = Date.now() - touchStartRef.current.time;

        if (
          touchDirectionRef.current === "unknown" &&
          deltaX > 20 &&
          deltaY < 10 &&
          deltaX > deltaY * 4 &&
          timeDelta < 500
        ) {
          touchDirectionRef.current = "horizontal";
          isDraggingRef.current = true;
          lastDragXRef.current = touch.clientX;
          e.preventDefault();
        } else if (touchDirectionRef.current === "unknown") {
          return;
        }

        if (
          touchDirectionRef.current === "horizontal" &&
          isDraggingRef.current
        ) {
          e.preventDefault();

          const currentX = touch.clientX;
          const dragDelta = currentX - (lastDragXRef.current || 0);
          lastDragXRef.current = currentX;

          if (uplotRef.current && Math.abs(dragDelta) > 2) {
            const xScale = uplotRef.current.scales.x;
            if (
              xScale &&
              xScale.min !== undefined &&
              xScale.max !== undefined
            ) {
              const pixelToTime = (xScale.max - xScale.min) / width;
              const timeDelta = -dragDelta * pixelToTime;

              const newMin = xScale.min + timeDelta;
              const newMax = xScale.max + timeDelta;

              uplotRef.current.setScale("x", { min: newMin, max: newMax });
              const [timestamps, values] = seriesToUPlotData(newData.long);
              updateYAxisScale(timestamps, values, newMin, newMax);

              manualScaleRef.current = {
                x: { min: newMin, max: newMax },
                y: {
                  min: uplotRef.current.scales.y?.min ?? 0,
                  max: uplotRef.current.scales.y?.max ?? 1,
                },
              };

              setViewMode("manual");
              setIsLiveMode(false);

              // Prop-based sync
              if (syncGraph?.onZoomChange) {
                syncGraph.onZoomChange(graphId, {
                  min: newMin,
                  max: newMax,
                });
              }

              if (syncGraph?.onViewModeChange) {
                syncGraph.onViewModeChange(graphId, "manual", false);
              }
            }
          }
        }
      } else if (e.touches.length === 2 && isPinchingRef.current) {
        e.preventDefault();

        const touch1 = e.touches[0];
        const touch2 = e.touches[1];

        const newDistance = Math.sqrt(
          Math.pow(touch2.clientX - touch1.clientX, 2) +
            Math.pow(touch2.clientY - touch1.clientY, 2),
        );

        if (lastPinchDistanceRef.current && uplotRef.current) {
          const scaleFactor = newDistance / lastPinchDistanceRef.current;
          const xScale = uplotRef.current.scales.x;

          if (
            xScale &&
            xScale.min !== undefined &&
            xScale.max !== undefined &&
            pinchCenterRef.current
          ) {
            const rect = containerRef.current?.getBoundingClientRect();
            if (rect) {
              const touchXRelative =
                (pinchCenterRef.current.x - rect.left) / rect.width;
              const centerTime =
                xScale.min + (xScale.max - xScale.min) * touchXRelative;

              const currentRange = xScale.max - xScale.min;
              const newRange = currentRange / scaleFactor;

              const leftRatio = (centerTime - xScale.min) / currentRange;
              const rightRatio = (xScale.max - centerTime) / currentRange;

              const newMin = centerTime - newRange * leftRatio;
              const newMax = centerTime + newRange * rightRatio;

              uplotRef.current.setScale("x", { min: newMin, max: newMax });
              const [timestamps, values] = seriesToUPlotData(newData.long);
              updateYAxisScale(timestamps, values, newMin, newMax);

              manualScaleRef.current = {
                x: { min: newMin, max: newMax },
                y: {
                  min: uplotRef.current.scales.y?.min ?? 0,
                  max: uplotRef.current.scales.y?.max ?? 1,
                },
              };
              setViewMode("manual");
              setIsLiveMode(false);

              // Prop-based sync
              if (syncGraph?.onZoomChange) {
                syncGraph.onZoomChange(graphId, {
                  min: newMin,
                  max: newMax,
                });
              }

              if (syncGraph?.onViewModeChange) {
                syncGraph.onViewModeChange(graphId, "manual", false);
              }
            }
          }
        }

        lastPinchDistanceRef.current = newDistance;
      }
    };

    const handleTouchEnd = (e: TouchEvent) => {
      if (e.touches.length === 0) {
        isDraggingRef.current = false;
        isPinchingRef.current = false;
        lastDragXRef.current = null;
        lastPinchDistanceRef.current = null;
        pinchCenterRef.current = null;
        touchStartRef.current = null;
        touchDirectionRef.current = "unknown";
      } else if (e.touches.length === 1 && isPinchingRef.current) {
        isPinchingRef.current = false;
        lastPinchDistanceRef.current = null;
        pinchCenterRef.current = null;

        const touch = e.touches[0];
        touchStartRef.current = {
          x: touch.clientX,
          y: touch.clientY,
          time: Date.now(),
        };
        touchDirectionRef.current = "unknown";
        isDraggingRef.current = false;
      }

      if (touchDirectionRef.current === "horizontal" && isDraggingRef.current) {
        e.preventDefault();
      }
    };

    const handleMouseDown = (e: MouseEvent) => {
      if (e.button === 0) {
        isUserZoomingRef.current = true;
      }
    };

    const handleWheel = (e: WheelEvent) => {
      e.preventDefault();
    };

    if (containerRef.current && uplotRef.current) {
      containerRef.current.addEventListener("touchstart", handleTouchStart, {
        passive: false,
      });
      containerRef.current.addEventListener("touchmove", handleTouchMove, {
        passive: false,
      });
      containerRef.current.addEventListener("touchend", handleTouchEnd, {
        passive: false,
      });

      containerRef.current.addEventListener("mousedown", handleMouseDown);

      containerRef.current.addEventListener("wheel", handleWheel, {
        passive: false,
      });
    }

    chartCreatedRef.current = true;

    lastRenderedDataRef.current = {
      timestamps: [...timestamps],
      values: [...values],
    };
    if (isLiveMode) {
      lastProcessedCountRef.current = timestamps.length;
    }
  };
  const handleTimeWindowChangeInternal = (
    newTimeWindow: number | "all",
    isSync: boolean = false,
  ) => {
    stopAnimations();
    setSelectedTimeWindow(newTimeWindow);

    if (!uplotRef.current) {
      return;
    }

    const [timestamps, values] = seriesToUPlotData(
      newData?.long || { timestamps: [], values: [] },
    );

    if (newTimeWindow === "all") {
      setViewMode("all");

      if (isLiveMode) {
        // Live mode: show all data up to the latest timestamp
        const fullStart =
          timestamps.length > 0
            ? timestamps[0]
            : Date.now() - 24 * 60 * 60 * 1000; // Default to last 24 hours if no data
        const fullEnd =
          timestamps.length > 0
            ? timestamps[timestamps.length - 1]
            : Date.now();
        uplotRef.current.setScale("x", { min: fullStart, max: fullEnd });
        updateYAxisScale(timestamps, values, fullStart, fullEnd);
      } else {
        // Historical mode: show all data but end at the freeze timestamp
        const endTimestamp =
          historicalFreezeTimestampRef.current ??
          captureHistoricalFreezeTimestamp(); // Ensure freeze timestamp is set
        const fullStart = endTimestamp - 24 * 60 * 60 * 1000; // Default to 24 hours before freeze timestamp

        uplotRef.current.setScale("x", { min: fullStart, max: endTimestamp });
        updateYAxisScale(timestamps, values, fullStart, endTimestamp);

        manualScaleRef.current = {
          x: { min: fullStart, max: endTimestamp },
          y: manualScaleRef.current?.y ?? {
            min: Math.min(...values),
            max: Math.max(...values),
          },
        };
      }
    } else {
      // Handle specific time window
      if (isLiveMode) {
        setViewMode("default");
        const latestTimestamp =
          timestamps.length > 0
            ? timestamps[timestamps.length - 1]
            : Date.now();
        const viewStart = latestTimestamp - newTimeWindow;

        uplotRef.current.setScale("x", {
          min: viewStart,
          max: latestTimestamp,
        });
        updateYAxisScale(timestamps, values, viewStart, latestTimestamp);
        manualScaleRef.current = null;
      } else {
        // Historical mode with specific time window
        setViewMode("default");

        const endTimestamp =
          historicalFreezeTimestampRef.current ??
          captureHistoricalFreezeTimestamp(); // Ensure freeze timestamp is set
        const newViewStart = endTimestamp - newTimeWindow;

        // Enforce the time window range, even if no data exists
        uplotRef.current.setScale("x", {
          min: newViewStart,
          max: endTimestamp,
        });

        // Update the Y-axis scale based on visible data within the range
        const visibleValues: number[] = [];
        for (let i = 0; i < timestamps.length; i++) {
          if (timestamps[i] >= newViewStart && timestamps[i] <= endTimestamp) {
            visibleValues.push(values[i]);
          }
        }

        const minY = visibleValues.length > 0 ? Math.min(...visibleValues) : 0;
        const maxY = visibleValues.length > 0 ? Math.max(...visibleValues) : 1;

        uplotRef.current.setScale("y", {
          min: minY - (maxY - minY) * 0.1,
          max: maxY + (maxY - minY) * 0.1,
        });

        manualScaleRef.current = {
          x: { min: newViewStart, max: endTimestamp },
          y: {
            min: minY - (maxY - minY) * 0.1,
            max: maxY + (maxY - minY) * 0.1,
          },
        };
      }
    }

    lastRenderedDataRef.current = { timestamps, values };
    if (isLiveMode) {
      lastProcessedCountRef.current = timestamps.length;
    }

    // Prop-based sync
    if (!isSync && syncGraph?.onTimeWindowChange) {
      syncGraph.onTimeWindowChange(graphId, newTimeWindow);
    }
  };

  // Chart creation effect
  useEffect(() => {
    if (!containerRef.current || !newData?.long) {
      chartCreatedRef.current = false;
      return;
    }

    const [timestamps] = seriesToUPlotData(newData.long);
    if (timestamps.length === 0) {
      chartCreatedRef.current = false;
      return;
    }

    createChart();

    return () => {
      if (uplotRef.current) {
        uplotRef.current.destroy();
        uplotRef.current = null;
      }
      stopAnimations();
      chartCreatedRef.current = false;
    };
  }, [newData?.long, containerRef.current]);

  // Data update effects - only run in live mode
  useEffect(() => {
    // CRITICAL: Only update data when in live mode
    if (
      !uplotRef.current ||
      !newData?.long ||
      !chartCreatedRef.current ||
      !isLiveMode ||
      viewMode === "manual"
    )
      return;

    const [timestamps, values] = seriesToUPlotData(newData.long);
    if (timestamps.length === 0) return;

    if (timestamps.length <= lastProcessedCountRef.current) {
      return;
    }

    const currentData = lastRenderedDataRef.current;
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

      animateNewPoint(currentData, limitedTargetData);
    } else if (targetData.timestamps.length === currentData.timestamps.length) {
      let hasChanges = false;
      for (let i = 0; i < targetData.values.length; i++) {
        if (Math.abs(targetData.values[i] - currentData.values[i]) > 0.001) {
          hasChanges = true;
          break;
        }
      }

      if (hasChanges) {
        const uData = buildUPlotData(timestamps, values);
        uplotRef.current.setData(uData);
        lastRenderedDataRef.current = { timestamps, values };

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
    newData?.long?.validCount,
    newData?.long?.lastTimestamp,
    viewMode,
    selectedTimeWindow,
    isLiveMode,
  ]);

  // Current data effect - only run in live mode
  useEffect(() => {
    // CRITICAL: Only update current data when in live mode
    if (
      !uplotRef.current ||
      !newData?.current ||
      !isLiveMode ||
      !chartCreatedRef.current ||
      animationStateRef.current.isAnimating ||
      viewMode === "manual"
    )
      return;

    const updateLiveData = () => {
      if (!newData?.long || !newData?.current || !uplotRef.current) return;

      const [timestamps, values] = seriesToUPlotData(newData.long);
      const cur = newData.current;
      const liveTimestamps = [...timestamps];
      const liveValues = [...values];

      liveTimestamps.push(cur.timestamp);
      liveValues.push(cur.value);

      const liveData = buildUPlotData(liveTimestamps, liveValues);
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
        startTimeRef.current = liveTimestamps[0];
      }
    };

    updateLiveData();
  }, [
    newData?.current?.timestamp,
    viewMode,
    selectedTimeWindow,
    config.lines,
    isLiveMode,
  ]);

  const displayValue =
    cursorValue !== null ? cursorValue : newData?.current?.value;

  return (
    <div className="h-[50vh] w-full">
      <div className="flex h-full w-full flex-col overflow-hidden rounded-3xl border border-gray-200 bg-white shadow">
        <div className="flex items-center justify-between pt-4 pr-5 pb-6 pl-6">
          <div className="mt-1 flex items-center gap-4">
            <Icon
              name={unit ? getUnitIcon(unit) : "lu:TrendingUp"}
              className="size-6 text-gray-600"
            />

            <h2 className="text-2xl leading-none font-bold text-gray-900">
              {config.title}
            </h2>

            <div className="flex items-center gap-2 text-base text-gray-600">
              <span className="font-mono leading-none font-bold text-gray-900">
                {displayValue !== undefined && displayValue !== null
                  ? renderValue
                    ? renderValue(displayValue)
                    : displayValue.toFixed(3)
                  : "N/A"}
              </span>
              <span className="leading-none text-gray-500">
                {renderUnitSymbol(unit)}
              </span>
            </div>
          </div>
        </div>

        <div className="-mt-2 px-6">
          <div className="h-px bg-gray-200"></div>
        </div>

        <div className="flex-1 overflow-hidden rounded-b-3xl pt-4">
          <div
            ref={containerRef}
            className="h-full w-full overflow-hidden"
            style={{ backgroundColor: colors.background }}
          />
        </div>
      </div>
    </div>
  );
}
