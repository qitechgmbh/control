import React, { useRef, useState, useCallback } from "react";
import uPlot from "uplot";
import "uplot/dist/uPlot.min.css";
import { renderUnitSymbol, getUnitIcon } from "@/control/units";
import { Icon } from "@/components/Icon";
import { BigGraphProps } from "./types";
import { GraphExportData } from "./excelExport";
import { DEFAULT_COLORS } from "./constants";
import { useAnimationRefs, stopAnimations } from "./animation";
import { HandlerRefs } from "./handlers";
import { useLiveMode } from "./liveMode";
import { useHistoricalMode } from "./historicalMode";
import { useBigGraphEffects } from "./useBigGraphEffects";

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

  const updateYAxisScale = useCallback(
    (timestamps: number[], values: number[], xMin?: number, xMax?: number) => {
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
    },
    [config.lines, viewMode],
  );

  // Initialize live mode handlers
  const liveMode = useLiveMode({
    newData,
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

  // Initialize historical mode handlers
  const historicalMode = useHistoricalMode({
    newData,
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
      } else {
        setViewMode("default");
      }

      if (isLiveMode) {
        liveMode.handleLiveTimeWindow(newTimeWindow);
      } else {
        historicalMode.handleHistoricalTimeWindow(newTimeWindow);
      }

      // Notify parent about time window change
      if (!isSync && syncGraph?.onTimeWindowChange) {
        syncGraph.onTimeWindowChange(graphId, newTimeWindow);
      }
    },
    [
      animationRefs,
      isLiveMode,
      liveMode.handleLiveTimeWindow,
      historicalMode.handleHistoricalTimeWindow,
      syncGraph,
      graphId,
    ],
  );

  // Use the extracted useEffect hooks
  useBigGraphEffects({
    // Refs
    containerRef,
    uplotRef,
    chartCreatedRef,
    startTimeRef,
    manualScaleRef,
    lastProcessedCountRef,
    animationRefs,
    handlerRefs,

    // Props
    newData,
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
