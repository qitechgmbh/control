// BigGraph.tsx
import React, { useRef, useState, useCallback, useEffect } from "react";
import uPlot from "uplot";
import "uplot/dist/uPlot.min.css";
import { renderUnitSymbol, getUnitIcon } from "@/control/units";
import { Icon } from "@/components/Icon";
import { BigGraphProps, HandlerRefs } from "./types";
import { GraphExportData } from "./excelExport";
import { DEFAULT_COLORS } from "./constants";
import {
  useAnimationRefs,
  stopAnimations,
  normalizeDataSeries,
  getPrimarySeries,
  formatDisplayValue,
} from "./animation";
import { useLiveMode } from "./liveMode";
import { useHistoricalMode } from "./historicalMode";
import { useBigGraphEffects } from "./useBigGraphEffects";
import { TouchButton } from "../touch/TouchButton";
import { seriesToUPlotData } from "@/lib/timeseries";
import { ControlCard } from "@/control/ControlCard";

// Helper function to collect lines connected to visible series
const getVisibleLines = (data: any, visibleSeries: boolean[]) => {
  const visibleLines: any[] = [];

  if (Array.isArray(data)) {
    // Multiple series - collect lines only from visible series
    data.forEach((series, index) => {
      if (visibleSeries[index] && series.lines && Array.isArray(series.lines)) {
        // Add series info to each line for proper styling
        const seriesLines = series.lines.map((line: any) => ({
          ...line,
          // Override color with series color if not explicitly set
          color: line.color || series.color,
          // Ensure lines are always dashed
          dash: line.dash || [5, 5],
          // Mark as connected to this series
          seriesIndex: index,
          seriesTitle: series.title,
        }));
        visibleLines.push(...seriesLines);
      }
    });
  } else {
    // Single series - include lines only if series is visible
    if (visibleSeries[0] && data.lines && Array.isArray(data.lines)) {
      const seriesLines = data.lines.map((line: any) => ({
        ...line,
        // Override color with series color if not explicitly set
        color: line.color || data.color,
        // Ensure lines are always dashed
        dash: line.dash || [5, 5],
        // Mark as connected to this series
        seriesIndex: 0,
        seriesTitle: data.title,
      }));
      visibleLines.push(...seriesLines);
    }
  }

  return visibleLines;
};

// Helper function to merge config lines with series-connected lines
const mergeConfigWithVisibleLines = (
  config: any,
  data: any,
  visibleSeries: boolean[],
) => {
  const configLines = config.lines || [];
  const seriesLines = getVisibleLines(data, visibleSeries);

  // Combine config lines with visible series lines
  const allLines = [...configLines, ...seriesLines];

  // Remove duplicates based on label and series connection
  const uniqueLines = allLines.filter((line, index, arr) => {
    return (
      arr.findIndex((l) => {
        // For series-connected lines, check both label and series index
        if (line.seriesIndex !== undefined && l.seriesIndex !== undefined) {
          return l.label === line.label && l.seriesIndex === line.seriesIndex;
        }
        // For config lines, just check label or value
        return (
          (l.label && l.label === line.label) ||
          (l.value === line.value && l.type === line.type)
        );
      }) === index
    );
  });

  return {
    ...config,
    lines: uniqueLines,
  };
};

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

  // Animation refs
  const animationRefs = useAnimationRefs();

  // Series visibility state
  const normalizedSeries = normalizeDataSeries(newData);
  const [visibleSeries, setVisibleSeries] = useState<boolean[]>(
    new Array(normalizedSeries.length).fill(true),
  );

  // Merge config with lines from visible series only
  const enhancedConfig = React.useMemo(() => {
    return mergeConfigWithVisibleLines(config, newData, visibleSeries);
  }, [config, newData, visibleSeries]); // Include visibleSeries in dependency

  // Track when visibility changes to force chart recreation
  const [visibilityVersion, setVisibilityVersion] = useState(0);
  const [, setIsRecreatingChart] = useState(false);

  // Initialize state from props or defaults
  const [viewMode, setViewMode] = useState<"default" | "all" | "manual">(
    syncGraph?.viewMode ?? "default",
  );
  const [isLiveMode, setIsLiveMode] = useState(syncGraph?.isLiveMode ?? true);
  const [selectedTimeWindow, setSelectedTimeWindow] = useState<number | "all">(
    syncGraph?.timeWindow ?? enhancedConfig.defaultTimeWindow ?? 30 * 60 * 1000,
  );

  const [cursorValue, setCursorValue] = useState<number | null>(null);
  const startTimeRef = useRef<number | null>(null);
  const manualScaleRef = useRef<{
    x: { min: number; max: number };
    y: { min: number; max: number };
  } | null>(null);
  const lastProcessedCountRef = useRef(0);

  // Handler refs
  const handlerRefs: HandlerRefs = {
    isUserZoomingRef: useRef(false),
    isDraggingRef: useRef(false),
    lastDragXRef: useRef<number | null>(null),
    isPinchingRef: useRef(false),
    lastPinchDistanceRef: useRef<number | null>(null),
    pinchCenterRef: useRef<{ x: number; y: number } | null>(null),
    touchStartRef: useRef<{ x: number; y: number; time: number } | null>(null),
    touchDirectionRef: useRef<"horizontal" | "vertical" | "unknown">("unknown"),
  };

  const colors = {
    primary: enhancedConfig.colors?.primary ?? DEFAULT_COLORS.primary,
    grid: enhancedConfig.colors?.grid ?? DEFAULT_COLORS.grid,
    axis: enhancedConfig.colors?.axis ?? DEFAULT_COLORS.axis,
    background: enhancedConfig.colors?.background ?? DEFAULT_COLORS.background,
  };

  // Create filtered data based on visibility
  const filteredData = React.useMemo(() => {
    if (Array.isArray(newData)) {
      const filtered = newData.filter((_, index) => visibleSeries[index]);
      return filtered.length > 0 ? filtered : [newData[0]]; // Ensure at least one series
    }
    return visibleSeries[0] ? newData : { newData: null };
  }, [newData, visibleSeries]);

  // Register export function that includes ALL series and their lines
  useEffect(() => {
    if (onRegisterForExport) {
      const getExportData = (): GraphExportData | null => {
        // For export, include all lines from all series
        const allSeriesLines = getVisibleLines(
          newData,
          new Array(normalizedSeries.length).fill(true),
        );
        const exportConfig = {
          ...config,
          lines: [...(config.lines || []), ...allSeriesLines],
        };

        return {
          config: exportConfig,
          data: newData,
          unit,
          renderValue,
        };
      };

      onRegisterForExport(graphId, getExportData);

      return () => {
        if (onUnregisterFromExport) {
          onUnregisterFromExport(graphId);
        }
      };
    }
  }, [
    onRegisterForExport,
    onUnregisterFromExport,
    graphId,
    config,
    newData,
    normalizedSeries.length,
    unit,
    renderValue,
  ]);

  const updateYAxisScale = useCallback(
    (xMin?: number, xMax?: number) => {
      if (!uplotRef.current) return;

      const isInHistoricalMode = !isLiveMode || viewMode === "manual";
      if (isInHistoricalMode) {
        return; // Don't update Y-axis in historical mode
      }

      const normalizedData = normalizeDataSeries(newData);
      let allVisibleValues: number[] = [];

      // Collect values from all visible series
      normalizedData.forEach((series, index) => {
        if (!visibleSeries[index] || !series.newData?.long) return;

        const [timestamps, values] = seriesToUPlotData(series.newData.long);
        if (values.length === 0) return;

        let seriesToInclude: number[] = [];

        if (xMin !== undefined && xMax !== undefined) {
          for (let i = 0; i < timestamps.length; i++) {
            if (timestamps[i] >= xMin && timestamps[i] <= xMax) {
              seriesToInclude.push(values[i]);
            }
          }
        } else {
          seriesToInclude = [...values];
        }

        allVisibleValues.push(...seriesToInclude);
      });

      // Include line values in Y-axis calculation (only visible lines)
      enhancedConfig.lines?.forEach((line) => {
        if (line.show !== false) {
          allVisibleValues.push(line.value);
        }
      });

      if (allVisibleValues.length === 0) {
        const primarySeries = getPrimarySeries(filteredData);
        if (primarySeries?.newData?.long) {
          const [, values] = seriesToUPlotData(primarySeries.newData.long);
          allVisibleValues = values;
        }
      }

      if (allVisibleValues.length === 0) return;

      const minY = Math.min(...allVisibleValues);
      const maxY = Math.max(...allVisibleValues);
      const range = maxY - minY || Math.abs(maxY) * 0.1 || 1;

      const yRange = {
        min: minY - range * 0.1,
        max: maxY + range * 0.1,
      };

      try {
        uplotRef.current.batch(() => {
          uplotRef.current!.setScale("y", yRange);
        });
      } catch (error) {
        console.warn("Error updating Y-axis scale:", error);
      }
    },
    [enhancedConfig.lines, viewMode, isLiveMode, newData, visibleSeries],
  );

  // Initialize live mode handlers with filtered data
  const liveMode = useLiveMode({
    newData: filteredData,
    uplotRef,
    config: enhancedConfig,
    animationRefs,
    viewMode,
    selectedTimeWindow,
    startTimeRef,
    updateYAxisScale,
    lastProcessedCountRef,
    chartCreatedRef,
  });

  // Initialize historical mode handlers with filtered data
  const historicalMode = useHistoricalMode({
    newData: filteredData,
    uplotRef,
    animationRefs,
    getCurrentLiveEndTimestamp: liveMode.getCurrentLiveEndTimestamp,
    updateYAxisScale,
    lastProcessedCountRef,
    manualScaleRef,
  });

  const handleTimeWindowChangeInternal = useCallback(
    (newTimeWindow: number | "all", isSync: boolean = false) => {
      stopAnimations(animationRefs);
      setSelectedTimeWindow(newTimeWindow);

      if (!uplotRef.current) {
        return;
      }

      if (newTimeWindow === "all") {
        setViewMode("all");
        if (!isLiveMode) {
          setIsLiveMode(true);
          historicalMode.switchToLiveMode();
        }
        liveMode.handleLiveTimeWindow(newTimeWindow);
      } else {
        setViewMode("default");
        if (isLiveMode) {
          liveMode.handleLiveTimeWindow(newTimeWindow);
        } else {
          historicalMode.handleHistoricalTimeWindow(newTimeWindow);
        }
      }

      if (!isSync && syncGraph?.onTimeWindowChange) {
        syncGraph.onTimeWindowChange(graphId, newTimeWindow);
      }
    },
    [
      animationRefs,
      isLiveMode,
      liveMode.handleLiveTimeWindow,
      historicalMode.handleHistoricalTimeWindow,
      historicalMode.switchToLiveMode,
      syncGraph,
      graphId,
    ],
  );

  // Toggle series visibility with smooth chart recreation
  const toggleSeries = useCallback((index: number) => {
    setVisibleSeries((prev) => {
      const newVisibility = [...prev];

      // Check if this would hide all series
      const wouldHideAll =
        newVisibility[index] &&
        newVisibility.filter((visible, i) => i !== index && visible).length ===
          0;

      if (wouldHideAll) {
        return prev; // Don't allow hiding the last visible series
      }

      newVisibility[index] = !newVisibility[index];

      // Set recreating flag to prevent flicker
      setIsRecreatingChart(true);

      // Force chart recreation by incrementing version
      setVisibilityVersion((v) => v + 1);

      return newVisibility;
    });
  }, []);

  // Force chart recreation when visibility changes
  useEffect(() => {
    if (visibilityVersion > 0 && uplotRef.current) {
      // Destroy existing chart
      uplotRef.current.destroy();
      uplotRef.current = null;
      chartCreatedRef.current = false;

      // Stop any ongoing animations
      stopAnimations(animationRefs);

      // Reset processed count to force recreation
      lastProcessedCountRef.current = 0;

      // Clear recreating flag after a short delay to allow chart to rebuild
      setTimeout(() => {
        setIsRecreatingChart(false);
      }, 100);
    }
  }, [visibilityVersion, animationRefs]);

  // Use the extracted useEffect hooks with filtered data
  useBigGraphEffects({
    // Refs
    containerRef,
    uplotRef,
    startTimeRef,
    manualScaleRef,
    lastProcessedCountRef,
    animationRefs,
    handlerRefs,
    chartCreatedRef,

    // Props - Use enhanced config and filtered data
    newData: filteredData,
    unit,
    renderValue,
    config: enhancedConfig,
    graphId,
    syncGraph,
    onRegisterForExport,
    onUnregisterFromExport,

    // State
    viewMode,
    isLiveMode,
    selectedTimeWindow,
    visibleSeries,

    // State setters
    setSelectedTimeWindow,
    setViewMode,
    setIsLiveMode,
    setCursorValue,

    // Handlers
    liveMode,
    historicalMode,
    colors,
    updateYAxisScale,
    handleTimeWindowChangeInternal,
  });

  // Use filtered data for display value calculation
  const primarySeries = getPrimarySeries(filteredData);
  const displayValue =
    cursorValue !== null ? cursorValue : primarySeries?.newData?.current?.value;

  const defaultColors = ["#3b82f6", "#ef4444", "#10b981", "#f59e0b", "#8b5cf6"];

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
              {enhancedConfig.title}
            </h2>
          </div>

          <div className="flex items-center gap-4">
            {normalizedSeries.length === 1 && (
              <ControlCard className="rounded-md px-4 py-3">
                <div className="flex items-center gap-2 text-base text-gray-600">
                  <span className="font-mono leading-none font-bold text-gray-900">
                    {formatDisplayValue(displayValue, renderValue)}
                  </span>
                  <span className="leading-none text-gray-500">
                    {renderUnitSymbol(unit)}
                  </span>
                </div>
              </ControlCard>
            )}

            {/* Series Toggle Controls */}
            {normalizedSeries.length > 1 && (
              <div className="flex items-center gap-2">
                <div className="flex items-center gap-1">
                  {normalizedSeries.map((series, index) => {
                    const seriesColor =
                      series.color ||
                      defaultColors[index % defaultColors.length];
                    const currentValue = series.newData?.current?.value;
                    const formattedValue = formatDisplayValue(
                      currentValue,
                      renderValue,
                    );

                    const isLastVisible =
                      visibleSeries[index] &&
                      visibleSeries.filter((visible) => visible).length === 1;

                    // Get connected lines for this series
                    const seriesLines = Array.isArray(newData)
                      ? newData[index]?.lines || []
                      : newData.lines || [];

                    return (
                      <TouchButton
                        key={index}
                        onClick={() => !isLastVisible && toggleSeries(index)}
                        className={`rounded-md px-4 py-1.5 text-sm transition-all ${
                          visibleSeries[index]
                            ? "text-white shadow-md"
                            : "bg-gray-100 text-gray-500 hover:bg-gray-200"
                        } ${isLastVisible ? "cursor-not-allowed" : ""}`}
                        style={{
                          backgroundColor: visibleSeries[index]
                            ? seriesColor
                            : undefined,
                        }}
                        title={`${series.title || `Series ${index + 1}`}: ${formattedValue} ${renderUnitSymbol(unit) || ""}${
                          seriesLines.length > 0
                            ? `\nLines: ${seriesLines.map((l) => l.label || `${l.value}`).join(", ")}`
                            : ""
                        }`}
                      >
                        <div className="flex flex-col items-center">
                          <span className="text-xs font-medium">
                            {series.title || `S${index + 1}`}
                          </span>
                          <span
                            className={`font-mono leading-none font-bold ${
                              visibleSeries[index]
                                ? "text-white"
                                : "text-gray-500"
                            }`}
                          >
                            {formatDisplayValue(currentValue, renderValue)}
                            {renderUnitSymbol(unit)}
                          </span>
                        </div>
                      </TouchButton>
                    );
                  })}
                </div>
              </div>
            )}
          </div>
        </div>

        <div className="-mt-2 px-6">
          <div className="h-px bg-gray-200"></div>
        </div>

        <div className="flex-1 overflow-hidden rounded-b-3xl pt-4">
          <div
            ref={containerRef}
            className={`h-full w-full overflow-hidden transition-opacity duration-100`}
            style={{ backgroundColor: colors.background }}
          />
        </div>
      </div>
    </div>
  );
}
