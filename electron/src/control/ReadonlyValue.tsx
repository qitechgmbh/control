import { renderUndefinedValue, renderUnitSymbol, Unit } from "./units";
import { Icon } from "@/components/Icon";
import { Separator } from "@/components/ui/separator";
import React from "react";

type Props = {
  unit?: Unit;
  value?: number;
  renderValue: (value: number) => string;
};

export function ReadOnlyValue({ unit, value, renderValue }: Props) {
  return (
    <div className="border-input bg-background flex w-min flex-col items-center gap-4 rounded-md border px-6 py-4 shadow-sm">
      <div className="flex flex-row items-center gap-2">
        <span className="font-mono text-4xl font-bold">
          {renderUndefinedValue(value, unit, renderValue)}
        </span>
        <span>{renderUnitSymbol(unit)}</span>
      </div>
    </div>
  );
}
