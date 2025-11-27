import { Icon, IconName } from "@/components/Icon";
import React, { useState, useMemo } from "react";
import {
  getUnitIcon,
  renderValueToReactNode,
  renderUnitSymbol,
  Unit,
} from "./units";
import { Label } from "./Label";
import { TimeSeries, TimeSeriesValue } from "@/lib/timeseries";

type TimeframeOption = {
  label: string;
  value: number; // in milliseconds
};

export const TIMEFRAME_OPTIONS: TimeframeOption[] = [
  { label: "1 min", value: 1 * 60 * 1000 },
  { label: "5 min", value: 5 * 60 * 1000 },
  { label: "10 min", value: 10 * 60 * 1000 },
  { label: "30 min", value: 30 * 60 * 1000 },
  { label: "1 hour", value: 60 * 60 * 1000 },
];

type Props = {
  label: string;
  unit?: Unit;
  timeseries: TimeSeries;
  icon?: IconName;
  renderValue?: (value: number) => string;
  defaultTimeframe?: number; // in milliseconds, defaults to 5 minutes
  /** Optional externally-controlled timeframe (ms). When provided, component becomes controlled. */
  timeframe?: number;
  /** Optional callback when timeframe is changed via this component's selector */
  onTimeframeChange?: (t: number) => void;
  /** When true, hide the built-in timeframe selector (useful when sharing a selector outside) */
  hideSelector?: boolean;
};

export function MinMaxValue({
  unit,
  timeseries,
  icon,
  label,
  renderValue,
  defaultTimeframe = 5 * 60 * 1000, // 5 minutes default
  timeframe,
  onTimeframeChange,
  hideSelector = false,
}: Props) {
  const [internalTimeframe, setInternalTimeframe] = useState(defaultTimeframe);
  const selectedTimeframe = timeframe ?? internalTimeframe;

  // Calculate min/max from the timeseries using the selected timeframe
  const { min, max, hasData } = useMemo(() => {
    // Use short series for timeframes <= 5 seconds, otherwise use long series
    // Short series typically has ~5 second retention with high resolution
    // Long series typically has ~1 hour retention with lower resolution
    const series =
      selectedTimeframe <= 5000 ? timeseries.short : timeseries.long;

    // Manually calculate min/max to exclude zero values (which might be invalid readings)
    const cutoffTime = series.lastTimestamp - selectedTimeframe;
    let min = Number.POSITIVE_INFINITY;
    let max = Number.NEGATIVE_INFINITY;
    let hasValidData = false;
    let dataCount = 0;

    const { values: raw, index, size, validCount } = series;

    const startIdx = validCount < size ? 0 : index;

    for (let i = 0; i < validCount; i++) {
      const idx = (startIdx + i) % size;
      const val = raw[idx];

      if (
        val &&
        val.timestamp > 0 &&
        val.timestamp >= cutoffTime &&
        val.value > 0
      ) {
        hasValidData = true;
        dataCount++;
        if (val.value < min) min = val.value;
        if (val.value > max) max = val.value;
      }
    }

    if (!hasValidData) {
      return { min: 0, max: 0, hasData: false };
    }

    return { min, max, hasData: true };
  }, [
    timeseries.short,
    timeseries.long,
    timeseries.current,
    selectedTimeframe,
  ]);

  return (
    <div className="flex w-full flex-col gap-3">
      <Label label={label}>
        <div className="flex flex-row items-center gap-6">
          {/* Min Value */}
          <div className="flex flex-row items-center gap-2">
            <Icon
              name={icon ?? (unit ? getUnitIcon(unit) : undefined)}
              className="size-5 text-blue-500"
            />
            <div className="flex flex-col">
              <span className="text-xs text-gray-500">Min</span>
              <div className="flex flex-row items-center gap-1">
                <span className="font-mono text-xl font-bold">
                  {hasData
                    ? renderValueToReactNode(min, unit, renderValue)
                    : "-"}
                </span>
                {hasData && (
                  <span className="text-sm">{renderUnitSymbol(unit)}</span>
                )}
              </div>
            </div>
          </div>

          {/* Max Value */}
          <div className="flex flex-row items-center gap-2">
            <Icon
              name={icon ?? (unit ? getUnitIcon(unit) : undefined)}
              className="size-5 text-red-500"
            />
            <div className="flex flex-col">
              <span className="text-xs text-gray-500">Max</span>
              <div className="flex flex-row items-center gap-1">
                <span className="font-mono text-xl font-bold">
                  {hasData
                    ? renderValueToReactNode(max, unit, renderValue)
                    : "-"}
                </span>
                {hasData && (
                  <span className="text-sm">{renderUnitSymbol(unit)}</span>
                )}
              </div>
            </div>
          </div>
        </div>
      </Label>

      {/* Timeframe Selector (hidden when sharing selector externally) */}
      {!hideSelector && (
        <div className="flex flex-row flex-wrap gap-2">
          {TIMEFRAME_OPTIONS.map((option) => (
            <button
              key={option.value}
              onClick={() => {
                if (onTimeframeChange) onTimeframeChange(option.value);
                else setInternalTimeframe(option.value);
              }}
              className={`rounded-md px-3 py-1 text-sm transition-colors ${
                selectedTimeframe === option.value
                  ? "bg-blue-500 text-white"
                  : "bg-gray-200 text-gray-700 hover:bg-gray-300"
              }`}
            >
              {option.label}
            </button>
          ))}
        </div>
      )}
    </div>
  );
}
