"use client";

import React from "react";
import { Icon, IconName } from "@/components/Icon";
import { Separator } from "@/components/ui/separator";
import {
  getUnitIcon,
  renderValueToReactNode,
  renderUnitSymbol,
  renderUnitSymbolLong,
  Unit,
} from "./units";
import { cva } from "class-variance-authority";

type Props = {
  unit?: Unit;
  value?: number | null;
  title: string;
  description?: string;
  icon?: IconName;
  renderValue: (value: number) => string;
  inverted?: boolean;
  minLabel?: string;
  maxLabel?: string;
  className?: string;
};

const containerStyle = cva(
  "flex flex-col gap-6 rounded-2xl p-6 shadow-2xl bg-white dark:bg-neutral-900 transition-colors duration-300 ease-in-out",
);

const valueRowStyle = cva("flex flex-row items-center justify-center gap-2", {
  variants: {
    inverted: {
      true: "flex-row-reverse",
    },
  },
});

/**
 * DisplayValue Component
 *
 * Read-only version of EditValue:
 * - Same style and layout
 * - No editing, numpad, or sliders
 * - Reactive updates from parent state/backend
 * - Graceful handling of undefined/null values
 */
export function DisplayValue({
  unit,
  value,
  renderValue,
  title,
  description,
  icon,
  inverted,
  className,
}: Props) {
  const valueIsDefined = value !== undefined && value !== null;
  const formattedValue = valueIsDefined ? renderValue(value!) : "--";

  return (
    <div className={containerStyle({ className })}>
      {/* Header section */}
      <div className="flex flex-row items-center gap-2">
        <Icon
          name={icon ?? (unit ? getUnitIcon(unit) : "lu:Gauge")}
          className="size-6"
        />
        <span className="text-lg font-medium">{title}</span>
      </div>

      {description && (
        <span className="text-sm text-neutral-500">{description}</span>
      )}

      <Separator />

      {/* Value row */}
      <div className={valueRowStyle({ inverted })}>
        <span className="font-mono text-5xl font-bold">
          {renderValueToReactNode(value, unit, renderValue)}
        </span>
        {unit && (
          <span className="text-2xl text-neutral-500">
            {renderUnitSymbol(unit)}
          </span>
        )}
      </div>

      {unit && (
        <span className="text-center text-sm text-gray-400 uppercase">
          {renderUnitSymbolLong(unit)}
        </span>
      )}
    </div>
  );
}
