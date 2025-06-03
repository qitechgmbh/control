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
  const UPDATE_INTERVAL_MS = newData?.long?.sampleInterval ?? 100;

  const timeWindowOptions = config.timeWindows ?? DEFAULT_TIME_WINDOW_OPTIONS;
  const colors = {
    primary: config.colors?.primary ?? "#3b82f6",
    grid: config.colors?.grid ?? "#e2e8f0",
    axis: config.colors?.axis ?? "#64748b",
    background: config.colors?.background ?? "#ffffff",
  };

  // Helper functions
  const updateYAxisScale = (values: number[]) => {
    if (!uplotRef.current || values.length === 0) return;

    const minY = Math.min(...values);
    const maxY = Math.max(...values);
    const range = maxY - minY || 1;

    uplotRef.current.setScale("y", {
      min: minY - range * 0.1,
      max: maxY + range * 0.1,
    });
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
  ): uPlot.AlignedData => {
    const uData: uPlot.AlignedData = [timestamps, values];

    // Add line data for each configured line
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
        padding: [10, 20, 20, 20],
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
                  const currentYScale = u.scales.y;
                  manualScaleRef.current = {
                    x: { min: xScale.min, max: xScale.max },
                    y: {
                      min: currentYScale?.min ?? minY - range * 0.1,
                      max: currentYScale?.max ?? maxY + range * 0.1,
                    },
                  };
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

    // Touch/drag handling
    const handleTouchStart = (e: TouchEvent) => {
      if (e.touches.length === 1) {
        isDraggingRef.current = true;
        lastDragXRef.current = e.touches[0].clientX;
        e.preventDefault();
      }
    };

    const handleTouchMove = (e: TouchEvent) => {
      if (
        isDraggingRef.current &&
        e.touches.length === 1 &&
        lastDragXRef.current !== null
      ) {
        const currentX = e.touches[0].clientX;
        const deltaX = currentX - lastDragXRef.current;
        lastDragXRef.current = currentX;

        if (uplotRef.current && Math.abs(deltaX) > 2) {
          const xScale = uplotRef.current.scales.x;
          if (xScale && xScale.min !== undefined && xScale.max !== undefined) {
            const pixelToTime = (xScale.max - xScale.min) / width;
            const timeDelta = -deltaX * pixelToTime;

            const newMin = xScale.min + timeDelta;
            const newMax = xScale.max + timeDelta;

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
        e.preventDefault();
      }
    };

    const handleTouchEnd = (e: TouchEvent) => {
      isDraggingRef.current = false;
      lastDragXRef.current = null;
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
  };
  const switchToLiveMode = () => {
    setIsLiveMode(true);

    // Don't change viewMode if "Show All" is selected
    if (selectedTimeWindow !== "all") {
      setViewMode("default");
    }

    manualScaleRef.current = null;

    if (uplotRef.current && newData?.long) {
      const [timestamps, values] = seriesToUPlotData(newData.long);
      if (timestamps.length > 0) {
        const latestTimestamp = timestamps[timestamps.length - 1];

        if (selectedTimeWindow === "all") {
          const fullStart = startTimeRef.current ?? timestamps[0];
          uplotRef.current.setScale("x", {
            min: fullStart,
            max: latestTimestamp,
          });
        } else {
          const viewStart = latestTimestamp - (selectedTimeWindow as number);
          uplotRef.current.setScale("x", {
            min: viewStart,
            max: latestTimestamp,
          });
        }
        updateYAxisScale(values);
      }
    }
  };

  const switchToHistoricalMode = () => {
    setIsLiveMode(false);
    setViewMode("manual");

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
      chartCreatedRef.current = false;
    };
  }, [newData?.long, containerRef.current]); // Recreate when data or container changes

  // Data updates effect - only update existing chart data
  useEffect(() => {
    if (
      !uplotRef.current ||
      !newData?.long ||
      isScrollingRef.current ||
      !chartCreatedRef.current
    )
      return;

    const [timestamps, values] = seriesToUPlotData(newData.long);
    if (timestamps.length === 0) return;

    // Build updated data
    const uData = buildUPlotData(timestamps, values);

    // Update chart data
    uplotRef.current.setData(uData);

    // Update view based on mode
    const lastTimestamp = timestamps[timestamps.length - 1] ?? 0;
    const { min: minY, max: maxY } = getSeriesMinMax(newData.long);
    const range = maxY - minY || 1;
    const yMin = minY - range * 0.1;
    const yMax = maxY + range * 0.1;

    if (viewMode === "default" && isLiveMode) {
      if (selectedTimeWindow === "all") {
        const fullStart = startTimeRef.current ?? timestamps[0];
        uplotRef.current.setScale("x", {
          min: fullStart,
          max: lastTimestamp,
        });
      } else {
        const defaultViewStart = lastTimestamp - (selectedTimeWindow as number);
        uplotRef.current.setScale("x", {
          min: defaultViewStart,
          max: lastTimestamp,
        });
      }
      uplotRef.current.setScale("y", { min: yMin, max: yMax });
    } else if (viewMode === "all") {
      const fullStart = startTimeRef.current ?? timestamps[0];
      uplotRef.current.setScale("x", {
        min: fullStart,
        max: lastTimestamp,
      });
      uplotRef.current.setScale("y", { min: yMin, max: yMax });
    } else if (viewMode === "manual" || !isLiveMode) {
      if (manualScaleRef.current) {
        uplotRef.current.setScale("x", {
          min: manualScaleRef.current.x.min,
          max: manualScaleRef.current.x.max,
        });
        uplotRef.current.setScale("y", { min: yMin, max: yMax });
      }
    }
  }, [
    newData?.long?.validCount,
    newData?.long?.lastTimestamp,
    viewMode,
    selectedTimeWindow,
    isLiveMode,
  ]);

  // Live updates useEffect
  useEffect(() => {
    if (
      !uplotRef.current ||
      !newData?.current ||
      isScrollingRef.current ||
      !isLiveMode ||
      !chartCreatedRef.current
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

      if (timeSinceLastPoint > UPDATE_INTERVAL_MS * 0.5) {
        liveTimestamps.push(cur.timestamp);
        liveValues.push(cur.value);
      }

      if (liveTimestamps.length === 0) return;

      if (startTimeRef.current === null) {
        startTimeRef.current = liveTimestamps[0];
      }

      const minY = Math.min(...liveValues);
      const maxY = Math.max(...liveValues);
      const range = maxY - minY || 1;

      // Build live data structure
      const liveData = buildUPlotData(liveTimestamps, liveValues);

      uplotRef.current.setData(liveData);

      const latestTimestamp = liveTimestamps[liveTimestamps.length - 1];
      const yMin = minY - range * 0.1;
      const yMax = maxY + range * 0.1;

      if (viewMode === "default" && isLiveMode) {
        if (selectedTimeWindow === "all") {
          const fullStart = startTimeRef.current ?? liveTimestamps[0];
          uplotRef.current.setScale("x", {
            min: fullStart,
            max: latestTimestamp,
          });
        } else {
          const defaultViewStart =
            latestTimestamp - (selectedTimeWindow as number);
          uplotRef.current.setScale("x", {
            min: defaultViewStart,
            max: latestTimestamp,
          });
        }
        uplotRef.current.setScale("y", { min: yMin, max: yMax });
      } else if (viewMode === "all" && isLiveMode) {
        const fullStart = startTimeRef.current ?? liveTimestamps[0];
        uplotRef.current.setScale("x", {
          min: fullStart,
          max: latestTimestamp,
        });
        uplotRef.current.setScale("y", { min: yMin, max: yMax });
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

    if (!uplotRef.current || !newData?.long) return;

    const [timestamps, values] = seriesToUPlotData(newData.long);
    if (timestamps.length === 0) return;

    if (newTimeWindow === "all") {
      setViewMode("all");
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

    updateYAxisScale(values);
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
      <div className="flex h-full w-full flex-col rounded-2xl border border-gray-200 bg-white shadow-sm">
        {/* Header */}
        <div className="flex items-center justify-between p-4">
          {/* Left side - Icon, Title, Current value */}
          <div className="flex items-center gap-4">
            <Icon
              name={unit ? getUnitIcon(unit) : "lu:TrendingUp"}
              className="size-6 text-gray-600"
            />

            <h2 className="text-lg font-semibold text-gray-900">
              {config.title}
            </h2>

            <div className="flex items-center gap-2 text-sm text-gray-600">
              <span className="font-mono font-bold text-gray-900">
                {displayValue !== undefined && displayValue !== null
                  ? renderValue
                    ? renderValue(displayValue)
                    : displayValue.toFixed(3)
                  : "N/A"}
              </span>
              <span className="text-gray-500">{renderUnitSymbol(unit)}</span>
            </div>
          </div>

          {/* Right side - Time window dropdown, View buttons, Export */}
          <div className="flex items-center gap-4">
            {/* Time window dropdown */}
            <DropdownMenu>
              <DropdownMenuTrigger asChild>
                <TouchButton
                  variant="outline"
                  className="border-gray-300 px-3 py-2 text-sm font-medium text-gray-900 hover:bg-gray-50"
                >
                  {getSelectedTimeWindowLabel()}
                  <Icon name="lu:ChevronDown" className="ml-2 size-4" />
                </TouchButton>
              </DropdownMenuTrigger>
              <DropdownMenuContent align="end">
                <DropdownMenuLabel>Time Window</DropdownMenuLabel>
                <DropdownMenuSeparator />
                {timeWindowOptions.map((option) => (
                  <DropdownMenuItem
                    key={option.value}
                    onClick={() => handleTimeWindowChange(option.value)}
                    className={`min-h-[44px] px-4 py-3 text-base ${
                      selectedTimeWindow === option.value ? "bg-blue-50" : ""
                    }`}
                  >
                    {option.label}
                  </DropdownMenuItem>
                ))}
              </DropdownMenuContent>
            </DropdownMenu>

            {/* View Buttons */}
            <div className="flex items-center gap-2">
              <TouchButton
                onClick={switchToLiveMode}
                variant="outline"
                className={`px-3 py-2 text-sm font-medium transition-colors ${
                  isLiveMode
                    ? "bg-blue-500 text-white shadow-sm"
                    : "border-gray-300 text-gray-700 hover:bg-gray-100"
                }`}
              >
                Live
              </TouchButton>

              <TouchButton
                onClick={switchToHistoricalMode}
                variant="outline"
                className={`px-3 py-2 text-sm font-medium transition-colors ${
                  !isLiveMode
                    ? "bg-blue-500 text-white shadow-sm"
                    : "border-gray-300 text-gray-700 hover:bg-gray-100"
                }`}
              >
                Historical
              </TouchButton>
            </div>

            {/* Separator line */}
            <div className="h-10 w-px bg-gray-300"></div>

            {/* Export Button */}
            <TouchButton
              onClick={exportToExcel}
              variant="outline"
              className="bg-green-50 px-3 py-2 text-sm font-medium text-green-700 transition-colors hover:bg-green-100"
            >
              Export
            </TouchButton>
          </div>
        </div>

        {/* Separator line with padding */}
        <div className="px-4">
          <div className="h-px bg-gray-200"></div>
        </div>

        {/* Graph Container - full width with only vertical padding */}
        <div className="flex-1">
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
