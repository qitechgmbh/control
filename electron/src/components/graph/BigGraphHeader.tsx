import React from "react";
import { renderUnitSymbol, getUnitIcon, type Unit } from "@/control/units";
import { Icon } from "@/components/icons/Icon";
import { TouchButton } from "@/components/touch/TouchButton";
import { ControlCard } from "@/control/ControlCard";
import type { SeriesData } from "./types";

type BigGraphHeaderProps = {
  title: string;
  unit?: Unit;
  series: SeriesData[];
  /** Line color of each series. */
  colors: string[];
  visible: boolean[];
  /** Value shown per series: under the cursor, else the latest. */
  values: (number | null)[];
  renderValue?: (value: number) => string;
  onToggleSeries: (index: number) => void;
};

function formatValue(
  value: number | null | undefined,
  renderValue?: (value: number) => string,
) {
  if (value === null || value === undefined || !Number.isFinite(value)) {
    return "N/A";
  }
  return renderValue ? renderValue(value) : value.toFixed(3);
}

/**
 * Title and current value of a graph. With several series, each one is a button that
 * shows its value and toggles its visibility (the last visible one stays on).
 */
export function BigGraphHeader({
  title,
  unit,
  series,
  colors,
  visible,
  values,
  renderValue,
  onToggleSeries,
}: BigGraphHeaderProps) {
  const unitSymbol = renderUnitSymbol(unit);
  const visibleCount = visible.filter(Boolean).length;

  return (
    <div className="flex shrink-0 items-center justify-between gap-4 pt-4 pr-5 pb-6 pl-6">
      <div className="mt-1 flex min-w-0 flex-1 items-center gap-4">
        <Icon
          name={unit ? getUnitIcon(unit) : "lu:TrendingUp"}
          className="size-6 text-gray-600"
        />
        <h2 className="text-2xl leading-none font-bold text-gray-900">
          {title}
        </h2>
      </div>

      <div className="flex shrink-0 items-center gap-4">
        {series.length === 1 && (
          <ControlCard className="rounded-lg border-gray-200/80 bg-gray-50/80 px-4 py-3 shadow-sm">
            <div className="flex items-center gap-2 text-base text-gray-600">
              <span className="font-mono leading-none font-bold text-gray-900">
                {formatValue(values[0], renderValue)}
              </span>
              <span className="leading-none text-gray-500">{unitSymbol}</span>
            </div>
          </ControlCard>
        )}

        {series.length > 1 && (
          <div className="flex items-center gap-1">
            {series.map((s, index) => {
              const formatted = formatValue(values[index], renderValue);
              const isOn = visible[index];
              const isLastVisible = isOn && visibleCount === 1;
              const lines = s.lines || [];

              return (
                <TouchButton
                  key={index}
                  onClick={() => {
                    if (!isLastVisible) onToggleSeries(index);
                  }}
                  className={`rounded-md px-4 py-1.5 text-sm transition-all ${
                    isOn
                      ? "text-white shadow-md"
                      : "bg-gray-100 text-gray-500 hover:bg-gray-200"
                  } ${isLastVisible ? "cursor-not-allowed" : ""}`}
                  style={{ backgroundColor: isOn ? colors[index] : undefined }}
                  title={`${s.title || `Series ${index + 1}`}: ${formatted} ${unitSymbol || ""}${
                    lines.length > 0
                      ? `\nLines: ${lines.map((l) => l.label || `${l.value}`).join(", ")}`
                      : ""
                  }`}
                >
                  <div className="flex flex-col items-center">
                    <span className="text-xs font-medium">
                      {s.title || `S${index + 1}`}
                    </span>
                    <span
                      className={`font-mono leading-none font-bold ${
                        isOn ? "text-white" : "text-gray-500"
                      }`}
                    >
                      {formatted} {unitSymbol}
                    </span>
                  </div>
                </TouchButton>
              );
            })}
          </div>
        )}
      </div>
    </div>
  );
}
