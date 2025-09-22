import React, { useRef, useState, useCallback, useEffect } from "react";
import uPlot from "uplot";
import "uplot/dist/uPlot.min.css";
import { renderUnitSymbol, getUnitIcon } from "@/control/units";
import { Icon } from "@/components/Icon";
import { BigGraphProps, HandlerRefs, SeriesData } from "./types";
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
import { ControlCard } from "@/control/ControlCard";

// Utility: lazy filter series to only points in window
function sliceSeriesForWindow(
  series: SeriesData,
  startTime: number,
  endTime: number
): SeriesData {
  if (!series.newData?.long) return series;
  return {
    ...series,
    newData: {
      ...series.newData,
      long: series.newData.long.filter(
        ([ts]) => ts >= startTime && ts <= endTime
      ),
    },
  };
}

// Merge config with visible lines
const mergeConfigWithVisibleLines = (config: any, data: any, visible: boolean[]) => {
  const configLines = config.lines || [];
  const seriesLines: any[] = [];

  const seriesArray = Array.isArray(data) ? data : [data];
  seriesArray.forEach((series, i) => {
    if (visible[i] && series.lines) {
      series.lines.forEach((line: any) => {
        seriesLines.push({
          ...line,
          color: line.color || series.color,
          seriesIndex: i,
          seriesTitle: series.title,
        });
      });
    }
  });

  return {
    ...config,
    lines: [...configLines, ...seriesLines],
  };
};

export function BigGraphLazy({
  newData,
  unit,
  renderValue,
  config,
  graphId,
  syncGraph,
  onRegisterForExport,
  onUnregisterFromExport,
}: BigGraphProps & {
  onRegisterForExport?: (graphId: string, getDataFn: () => GraphExportData | null) => void;
  onUnregisterFromExport?: (graphId: string) => void;
}) {
  const containerRef = useRef<HTMLDivElement | null>(null);
  const uplotRef = useRef<uPlot | null>(null);
  const chartCreatedRef = useRef(false);
  const animationRefs = useAnimationRefs();

  const normalizedSeries = normalizeDataSeries(newData);
  const [visibleSeries, setVisibleSeries] = useState(
    new Array(normalizedSeries.length).fill(true)
  );

  const [viewMode, setViewMode] = useState<"default" | "all" | "manual">(syncGraph?.viewMode ?? "default");
  const [isLiveMode, setIsLiveMode] = useState(syncGraph?.isLiveMode ?? true);
  const [selectedTimeWindow, setSelectedTimeWindow] = useState<number | "all">(
    syncGraph?.timeWindow ?? config.defaultTimeWindow ?? 30 * 60 * 1000
  );

  const [cursorValue, setCursorValue] = useState<number | null>(null);
  const [cursorValues, setCursorValues] = useState<(number | null)[]>(
    new Array(normalizedSeries.length).fill(null)
  );

  const startTimeRef = useRef<number | null>(null);
  const manualScaleRef = useRef<{ x: { min: number; max: number }; y: { min: number; max: number } } | null>(null);
  const lastProcessedCountRef = useRef(0);

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

  // Lazy slice series for window
  const lazyData = React.useMemo(() => {
    const startTime = startTimeRef.current ?? Date.now() - (selectedTimeWindow === "all" ? Infinity : selectedTimeWindow);
    const endTime = startTime + (selectedTimeWindow === "all" ? Infinity : selectedTimeWindow);

    return normalizedSeries.map((series) =>
      sliceSeriesForWindow(series, startTime, endTime)
    );
  }, [normalizedSeries, selectedTimeWindow]);

  const enhancedConfig = React.useMemo(() => mergeConfigWithVisibleLines(config, lazyData, visibleSeries), [
    config,
    lazyData,
    visibleSeries,
  ]);

  // Update cursor values when series length changes
  useEffect(() => {
    setCursorValues(new Array(normalizedSeries.length).fill(null));
  }, [normalizedSeries.length]);

  // Toggle series visibility
  const toggleSeries = useCallback((index: number) => {
    setVisibleSeries((prev) => {
      const newVisibility = [...prev];
      const currentlyVisible = newVisibility.filter((v) => v);
      const wouldHideAll = newVisibility[index] && currentlyVisible.length === 1;
      if (wouldHideAll) return prev;
      newVisibility[index] = !newVisibility[index];
      return newVisibility;
    });
  }, []);

  // Live & historical modes
  const liveMode = useLiveMode({
    newData: lazyData,
    uplotRef,
    config: enhancedConfig,
    animationRefs,
    viewMode,
    selectedTimeWindow,
    startTimeRef,
    updateYAxisScale: () => {}, // handled by uPlot internal lazy updates
    lastProcessedCountRef,
    chartCreatedRef,
  });

  const historicalMode = useHistoricalMode({
    newData: lazyData,
    uplotRef,
    animationRefs,
    getCurrentLiveEndTimestamp: liveMode.getCurrentLiveEndTimestamp,
    updateYAxisScale: () => {},
    lastProcessedCountRef,
    manualScaleRef,
  });

  const handleTimeWindowChange = useCallback(
    (newWindow: number | "all") => {
      stopAnimations(animationRefs);
      setSelectedTimeWindow(newWindow);
      if (newWindow === "all") {
        setViewMode("all");
        if (!isLiveMode) {
          setIsLiveMode(true);
          historicalMode.switchToLiveMode();
        }
        liveMode.handleLiveTimeWindow(newWindow);
      } else {
        setViewMode("default");
        if (isLiveMode) {
          liveMode.handleLiveTimeWindow(newWindow);
        } else {
          historicalMode.handleHistoricalTimeWindow(newWindow);
        }
      }
    },
    [animationRefs, isLiveMode, liveMode, historicalMode]
  );

  // Effects for chart creation and updates
  useBigGraphEffects({
    containerRef,
    uplotRef,
    startTimeRef,
    manualScaleRef,
    lastProcessedCountRef,
    animationRefs,
    handlerRefs,
    chartCreatedRef,
    newData: lazyData,
    unit,
    renderValue,
    config: enhancedConfig,
    graphId,
    syncGraph,
    viewMode,
    isLiveMode,
    selectedTimeWindow,
    visibleSeries,
    setSelectedTimeWindow,
    setViewMode,
    setIsLiveMode,
    setCursorValue,
    setCursorValues,
    liveMode,
    historicalMode,
    colors,
    handleTimeWindowChangeInternal: handleTimeWindowChange,
  });

  // Get series display value
  const getSeriesDisplayValue = useCallback(
    (index: number) => {
      const series = lazyData[index];
      if (!series) return null;
      return lazyData.length > 1
        ? cursorValues[index] ?? series.newData?.current?.value
        : cursorValue ?? series.newData?.current?.value;
    },
    [lazyData, cursorValues, cursorValue]
  );

  const displayValue = getSeriesDisplayValue(0);
  const defaultColors = ["#3b82f6", "#ef4444", "#10b981", "#f59e0b", "#8b5cf6"];

  return (
    <div className="h-[50vh] w-full">
      <div className="flex h-full w-full flex-col overflow-hidden rounded-3xl border border-gray-200 bg-white shadow">
        {/* Header */}
        <div className="flex items-center justify-between pt-4 pr-5 pb-6 pl-6">
          <div className="mt-1 flex items-center gap-4">
            <Icon name={unit ? getUnitIcon(unit) : "lu:TrendingUp"} className="size-6 text-gray-600" />
            <h2 className="text-2xl leading-none font-bold text-gray-900">{enhancedConfig.title}</h2>
          </div>

          {/* Series values / toggles */}
          <div className="flex items-center gap-4">
            {lazyData.length === 1 && (
              <ControlCard className="rounded-md px-4 py-3">
                <div className="flex items-center gap-2 text-base text-gray-600">
                  <span className="font-mono leading-none font-bold text-gray-900">
                    {formatDisplayValue(displayValue, renderValue)}
                  </span>
                  <span className="leading-none text-gray-500">{renderUnitSymbol(unit)}</span>
                </div>
              </ControlCard>
            )}

            {lazyData.length > 1 && (
              <div className="flex items-center gap-2">
                {lazyData.map((series, index) => {
                  const seriesColor = series.color ?? defaultColors[index % defaultColors.length];
                  const val = getSeriesDisplayValue(index);
                  const formatted = formatDisplayValue(val, renderValue);
                  const isLastVisible = visibleSeries[index] && visibleSeries.filter(Boolean).length === 1;

                  return (
                    <TouchButton
                      key={index}
                      onClick={() => !isLastVisible && toggleSeries(index)}
                      className={`rounded-md px-4 py-1.5 text-sm transition-all ${
                        visibleSeries[index] ? "text-white shadow-md" : "bg-gray-100 text-gray-500 hover:bg-gray-200"
                      } ${isLastVisible ? "cursor-not-allowed" : ""}`}
                      style={{ backgroundColor: visibleSeries[index] ? seriesColor : undefined }}
                      title={`${series.title || `S${index + 1}`}: ${formatted} ${renderUnitSymbol(unit)}`}
                    >
                      <div className="flex flex-col items-center">
                        <span className="text-xs font-medium">{series.title || `S${index + 1}`}</span>
                        <span className={`font-mono leading-none font-bold ${visibleSeries[index] ? "text-white" : "text-gray-500"}`}>
                          {formatted} {renderUnitSymbol(unit)}
                        </span>
                      </div>
                    </TouchButton>
                  );
                })}
              </div>
            )}
          </div>
        </div>

        <div className="-mt-2 px-6">
          <div className="h-px bg-gray-200"></div>
        </div>

        <div className="flex-1 overflow-hidden rounded-b-3xl pt-4">
          <div ref={containerRef} className="h-full w-full overflow-hidden" style={{ backgroundColor: colors.background }} />
        </div>
      </div>
    </div>
  );
}
