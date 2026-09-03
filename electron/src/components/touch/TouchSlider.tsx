import * as React from "react";
import * as SliderPrimitive from "@radix-ui/react-slider";

import { cn } from "@/lib/utils";
import { Icon } from "../Icon";
import { renderUnitSymbol, renderUnitSyntax, Unit } from "@/control/units";
import { cva } from "class-variance-authority";

type Props = {
  minLabel?: string;
  maxLabel?: string;
  renderValue?: (value: number) => string;
  unit?: Unit;
} & React.ComponentProps<typeof SliderPrimitive.Root>;

export function TouchSlider({
  className,
  defaultValue,
  value,
  min = 0,
  max = 100,
  minLabel: _minLabel,
  maxLabel: _maxLabel,
  unit,
  renderValue,
  inverted,
  ...props
}: Props) {
  const _values = React.useMemo(
    () =>
      Array.isArray(value)
        ? value
        : Array.isArray(defaultValue)
          ? defaultValue
          : [min, max],
    [value, defaultValue, min, max],
  );

  const minLabel = _minLabel ?? "MIN";
  const maxLabel = _maxLabel ?? "MAX";

  const labelStyle = cva(
    "flex flex-row h-0 justify-between text-xs text-gray-400",
    {
      variants: {
        inverted: {
          true: "flex-row-reverse",
        },
      },
    },
  );

  return (
    <div className="flex flex-col-reverse">
      <div className="relative -top-4">
        <div
          className={labelStyle({
            inverted,
          })}
        >
          <span>{minLabel}</span>
          <span>{maxLabel}</span>
        </div>
      </div>
      <SliderPrimitive.Root
        data-slot="slider"
        defaultValue={defaultValue}
        value={value}
        min={min}
        max={max}
        inverted={inverted}
        className={cn(
          "relative flex w-full touch-none items-center py-5 select-none data-[disabled]:opacity-50 data-[orientation=vertical]:h-full data-[orientation=vertical]:min-h-44 data-[orientation=vertical]:w-auto data-[orientation=vertical]:flex-col",
          className,
        )}
        {...props}
      >
        <SliderPrimitive.Track
          data-slot="slider-track"
          className={cn(
            "relative grow overflow-hidden rounded-full bg-gray-200 data-[orientation=horizontal]:h-4 data-[orientation=horizontal]:w-full data-[orientation=vertical]:h-full data-[orientation=vertical]:w-1.5 dark:bg-gray-800",
          )}
        >
          <SliderPrimitive.Range
            data-slot="slider-range"
            className={cn(
              "absolute bg-black data-[orientation=horizontal]:h-full data-[orientation=vertical]:w-full dark:bg-gray-50",
            )}
          />
        </SliderPrimitive.Track>
        {Array.from({ length: _values.length }, (_, index) => (
          <SliderPrimitive.Thumb
            data-slot="slider-thumb"
            key={index}
            className="block size-14 shrink-0 rounded-2xl bg-black shadow-sm ring-gray-950/50 transition-[color,box-shadow] focus-visible:ring-8 focus-visible:outline-hidden disabled:pointer-events-none disabled:opacity-50 dark:border-gray-50 dark:border-gray-800 dark:bg-gray-950 dark:ring-gray-300/50"
          >
            <div className="flex h-full w-full items-center justify-center">
              <Icon name="lu:Pointer" className="text-white" />
            </div>
          </SliderPrimitive.Thumb>
        ))}
      </SliderPrimitive.Root>
      <div className="relative">
        <div
          className={labelStyle({
            inverted,
          })}
        >
          <span>
            {renderUnitSyntax(renderValue?.(min), unit)}{" "}
            {renderUnitSymbol(unit)}
          </span>
          <span>
            {renderUnitSyntax(renderValue?.(max), unit)}{" "}
            {renderUnitSymbol(unit)}
          </span>
        </div>
      </div>
    </div>
  );
}
