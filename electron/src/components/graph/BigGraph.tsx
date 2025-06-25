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

  // Track when visibility changes to force chart recreation
  const [visibilityVersion, setVisibilityVersion] = useState(0);
  const [isRecreatingChart, setIsRecreatingChart] = useState(false);

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
    primary: config.colors?.primary ?? DEFAULT_COLORS.primary,
    grid: config.colors?.grid ?? DEFAULT_COLORS.grid,
    axis: config.colors?.axis ?? DEFAULT_COLORS.axis,
    background: config.colors?.background ?? DEFAULT_COLORS.background,
  };

  // Create filtered data based on visibility - THIS IS THE KEY FIX
  const filteredData = React.useMemo(() => {
    if (Array.isArray(newData)) {
      const filtered = newData.filter((_, index) => visibleSeries[index]);
      return filtered.length > 0 ? filtered : [newData[0]]; // Ensure at least one series
    }
    return visibleSeries[0] ? newData : { newData: null };
  }, [newData, visibleSeries]);

  // Register export function that includes ALL series (not just visible ones)
  useEffect(() => {
    if (onRegisterForExport) {
      const getExportData = (): GraphExportData | null => {
        return {
          config,
          data: newData, // Use original newData for export
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

      config.lines?.forEach((line) => {
        if (line.show !== false) {
          allVisibleValues.push(line.value);
        }
      });

      if (allVisibleValues.length === 0) {
        const primarySeries = getPrimarySeries(filteredData); // Use filtered data
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
    [config.lines, viewMode, isLiveMode, newData, visibleSeries],
  );

  // FIXED: Initialize live mode handlers with filtered data
  const liveMode = useLiveMode({
    newData: filteredData, // Use filtered data
    uplotRef,
    config,
    animationRefs,
    viewMode,
    selectedTimeWindow,
    startTimeRef,
    updateYAxisScale,
    lastProcessedCountRef,
    chartCreatedRef,
  });

  // FIXED: Initialize historical mode handlers with filtered data
  const historicalMode = useHistoricalMode({
    newData: filteredData, // Use filtered data
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

    // Props - FIXED: Use filtered data for chart operations
    newData: filteredData, // Use filtered data
    unit,
    renderValue,
    config,
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
              {config.title}
            </h2>

            {normalizedSeries.length === 1 && (
              <div className="flex items-center gap-2 text-base text-gray-600">
                <span className="font-mono leading-none font-bold text-gray-900">
                  {formatDisplayValue(displayValue, renderValue)}
                </span>
                <span className="leading-none text-gray-500">
                  {renderUnitSymbol(unit)}
                </span>
              </div>
            )}
          </div>

          {/* Series Toggle Controls */}
          {normalizedSeries.length > 1 && (
            <div className="flex items-center gap-2">
              <div className="flex items-center gap-1">
                {normalizedSeries.map((series, index) => {
                  const seriesColor =
                    series.color || defaultColors[index % defaultColors.length];
                  const currentValue = series.newData?.current?.value;
                  const formattedValue = formatDisplayValue(
                    currentValue,
                    renderValue,
                  );

                  const isLastVisible =
                    visibleSeries[index] &&
                    visibleSeries.filter((visible) => visible).length === 1;

                  return (
                    <TouchButton
                      key={index}
                      onClick={() => !isLastVisible && toggleSeries(index)}
                      className={`rounded-md px-3 py-1 text-sm transition-all ${
                        visibleSeries[index]
                          ? "text-white shadow-md"
                          : "bg-gray-100 text-gray-500 hover:bg-gray-200"
                      } ${isLastVisible ? "cursor-not-allowed" : ""}`}
                      style={{
                        backgroundColor: visibleSeries[index]
                          ? seriesColor
                          : undefined,
                      }}
                      title={`${series.title || `Series ${index + 1}`}: ${formattedValue} ${renderUnitSymbol(unit) || ""}`}
                    >
                      <div className="flex flex-col items-center">
                        <span className="text-xs font-medium">
                          {series.title || `S${index + 1}`}
                        </span>
                        <span className="font-mono leading-none font-bold text-white">
                          {formattedValue} {renderUnitSymbol(unit)}
                        </span>
                      </div>
                    </TouchButton>
                  );
                })}
              </div>
            </div>
          )}
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
