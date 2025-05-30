import React, { useEffect, useRef, useState } from "react";
import uPlot from "uplot";
import "uplot/dist/uPlot.min.css";
import {
  TimeSeries,
  seriesToUPlotData,
  getSeriesMinMax,
} from "@/lib/timeseries";
import { renderUnitSymbol, Unit, getUnitIcon } from "@/control/units";
import { TouchButton } from "@/components/touch/TouchButton";
import { Icon, IconName } from "@/components/Icon";
import * as XLSX from "xlsx";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";

// Configuration types for additional lines
export type GraphLine = {
  type: "threshold" | "target" | "reference";
  value: number;
  label: string;
  color: string;
  width?: number;
  dash?: number[];
  show?: boolean;
};

export type GraphConfig = {
  title: string;
  description?: string;
  icon?: IconName;
  lines?: GraphLine[];
  timeWindows?: Array<{ value: number | "all"; label: string }>;
  defaultTimeWindow?: number | "all";
  exportFilename?: string;
  showLegend?: boolean;
  colors?: {
    primary?: string;
    grid?: string;
    axis?: string;
    background?: string;
  };
};

type BigGraphProps = {
  newData: TimeSeries | null;
  unit?: Unit;
  renderValue?: (value: number) => string;
  config: GraphConfig;
};

// Default time window options with "Show All" included
const DEFAULT_TIME_WINDOW_OPTIONS = [
  { value: 10 * 1000, label: "10s" },
  { value: 30 * 1000, label: "30s" },
  { value: 1 * 60 * 1000, label: "1m" },
  { value: 5 * 60 * 1000, label: "5m" },
  { value: 10 * 60 * 1000, label: "10m" },
  { value: 30 * 60 * 1000, label: "30m" },
  { value: 1 * 60 * 60 * 1000, label: "1h" },
  { value: "all" as const, label: "Show All" },
];

export function BigGraph({
  newData,
  unit,
  renderValue,
  config,
}: BigGraphProps) {
  const containerRef = useRef<HTMLDivElement | null>(null);
  const uplotRef = useRef<uPlot | null>(null);
  const chartCreatedRef = useRef(false);
  const isScrollingRef = useRef(false);
  const scrollTimeoutRef = useRef<NodeJS.Timeout | null>(null);

  const [viewMode, setViewMode] = useState<"default" | "all" | "manual">(
    "default",
  );
  const [isLiveMode, setIsLiveMode] = useState(true);
  const [selectedTimeWindow, setSelectedTimeWindow] = useState<number | "all">(
    config.defaultTimeWindow ?? 1 * 60 * 1000,
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
  // Add this new ref to track which points are real vs animated
  const realPointsCountRef = useRef(0);
  // Touch/pinch refs
  const isPinchingRef = useRef(false);
  const lastPinchDistanceRef = useRef<number | null>(null);
  const pinchCenterRef = useRef<{ x: number; y: number } | null>(null);

  //  Point-by-point animation refs**
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

  // **ANIMATION CONSTANTS**
  const POINT_ANIMATION_DURATION = 1000; // 1 second per point animation
  const TARGET_FPS = 60; // Smooth 60fps updates

  const UPDATE_INTERVAL_MS = newData?.long?.sampleInterval ?? 100;
  const timeWindowOptions = config.timeWindows ?? DEFAULT_TIME_WINDOW_OPTIONS;
  const colors = {
    primary: config.colors?.primary ?? "#3b82f6",
    grid: config.colors?.grid ?? "#e2e8f0",
    axis: config.colors?.axis ?? "#64748b",
    background: config.colors?.background ?? "#ffffff",
  };

  // **LINEAR INTERPOLATION FUNCTION**
  const lerp = (start: number, end: number, t: number): number => {
    return start + (end - start) * t;
  };

  // **ANIMATE NEW POINT ADDITION**
  const animateNewPoint = (
    currentData: { timestamps: number[]; values: number[] },
    targetData: { timestamps: number[]; values: number[] },
  ) => {
    // Check if we have a new point to animate
    if (targetData.timestamps.length <= currentData.timestamps.length) {
      return; // No new points
    }

    // Get the new point details
    const newIndex = currentData.timestamps.length;
    const prevValue =
      currentData.values[newIndex - 1] ?? targetData.values[newIndex];
    const prevTimestamp =
      currentData.timestamps[newIndex - 1] ?? targetData.timestamps[newIndex];
    const newValue = targetData.values[newIndex];
    const newTimestamp = targetData.timestamps[newIndex];

    // Cancel any existing animation
    if (animationFrameRef.current) {
      cancelAnimationFrame(animationFrameRef.current);
      animationFrameRef.current = null;
    }

    // Set up animation state
    animationStateRef.current = {
      isAnimating: true,
      startTime: performance.now(),
      fromValue: prevValue,
      toValue: newValue,
      fromTimestamp: prevTimestamp,
      toTimestamp: newTimestamp,
      targetIndex: newIndex,
    };

    // Start animation loop
    const animate = (currentTime: number) => {
      if (!uplotRef.current || !animationStateRef.current.isAnimating) return;

      const elapsed = currentTime - animationStateRef.current.startTime;
      const progress = Math.min(elapsed / POINT_ANIMATION_DURATION, 1);

      // Create interpolated data
      const animatedTimestamps = [...currentData.timestamps];
      const animatedValues = [...currentData.values];

      if (progress < 1) {
        // Interpolate both timestamp AND value
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

        // Add the interpolated point
        animatedTimestamps.push(interpolatedTimestamp);
        animatedValues.push(interpolatedValue);
      } else {
        // Animation complete - add all remaining points up to the target
        for (
          let i = animationStateRef.current.targetIndex;
          i < targetData.timestamps.length;
          i++
        ) {
          animatedTimestamps.push(targetData.timestamps[i]);
          animatedValues.push(targetData.values[i]);
        }

        // Update the last rendered data and real points count
        lastRenderedDataRef.current = {
          timestamps: [...targetData.timestamps],
          values: [...targetData.values],
        };
        realPointsCountRef.current = targetData.timestamps.length;

        animationStateRef.current.isAnimating = false;
      }

      // **KEY CHANGE: Pass the real points count to buildUPlotData**
      const animatedUData = buildUPlotData(
        animatedTimestamps,
        animatedValues,
        currentData.timestamps.length, // Only show points for the original data
      );
      uplotRef.current.setData(animatedUData);

      // Update scales if in live mode
      if (isLiveMode && animatedTimestamps.length > 0) {
        const latestTimestamp =
          animatedTimestamps[animatedTimestamps.length - 1];

        if (viewMode === "default") {
          let xMin, xMax;

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
        // Continue animation
        animationFrameRef.current = requestAnimationFrame(animate);
      } else {
        // Check if there are more points to animate
        if (targetData.timestamps.length > animatedTimestamps.length) {
          // Start next animation immediately
          animateNewPoint(
            { timestamps: animatedTimestamps, values: animatedValues },
            targetData,
          );
        }
      }
    };

    animationFrameRef.current = requestAnimationFrame(animate);
  };

  // **STOP ALL ANIMATIONS**
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

    let yRange;

    if (xMin !== undefined && xMax !== undefined) {
      // Scale based on visible time window
      yRange = getVisibleDataRange(timestamps, values, xMin, xMax);
    }

    if (!yRange) {
      // Fallback to full data range
      const minY = Math.min(...values);
      const maxY = Math.max(...values);
      const range = maxY - minY || 1;

      // Include line values for proper scaling
      const allValues = [...values];
      config.lines?.forEach((line) => {
        if (line.show !== false) {
          allValues.push(line.value);
        }
      });

      const finalMinY = Math.min(...allValues);
      const finalMaxY = Math.max(...allValues);
      const finalRange = finalMaxY - finalMinY || 1;

      yRange = {
        min: finalMinY - finalRange * 0.1,
        max: finalMaxY + finalRange * 0.1,
      };
    }

    uplotRef.current.setScale("y", yRange);
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

    // Store the real points count for point visibility logic
    if (realPointsCount !== undefined) {
      realPointsCountRef.current = realPointsCount;
    }

    // Add line data for each configured line
    config.lines?.forEach((line) => {
      if (line.show !== false) {
        uData.push(timestamps.map(() => line.value));
      }
    });

    return uData;
  };

  // Helper function for visible data range
  const getVisibleDataRange = (
    timestamps: number[],
    values: number[],
    xMin: number,
    xMax: number,
  ) => {
    const visibleValues: number[] = [];

    for (let i = 0; i < timestamps.length; i++) {
      if (timestamps[i] >= xMin && timestamps[i] <= xMax) {
        visibleValues.push(values[i]);
      }
    }

    // Include line values in the visible range for proper scaling
    config.lines?.forEach((line) => {
      if (line.show !== false) {
        visibleValues.push(line.value);
      }
    });

    if (visibleValues.length === 0) {
      return null;
    }

    const minY = Math.min(...visibleValues);
    const maxY = Math.max(...visibleValues);
    const range = maxY - minY || 1;

    return {
      min: minY - range * 0.1,
      max: maxY + range * 0.1,
    };
  };

  const createChart = () => {
    if (!containerRef.current || !newData?.long) return;

    const [timestamps, values] = seriesToUPlotData(newData.long);
    if (timestamps.length === 0) return;

    // Set start time reference
    if (startTimeRef.current === null && timestamps.length > 0) {
      startTimeRef.current = timestamps[0];
    }

    const { min: minY, max: maxY } = getSeriesMinMax(newData.long);
    const range = maxY - minY || 1;

    // Build uPlot data structure
    const uData = buildUPlotData(timestamps, values);

    // Calculate initial view
    const lastTimestamp = timestamps[timestamps.length - 1] ?? 0;
    const fullStart = startTimeRef.current ?? timestamps[0] ?? 0;

    let initialMin, initialMax;
    if (viewMode === "manual" && manualScaleRef.current) {
      initialMin = manualScaleRef.current.x.min;
      initialMax = manualScaleRef.current.x.max;
    } else if (selectedTimeWindow === "all") {
      initialMin = fullStart;
      initialMax = lastTimestamp;
    } else {
      const defaultViewStart = Math.max(
        lastTimestamp - (selectedTimeWindow as number),
        fullStart,
      );
      initialMin = defaultViewStart;
      initialMax = lastTimestamp;
    }

    const rect = containerRef.current.getBoundingClientRect();
    const width = rect.width;
    const height = Math.min(rect.height, window.innerHeight * 0.5);

    // Build series configuration
    const seriesConfig: uPlot.Series[] = [
      { label: "Time" },
      {
        label: "Value",
        stroke: colors.primary,
        width: 2,
        spanGaps: true,
        points: {
          show: (u, seriesIdx, dataIdx) => {
            return dataIdx < realPointsCountRef.current;
          },
          size: 6,
          stroke: colors.primary,
          fill: colors.primary, // Changed from "#ffffff" to match the line color
          width: 2,
        },
      },
    ];

    // Add line series
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

    // Destroy existing chart if it exists
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

                  // Use the enhanced scaling function
                  const yRange = getVisibleDataRange(
                    timestamps,
                    values,
                    xScale.min,
                    xScale.max,
                  );

                  if (yRange) {
                    u.setScale("y", yRange);

                    manualScaleRef.current = {
                      x: { min: xScale.min, max: xScale.max },
                      y: { min: yRange.min, max: yRange.max },
                    };
                  }

                  setViewMode("manual");
                  setIsLiveMode(false);
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
            min: initialMin,
            max: initialMax,
          },
          y: {
            auto: false,
            min: minY - range * 0.1,
            max: maxY + range * 0.1,
          },
        },
        axes: [
          {
            stroke: colors.axis,
            labelSize: 14,
            labelFont: "Inter, system-ui, sans-serif",
            grid: { stroke: colors.grid, width: 1 },
            values: (u, ticks) =>
              ticks.map((ts) =>
                new Date(ts).toLocaleTimeString("en-GB", {
                  hour12: false,
                  hour: "2-digit",
                  minute: "2-digit",
                  second: "2-digit",
                }),
              ),
          },
          {
            stroke: colors.axis,
            labelSize: 14,
            labelFont: "Inter, system-ui, sans-serif",
            grid: { stroke: colors.grid, width: 1 },
            side: 1,
            values: (u, ticks) =>
              ticks.map((v) => (renderValue ? renderValue(v) : v.toFixed(3))),
          },
        ],
        series: seriesConfig,
      },
      uData,
      containerRef.current,
    );

    const handleTouchStart = (e: TouchEvent) => {
      if (e.touches.length === 1) {
        // Single finger - dragging
        isDraggingRef.current = true;
        lastDragXRef.current = e.touches[0].clientX;
        e.preventDefault();
      } else if (e.touches.length === 2) {
        // Two fingers - pinch to zoom
        isPinchingRef.current = true;
        isDraggingRef.current = false;

        const touch1 = e.touches[0];
        const touch2 = e.touches[1];

        // Calculate distance between fingers
        const distance = Math.sqrt(
          Math.pow(touch2.clientX - touch1.clientX, 2) +
            Math.pow(touch2.clientY - touch1.clientY, 2),
        );
        lastPinchDistanceRef.current = distance;

        // Calculate center point between fingers
        pinchCenterRef.current = {
          x: (touch1.clientX + touch2.clientX) / 2,
          y: (touch1.clientY + touch2.clientY) / 2,
        };

        e.preventDefault();
      }
    };

    const handleTouchMove = (e: TouchEvent) => {
      if (e.touches.length === 1 && isDraggingRef.current) {
        // Single finger dragging - continue regardless of pinch state
        const currentX = e.touches[0].clientX;
        const deltaX = currentX - (lastDragXRef.current || 0);
        lastDragXRef.current = currentX;

        // In your touch move handler, when updating the scale:
        if (uplotRef.current && Math.abs(deltaX) > 1) {
          const xScale = uplotRef.current.scales.x;
          if (xScale && xScale.min !== undefined && xScale.max !== undefined) {
            const pixelToTime = (xScale.max - xScale.min) / width;
            const timeDelta = -deltaX * pixelToTime;

            const newMin = xScale.min + timeDelta;
            const newMax = xScale.max + timeDelta;

            uplotRef.current.setScale("x", { min: newMin, max: newMax });

            // Update Y-axis based on new visible window
            const [timestamps, values] = seriesToUPlotData(newData.long);
            const yRange = getVisibleDataRange(
              timestamps,
              values,
              newMin,
              newMax,
            );

            if (yRange) {
              uplotRef.current.setScale("y", yRange);
              manualScaleRef.current = {
                x: { min: newMin, max: newMax },
                y: { min: yRange.min, max: yRange.max },
              };
            }

            setViewMode("manual");
            setIsLiveMode(false);
          }
        }

        e.preventDefault();
      } else if (e.touches.length === 2 && isPinchingRef.current) {
        // Two finger pinch zoom
        const touch1 = e.touches[0];
        const touch2 = e.touches[1];

        // Calculate new distance
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
            // Get the container bounds to convert touch coordinates to chart coordinates
            const rect = containerRef.current?.getBoundingClientRect();
            if (rect) {
              // Convert touch center to chart time coordinate
              const touchXRelative =
                (pinchCenterRef.current.x - rect.left) / rect.width;
              const centerTime =
                xScale.min + (xScale.max - xScale.min) * touchXRelative;

              // Calculate new time range
              const currentRange = xScale.max - xScale.min;
              const newRange = currentRange / scaleFactor;

              // Keep the zoom centered on the pinch point
              const leftRatio = (centerTime - xScale.min) / currentRange;
              const rightRatio = (xScale.max - centerTime) / currentRange;

              const newMin = centerTime - newRange * leftRatio;
              const newMax = centerTime + newRange * rightRatio;

              uplotRef.current.setScale("x", {
                min: newMin,
                max: newMax,
              });

              // Update manual scale reference
              const currentYScale = uplotRef.current.scales.y;
              manualScaleRef.current = {
                x: { min: newMin, max: newMax },
                y: {
                  min: currentYScale?.min ?? minY - range * 0.1,
                  max: currentYScale?.max ?? maxY + range * 0.1,
                },
              };
              setViewMode("manual");
              setIsLiveMode(false);
            }
          }
        }

        lastPinchDistanceRef.current = newDistance;
        e.preventDefault();
      }
    };

    const handleTouchEnd = (e: TouchEvent) => {
      if (e.touches.length === 0) {
        // All fingers lifted - stop all interactions
        isDraggingRef.current = false;
        isPinchingRef.current = false;
        lastDragXRef.current = null;
        lastPinchDistanceRef.current = null;
        pinchCenterRef.current = null;
      } else if (e.touches.length === 1) {
        // Went from multiple fingers to single finger
        if (isPinchingRef.current) {
          // Was pinching, now switch to dragging
          isPinchingRef.current = false;
          lastPinchDistanceRef.current = null;
          pinchCenterRef.current = null;

          // Start fresh drag with the remaining finger
          isDraggingRef.current = true;
          lastDragXRef.current = e.touches[0].clientX;
        }
        // If was already dragging, continue dragging (don't reset)
      }
      e.preventDefault();
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
      // Touch events for mobile/tablet
      containerRef.current.addEventListener("touchstart", handleTouchStart, {
        passive: false,
      });
      containerRef.current.addEventListener("touchmove", handleTouchMove, {
        passive: false,
      });
      containerRef.current.addEventListener("touchend", handleTouchEnd, {
        passive: false,
      });

      // Mouse events
      containerRef.current.addEventListener("mousedown", handleMouseDown);
      containerRef.current.addEventListener("wheel", handleWheel, {
        passive: false,
      });
    }

    chartCreatedRef.current = true;

    // **INITIALIZE LAST RENDERED DATA**
    lastRenderedDataRef.current = {
      timestamps: [...timestamps],
      values: [...values],
    };
  };

  const switchToLiveMode = () => {
    console.log(
      "Switching to live mode with selectedTimeWindow:",
      selectedTimeWindow,
    );

    setIsLiveMode(true);

    // Stop any ongoing animations
    stopAnimations();

    // Store the current time window before any state changes
    const currentTimeWindow = selectedTimeWindow;

    // Force set the view mode based on time window
    if (currentTimeWindow === "all") {
      setViewMode("all");
    } else {
      setViewMode("default");
    }

    if (uplotRef.current && newData?.long) {
      const [timestamps, values] = seriesToUPlotData(newData.long);
      if (timestamps.length > 0) {
        const latestTimestamp = timestamps[timestamps.length - 1];

        // Apply the selected time window immediately
        if (currentTimeWindow === "all") {
          const fullStart = startTimeRef.current ?? timestamps[0];
          uplotRef.current.setScale("x", {
            min: fullStart,
            max: latestTimestamp,
          });
          updateYAxisScale(timestamps, values, fullStart, latestTimestamp);
          manualScaleRef.current = null;
        } else {
          const viewStart = latestTimestamp - (currentTimeWindow as number);
          console.log("Setting live mode scale:", {
            currentTimeWindow,
            latestTimestamp,
            viewStart,
            calculatedWindow: latestTimestamp - viewStart,
          });
          uplotRef.current.setScale("x", {
            min: viewStart,
            max: latestTimestamp,
          });
          updateYAxisScale(timestamps, values, viewStart, latestTimestamp);
          manualScaleRef.current = null;
        }

        // Show all data immediately in live mode
        const fullData = buildUPlotData(timestamps, values);
        uplotRef.current.setData(fullData);

        // Update last rendered data
        lastRenderedDataRef.current = { timestamps, values };
      }
    }

    console.log(
      "Live mode switch complete. Applied timeWindow:",
      currentTimeWindow,
      "viewMode should be:",
      currentTimeWindow === "all" ? "all" : "default",
    );
  };

  const switchToHistoricalMode = () => {
    setIsLiveMode(false);
    setViewMode("manual");

    // Stop any ongoing animations
    stopAnimations();

    if (uplotRef.current && uplotRef.current.scales) {
      const xScale = uplotRef.current.scales.x;
      const yScale = uplotRef.current.scales.y;
      if (
        xScale &&
        yScale &&
        xScale.min !== undefined &&
        xScale.max !== undefined &&
        yScale.min !== undefined &&
        yScale.max !== undefined
      ) {
        manualScaleRef.current = {
          x: { min: xScale.min, max: xScale.max },
          y: { min: yScale.min, max: yScale.max },
        };
      }
    }

    // Show all data immediately in historical mode with current time window
    if (uplotRef.current && newData?.long) {
      const [timestamps, values] = seriesToUPlotData(newData.long);
      const fullData = buildUPlotData(timestamps, values);
      uplotRef.current.setData(fullData);

      // Apply the selected time window immediately
      if (timestamps.length > 0) {
        const latestTimestamp = timestamps[timestamps.length - 1];

        if (selectedTimeWindow === "all") {
          const fullStart = startTimeRef.current ?? timestamps[0];
          uplotRef.current.setScale("x", {
            min: fullStart,
            max: latestTimestamp,
          });
          updateYAxisScale(timestamps, values, fullStart, latestTimestamp);
        } else {
          const viewStart = latestTimestamp - (selectedTimeWindow as number);
          uplotRef.current.setScale("x", {
            min: viewStart,
            max: latestTimestamp,
          });
          updateYAxisScale(timestamps, values, viewStart, latestTimestamp);

          // Update manual scale reference with the time window
          manualScaleRef.current = {
            x: { min: viewStart, max: latestTimestamp },
            y: manualScaleRef.current?.y ?? {
              min: Math.min(...values),
              max: Math.max(...values),
            },
          };
        }
      }

      // Update last rendered data
      lastRenderedDataRef.current = { timestamps, values };
    }
  };

  // Simple scroll detection
  useEffect(() => {
    const handleScroll = () => {
      isScrollingRef.current = true;

      if (scrollTimeoutRef.current) {
        clearTimeout(scrollTimeoutRef.current);
      }

      scrollTimeoutRef.current = setTimeout(() => {
        isScrollingRef.current = false;
      }, 100);
    };

    window.addEventListener("scroll", handleScroll, { passive: true });

    return () => {
      window.removeEventListener("scroll", handleScroll);
      if (scrollTimeoutRef.current) {
        clearTimeout(scrollTimeoutRef.current);
      }
    };
  }, []);

  // Chart creation effect - only create when we have data and container
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

    // Always recreate the chart to ensure proper timestamp ordering
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
  // **NEW: Data updates effect with point-by-point animation**
  useEffect(() => {
    if (
      !uplotRef.current ||
      !newData?.long ||
      isScrollingRef.current ||
      !chartCreatedRef.current ||
      !isLiveMode // Only animate in live mode
    )
      return;

    const [timestamps, values] = seriesToUPlotData(newData.long);
    if (timestamps.length === 0) return;

    const currentData = lastRenderedDataRef.current;
    const targetData = { timestamps, values };

    // Check if we have new data points to animate
    if (targetData.timestamps.length > currentData.timestamps.length) {
      // We have new points - start animation
      animateNewPoint(currentData, targetData);
    } else if (targetData.timestamps.length === currentData.timestamps.length) {
      // Same number of points - check if any values changed
      let hasChanges = false;
      for (let i = 0; i < targetData.values.length; i++) {
        if (Math.abs(targetData.values[i] - currentData.values[i]) > 0.001) {
          hasChanges = true;
          break;
        }
      }

      if (hasChanges) {
        // Values changed - update immediately without animation
        const uData = buildUPlotData(timestamps, values);
        uplotRef.current.setData(uData);
        lastRenderedDataRef.current = { timestamps, values };

        // Update scales
        const lastTimestamp = timestamps[timestamps.length - 1] ?? 0;
        if (viewMode === "default") {
          let xMin, xMax;

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

  // **MODIFIED: Live updates effect for current value (no animation)**
  useEffect(() => {
    if (
      !uplotRef.current ||
      !newData?.current ||
      isScrollingRef.current ||
      !isLiveMode ||
      !chartCreatedRef.current ||
      animationStateRef.current.isAnimating // Don't interfere with point animations
    )
      return;

    const updateLiveData = () => {
      if (!newData?.long || !newData?.current || !uplotRef.current) return;

      const [timestamps, values] = seriesToUPlotData(newData.long);
      const cur = newData.current;
      const liveTimestamps = [...timestamps];
      const liveValues = [...values];

      const lastTimestamp = timestamps[timestamps.length - 1] || 0;
      const timeSinceLastPoint = cur.timestamp - lastTimestamp;

      // Only add current point if it's significantly different from last recorded point
      if (timeSinceLastPoint > UPDATE_INTERVAL_MS * 0.5) {
        liveTimestamps.push(cur.timestamp);
        liveValues.push(cur.value);

        // This will trigger the animation in the data updates effect above
        // by updating newData.long, so we don't need to animate here
      } else {
        // Just update the display with current data including live point
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

          uplotRef.current.setScale("x", { min: xMin, max: xMax });
          updateYAxisScale(liveTimestamps, liveValues, xMin, xMax);
        } else if (viewMode === "all") {
          const fullStart = startTimeRef.current ?? liveTimestamps[0];
          uplotRef.current.setScale("x", {
            min: fullStart,
            max: latestTimestamp,
          });
          updateYAxisScale(
            liveTimestamps,
            liveValues,
            fullStart,
            latestTimestamp,
          );
        }
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

  // Excel export function
  const exportToExcel = () => {
    try {
      if (!newData?.long) {
        alert("No data to export");
        return;
      }

      const [timestamps, values] = seriesToUPlotData(newData.long);

      if (timestamps.length === 0) {
        alert("No data to export");
        return;
      }

      // Prepare data for Excel export with proper timestamp formatting
      const exportData = timestamps.map((timestamp, index) => {
        const row: any = {
          Timestamp: new Date(timestamp),
          [`Value (${renderUnitSymbol(unit)})`]: renderValue
            ? renderValue(values[index])
            : values[index]?.toFixed(3) || "",
        };

        // Add line values for comparison
        config.lines?.forEach((line) => {
          row[`${line.label} (${renderUnitSymbol(unit)})`] = renderValue
            ? renderValue(line.value)
            : line.value.toFixed(3);

          if (line.type === "threshold") {
            row[`Within ${line.label}`] =
              Math.abs(values[index] - line.value) <= line.value * 0.05
                ? "Yes"
                : "No";
          }
        });

        return row;
      });

      // Create workbook and worksheet
      const workbook = XLSX.utils.book_new();
      const worksheet = XLSX.utils.json_to_sheet(exportData);

      // Create summary data
      const summaryData = [
        [`${config.title} Export Summary`, ""],
        ["Export Date", new Date()],
        ["", ""],
        ["Parameters", ""],
      ];

      // Add lines to summary
      config.lines?.forEach((line) => {
        summaryData.push([
          `${line.label} (${renderUnitSymbol(unit)})`,
          renderValue ? renderValue(line.value) : line.value.toFixed(3),
        ]);
      });

      // Add statistics
      summaryData.push(["", ""], ["Statistics", ""]);
      summaryData.push(["Total Data Points", timestamps.length.toString()]);
      summaryData.push(["Time Range Start", new Date(timestamps[0])]);
      summaryData.push([
        "Time Range End",
        new Date(timestamps[timestamps.length - 1]),
      ]);
      summaryData.push([
        `Min Value (${renderUnitSymbol(unit)})`,
        renderValue
          ? renderValue(Math.min(...values))
          : Math.min(...values).toFixed(3),
      ]);
      summaryData.push([
        `Max Value (${renderUnitSymbol(unit)})`,
        renderValue
          ? renderValue(Math.max(...values))
          : Math.max(...values).toFixed(3),
      ]);
      summaryData.push([
        `Average Value (${renderUnitSymbol(unit)})`,
        renderValue
          ? renderValue(values.reduce((a, b) => a + b, 0) / values.length)
          : (values.reduce((a, b) => a + b, 0) / values.length).toFixed(3),
      ]);

      const summaryWorksheet = XLSX.utils.aoa_to_sheet(summaryData);
      XLSX.utils.book_append_sheet(workbook, summaryWorksheet, "Summary");
      XLSX.utils.book_append_sheet(workbook, worksheet, "Data");

      const timestamp = new Date()
        .toISOString()
        .replace(/[:.]/g, "-")
        .slice(0, 19);
      const filename = config.exportFilename
        ? `${config.exportFilename}_${timestamp}.xlsx`
        : `${config.title.toLowerCase().replace(/\s+/g, "_")}_${timestamp}.xlsx`;

      XLSX.writeFile(workbook, filename);
    } catch (error) {
      alert("Error exporting data to Excel. Please try again.");
    }
  };

  // Handle time window change with immediate application
  const handleTimeWindowChange = (newTimeWindow: number | "all") => {
    setSelectedTimeWindow(newTimeWindow);

    // Stop any ongoing animations when changing time window
    stopAnimations();

    if (!uplotRef.current || !newData?.long) return;

    const [timestamps, values] = seriesToUPlotData(newData.long);
    if (timestamps.length === 0) return;

    if (newTimeWindow === "all") {
      setViewMode("all");
      setIsLiveMode(true);
      const fullStart = startTimeRef.current ?? timestamps[0];
      const fullEnd = timestamps[timestamps.length - 1];

      uplotRef.current.setScale("x", {
        min: fullStart,
        max: fullEnd,
      });
      manualScaleRef.current = null;
    } else if (isLiveMode) {
      const latestTimestamp = timestamps[timestamps.length - 1];
      const viewStart = latestTimestamp - newTimeWindow;

      uplotRef.current.setScale("x", {
        min: viewStart,
        max: latestTimestamp,
      });
      setViewMode("default");
      manualScaleRef.current = null;
    } else {
      const rightmostTimestamp = getRightmostVisibleTimestamp();
      if (rightmostTimestamp) {
        const newViewStart = rightmostTimestamp - newTimeWindow;
        const minY = Math.min(...values);
        const maxY = Math.max(...values);
        const range = maxY - minY || 1;

        uplotRef.current.setScale("x", {
          min: newViewStart,
          max: rightmostTimestamp,
        });

        manualScaleRef.current = {
          x: { min: newViewStart, max: rightmostTimestamp },
          y: { min: minY - range * 0.1, max: maxY + range * 0.1 },
        };
        setViewMode("manual");
      }
    }

    updateYAxisScale(timestamps, values);

    // Update last rendered data to current state
    lastRenderedDataRef.current = { timestamps, values };
  };

  const displayValue =
    cursorValue !== null ? cursorValue : newData?.current?.value;

  const getSelectedTimeWindowLabel = () => {
    const option = timeWindowOptions.find(
      (opt) => opt.value === selectedTimeWindow,
    );
    return option ? option.label : "1m";
  };

  return (
    <div className="h-[50vh] w-full">
      <div className="flex h-full w-full flex-col overflow-hidden rounded-3xl border border-gray-200 bg-white shadow">
        {/* Header */}
        <div className="flex items-center justify-between pt-4 pr-5 pb-6 pl-6">
          {/* Left side - Icon, Title, Current value */}
          <div className="-mt-1 flex items-center gap-4">
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
          {/* Right side - Time window dropdown, View buttons, Export */}
          <div className="flex items-center gap-4">
            {/* Time window dropdown */}
            <DropdownMenu>
              <DropdownMenuTrigger asChild>
                <TouchButton
                  variant="outline"
                  className="border-gray-300 px-3 py-2 text-base font-medium text-gray-900 hover:bg-gray-50"
                >
                  {getSelectedTimeWindowLabel()}
                  <Icon name="lu:ChevronDown" className="ml-2 size-4" />
                </TouchButton>
              </DropdownMenuTrigger>
              <DropdownMenuContent align="end">
                <DropdownMenuLabel className="text-base font-medium">
                  Time Window
                </DropdownMenuLabel>
                <DropdownMenuSeparator />
                {timeWindowOptions.map((option) => (
                  <DropdownMenuItem
                    key={option.value}
                    onClick={() => handleTimeWindowChange(option.value)}
                    className={`min-h-[48px] px-4 py-3 text-base ${
                      selectedTimeWindow === option.value ? "bg-blue-50" : ""
                    }`}
                  >
                    {option.label}
                  </DropdownMenuItem>
                ))}
              </DropdownMenuContent>
            </DropdownMenu>

            {/* View Buttons */}
            <div className="flex items-center gap-4">
              <TouchButton
                onClick={switchToHistoricalMode}
                variant="outline"
                className={`px-3 py-2 text-base font-medium transition-colors ${
                  !isLiveMode
                    ? "bg-black text-white shadow-sm"
                    : "border-gray-300 text-gray-700 hover:bg-gray-100"
                }`}
              >
                Historical
              </TouchButton>
              <TouchButton
                onClick={switchToLiveMode}
                variant="outline"
                className={`px-3 py-2 text-base font-medium transition-colors ${
                  isLiveMode
                    ? "bg-black text-white shadow-sm"
                    : "border-gray-300 text-gray-700 hover:bg-gray-100"
                }`}
              >
                Live
              </TouchButton>
            </div>

            {/* Separator line */}
            <div className="h-12 w-px bg-gray-300"></div>

            {/* Export Button */}
            <TouchButton
              onClick={exportToExcel}
              variant="outline"
              className="bg-green-600 px-3 py-2 text-base font-medium text-white transition-colors hover:bg-green-100"
            >
              Export
            </TouchButton>
          </div>
        </div>

        {/* Separator line with padding */}
        <div className="-mt-2 px-6">
          <div className="h-px bg-gray-200"></div>
        </div>

        {/* Graph Container - full width with only vertical padding */}
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

// Convenience wrapper for diameter graphs
export function DiameterGraph({
  newData,
  threshold1,
  threshold2,
  target,
  unit,
  renderValue,
}: {
  newData: TimeSeries | null;
  threshold1: number;
  threshold2: number;
  target: number;
  unit?: Unit;
  renderValue?: (value: number) => string;
}) {
  const config: GraphConfig = {
    title: "Diameter",
    description: "Real-time diameter measurements with thresholds",
    icon: "lu:Circle",
    lines: [
      {
        type: "threshold",
        value: threshold1,
        label: "Upper Threshold",
        color: "#ef4444",
        dash: [5, 5],
      },
      {
        type: "threshold",
        value: threshold2,
        label: "Lower Threshold",
        color: "#f97316",
        dash: [5, 5],
      },
      {
        type: "target",
        value: target,
        label: "Target",
        color: "#6b7280",
      },
    ],
    colors: {
      primary: "#3b82f6",
      grid: "#e2e8f0",
      axis: "#64748b",
      background: "#ffffff",
    },
    exportFilename: "diameter_data",
  };

  return (
    <BigGraph
      newData={newData}
      unit={unit}
      renderValue={renderValue}
      config={config}
    />
  );
}
