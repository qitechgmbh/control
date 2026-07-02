import React, { useRef, useState, useCallback, useEffect } from "react";
import type { IChartApi } from "lightweight-charts";
import { renderUnitSymbol, getUnitIcon } from "@/control/units";
import { Icon } from "@/components/Icon";
import { BigGraphProps, SeriesData, SeriesRefs } from "./types";
import { GraphExportData } from "./excelExport";
import { DEFAULT_COLORS } from "./constants";
import { normalizeDataSeries, formatDisplayValue } from "./dataHelpers";
import { useLiveMode } from "./liveMode";
import { useHistoricalMode } from "./historicalMode";
import { useBigGraphEffects } from "./useBigGraphEffects";
import { TouchButton } from "../touch/TouchButton";
import { ControlCard } from "@/control/ControlCard";
import { TargetDashOverlay } from "./TargetDashOverlay";

// Collects lines connected to visible series for rendering
const getVisibleLines = (data: any, visibleSeries: boolean[]) => {
  const visibleLines: any[] = [];

  if (Array.isArray(data)) {
    data.forEach((series, index) => {
      if (visibleSeries[index] && series.lines && Array.isArray(series.lines)) {
        const seriesLines = series.lines.map((line: any) => ({
          ...line,
          color: line.color || series.color,
          dash: line.dash || [5, 5],
          seriesIndex: index,
          seriesTitle: series.title,
        }));
        visibleLines.push(...seriesLines);
      }
    });
  } else {
    if (visibleSeries[0] && data.lines && Array.isArray(data.lines)) {
      const seriesLines = data.lines.map((line: any) => ({
        ...line,
        color: line.color || data.color,
        dash: line.dash || [5, 5],
        seriesIndex: 0,
        seriesTitle: data.title,
      }));
      visibleLines.push(...seriesLines);
    }
  }

  return visibleLines;
};

// Merges configuration lines with lines from visible series
const mergeConfigWithVisibleLines = (
  config: any,
  data: any,
  visibleSeries: boolean[],
) => {
  const configLines = config.lines || [];
  const seriesLines = getVisibleLines(data, visibleSeries);

  // Combine and remove duplicate lines based on label and series connection
  const allLines = [...configLines, ...seriesLines];
  const uniqueLines = allLines.filter((line, index, arr) => {
    return (
      arr.findIndex((l) => {
        if (line.seriesIndex !== undefined && l.seriesIndex !== undefined) {
          return l.label === line.label && l.seriesIndex === line.seriesIndex;
        }
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
  chartRefOut,
  containerRefOut,
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
  const chartRef = useRef<IChartApi | null>(null);
  const seriesRefs = useRef<SeriesRefs>({ dataSeries: [], lineSeries: [] });
  const chartCreatedRef = useRef(false);

  // State for series visibility
  const normalizedSeries = normalizeDataSeries(newData);
  const latestExportContextRef = useRef({
    normalizedSeries,
    config,
    unit,
    renderValue,
  });
  const [visibleSeries, setVisibleSeries] = useState<boolean[]>(
    new Array(normalizedSeries.length).fill(true),
  );

  // Keep latest export context in a ref so export callbacks always read fresh data.
  useEffect(() => {
    latestExportContextRef.current = {
      normalizedSeries,
      config,
      unit,
      renderValue,
    };
  }, [normalizedSeries, config, unit, renderValue]);

  // Enhanced configuration with visible series lines
  const enhancedConfig = React.useMemo(() => {
    return mergeConfigWithVisibleLines(config, newData, visibleSeries);
  }, [config, newData, visibleSeries]);

  // State for managing chart visibility and modes
  const [viewMode, setViewMode] = useState<"default" | "all" | "manual">(
    syncGraph?.viewMode ?? "default",
  );
  const [isLiveMode, setIsLiveMode] = useState(syncGraph?.isLiveMode ?? true);
  const [selectedTimeWindow, setSelectedTimeWindow] = useState<number | "all">(
    syncGraph?.timeWindow ?? enhancedConfig.defaultTimeWindow ?? 30 * 60 * 1000,
  );

  // Cursor state for displaying values
  const [cursorValue, setCursorValue] = useState<number | null>(null);
  const [cursorValues, setCursorValues] = useState<(number | null)[]>(
    new Array(normalizedSeries.length).fill(null),
  );

  const startTimeRef = useRef<number | null>(null);
  const manualScaleRef = useRef<{
    x: { min: number; max: number };
  } | null>(null);
  const suppressRangeEventRef = useRef(false);
  const isUserInteractingRef = useRef(false);
  const lastProcessedCountRef = useRef(0);

  const colors = {
    primary: enhancedConfig.colors?.primary ?? DEFAULT_COLORS.primary,
    grid: enhancedConfig.colors?.grid ?? DEFAULT_COLORS.grid,
    axis: enhancedConfig.colors?.axis ?? DEFAULT_COLORS.axis,
    background: enhancedConfig.colors?.background ?? DEFAULT_COLORS.background,
  };

  // Update cursor values when series length changes
  useEffect(() => {
    setCursorValues(new Array(normalizedSeries.length).fill(null));
  }, [normalizedSeries.length]);

  // FAST VISIBILITY UPDATE FUNCTION
  const updateSeriesVisibility = useCallback(() => {
    if (!chartRef.current || !visibleSeries) return;

    seriesRefs.current.dataSeries.forEach((series, index) => {
      series.applyOptions({ visible: visibleSeries[index] });
    });
  }, [visibleSeries]);

  // SEPARATE EFFECT FOR VISIBILITY UPDATES ONLY
  useEffect(() => {
    if (chartRef.current && chartCreatedRef.current) {
      updateSeriesVisibility();
    }
  }, [visibleSeries, updateSeriesVisibility]);

  // Register export functionality for the graph
  useEffect(() => {
    if (onRegisterForExport) {
      // Register each series as a separate export entry
      normalizedSeries.forEach((_, index) => {
        const seriesId = `${graphId}-series-${index}`;

        const getSeriesExportData = (): GraphExportData | null => {
          const current = latestExportContextRef.current;
          const series = current.normalizedSeries[index];
          if (!series?.newData) return null;

          // Create export data for this specific series only
          const seriesLines = series.lines || [];
          const exportConfig = {
            ...current.config,
            title: current.config.title, // Keep original graph title
            lines: [...(current.config.lines || []), ...seriesLines],
          };

          // Create a single-series data structure for this series
          const singleSeriesData: SeriesData = {
            newData: series.newData,
            title: series.title || `Series ${index + 1}`,
            color: series.color,
            lines: series.lines,
          };

          return {
            config: exportConfig,
            data: singleSeriesData, // Return ONLY this specific series
            unit: current.unit,
            renderValue: current.renderValue,
          };
        };

        onRegisterForExport(seriesId, getSeriesExportData);
      });

      return () => {
        if (onUnregisterFromExport) {
          // Clean up all series registrations
          normalizedSeries.forEach((_, index) => {
            const seriesId = `${graphId}-series-${index}`;
            onUnregisterFromExport(seriesId);
          });
        }
      };
    }
  }, [
    onRegisterForExport,
    onUnregisterFromExport,
    graphId,
    normalizedSeries.length,
  ]);

  // Initialize live mode handlers
  const liveMode = useLiveMode({
    newData,
    chartRef,
    seriesRefs,
    viewMode,
    selectedTimeWindow,
    startTimeRef,
    suppressRangeEventRef,
    lastProcessedCountRef,
    chartCreatedRef,
  });

  // Initialize historical mode handlers
  const historicalMode = useHistoricalMode({
    newData,
    chartRef,
    suppressRangeEventRef,
    getCurrentLiveEndTimestamp: liveMode.getCurrentLiveEndTimestamp,
    lastProcessedCountRef,
    manualScaleRef,
  });

  // Handles time window changes
  const handleTimeWindowChangeInternal = useCallback(
    (newTimeWindow: number | "all", isSync: boolean = false) => {
      setSelectedTimeWindow(newTimeWindow);

      if (!chartRef.current) {
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
      isLiveMode,
      liveMode.handleLiveTimeWindow,
      historicalMode.handleHistoricalTimeWindow,
      historicalMode.switchToLiveMode,
      syncGraph,
      graphId,
    ],
  );

  // Toggles visibility of a series
  const toggleSeries = useCallback((index: number) => {
    setVisibleSeries((prev) => {
      const newVisibility = [...prev];

      const currentlyVisible = newVisibility.filter((visible) => visible);
      const wouldHideAll =
        newVisibility[index] && currentlyVisible.length === 1;

      if (wouldHideAll) {
        return prev;
      }

      newVisibility[index] = !newVisibility[index];
      return newVisibility;
    });
  }, []);

  // Gets the display value for a series
  const getSeriesDisplayValue = useCallback(
    (seriesIndex: number) => {
      const series = normalizedSeries[seriesIndex];
      if (!series) return null;

      if (normalizedSeries.length > 1) {
        return cursorValues[seriesIndex] !== null
          ? cursorValues[seriesIndex]
          : series.newData?.current?.value;
      } else {
        return cursorValue !== null
          ? cursorValue
          : series.newData?.current?.value;
      }
    },
    [normalizedSeries, cursorValues, cursorValue],
  );

  // Applies effects for the graph
  useBigGraphEffects({
    containerRef,
    chartRef,
    chartRefOut,
    containerRefOut,
    seriesRefs,
    startTimeRef,
    manualScaleRef,
    suppressRangeEventRef,
    isUserInteractingRef,
    lastProcessedCountRef,
    chartCreatedRef,
    newData,
    unit,
    renderValue,
    config: enhancedConfig,
    graphId,
    syncGraph,
    onRegisterForExport,
    onUnregisterFromExport,
    viewMode,
    isLiveMode,
    selectedTimeWindow,
    visibleSeries, // Keep for initial creation
    showFromTimestamp: syncGraph?.showFromTimestamp,
    setSelectedTimeWindow,
    setViewMode,
    setIsLiveMode,
    setCursorValue,
    setCursorValues,
    liveMode,
    historicalMode,
    colors,
    handleTimeWindowChangeInternal,
  });

  const displayValue = getSeriesDisplayValue(0);

  const defaultColors = ["#3b82f6", "#ef4444", "#10b981", "#f59e0b", "#8b5cf6"];

  return (
    <div className="h-[50vh] w-full">
      <div className="flex h-full w-full flex-col overflow-hidden rounded-3xl border border-gray-200 bg-white shadow">
        <div className="flex shrink-0 items-center justify-between gap-4 pt-4 pr-5 pb-6 pl-6">
          <div className="mt-1 flex min-w-0 flex-1 items-center gap-4">
            <Icon
              name={unit ? getUnitIcon(unit) : "lu:TrendingUp"}
              className="size-6 text-gray-600"
            />

            <h2 className="text-2xl leading-none font-bold text-gray-900">
              {enhancedConfig.title}
            </h2>
          </div>

          <div className="flex shrink-0 items-center gap-4">
            {normalizedSeries.length === 1 && (
              <ControlCard className="rounded-lg border-gray-200/80 bg-gray-50/80 px-4 py-3 shadow-sm">
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

            {normalizedSeries.length > 1 && (
              <div className="flex items-center gap-2">
                <div className="flex items-center gap-1">
                  {normalizedSeries.map((series, index) => {
                    const seriesColor =
                      series.color ||
                      defaultColors[index % defaultColors.length];

                    const displayValue = getSeriesDisplayValue(index);
                    const formattedValue = formatDisplayValue(
                      displayValue,
                      renderValue,
                    );

                    const isLastVisible =
                      visibleSeries[index] &&
                      visibleSeries.filter((visible) => visible).length === 1;

                    const seriesLines = Array.isArray(newData)
                      ? newData[index]?.lines || []
                      : newData.lines || [];

                    return (
                      <TouchButton
                        key={index}
                        onClick={() => {
                          if (!isLastVisible) {
                            toggleSeries(index);
                          }
                        }}
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
        </div>

        <div className="-mt-2 px-6">
          <div className="h-px bg-gray-200"></div>
        </div>

        <div className="flex-1 overflow-hidden rounded-b-3xl pt-4">
          <div
            className={`relative h-full w-full overflow-hidden transition-opacity duration-100`}
            style={{ backgroundColor: colors.background }}
          >
            <div ref={containerRef} className="h-full w-full overflow-hidden" />
            <TargetDashOverlay
              chartRef={chartRef}
              seriesRefs={seriesRefs}
              containerRef={containerRef}
            />
          </div>
        </div>
      </div>
    </div>
  );
}
