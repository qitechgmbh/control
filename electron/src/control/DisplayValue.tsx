"use client";

import React from "react";
import { Icon, IconName } from "@/components/Icon";
import { Unit, renderUnitSymbol, renderValueToReactNode } from "./units";
import { cva } from "class-variance-authority";

type Props = {
  /** Label above the value (e.g. "Temperature") */
  title: string;
  /** Icon shown next to the title */
  icon: IconName;
  /** Optional unit (°C, W, etc.) */
  unit?: Unit;
  /** Value from backend (can be null/undefined) */
  value?: number | null;
  /** Function to render the value (e.g. v => v.toFixed(1)) */
  renderValue: (v: number) => string;
  /** Optional extra class name for layout */
  className?: string;
};

const containerStyle = cva(
  "flex flex-col justify-center rounded-2xl border border-neutral-200 bg-white dark:bg-neutral-900 px-6 py-4 shadow-sm transition-colors duration-300 ease-in-out",
);

/**
 * DisplayValue
 *
 * Read-only display tile matching the visual style of EditValue
 * (without edit icons, sliders, or graphs).
 *
 * Shows:
 * - A title and icon (e.g. "Temperature")
 * - A large monospaced value with unit
 * - Smoothly updates when the value prop changes
 */
export function DisplayValue({
  title,
  icon,
  unit,
  value,
  renderValue,
  className,
}: Props) {
  const valueIsDefined = value !== undefined && value !== null;

  return (
    <div className={containerStyle({ class: className })}>
      {/* Header */}
      <div className="mb-1 flex flex-row items-center gap-2 text-sm text-neutral-600 dark:text-neutral-400">
        <Icon name={icon} className="size-5" />
        <span>{title}</span>
      </div>

      {/* Value */}
      <div className="flex flex-row items-end gap-1">
        <span className="font-mono text-5xl leading-none font-bold text-neutral-900 dark:text-neutral-100">
          {valueIsDefined
            ? renderValueToReactNode(value, unit, renderValue)
            : "--"}
        </span>
        {unit && (
          <span className="pb-1 text-2xl font-light text-neutral-500 dark:text-neutral-400">
            {renderUnitSymbol(unit)}
          </span>
        )}
      </div>
    </div>
  );
}
