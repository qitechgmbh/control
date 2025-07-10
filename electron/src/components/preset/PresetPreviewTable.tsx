import React, { Fragment } from "react";
import { Preset } from "@/lib/preset/preset";
import { renderUnitSymbol, renderUnitSyntax, Unit } from "@/control/units";

export type PresetPreviewEntry<T> = {
  name: string;
  unit: Unit;
  renderValue: (preset: Preset<T>) => string | undefined;
};

export type PresetPreviewTableProps<T> = {
  preset: Preset<T>;
  entries: PresetPreviewEntry<T>[];
};

export function PresetPreviewTable<T>({
  preset,
  entries,
}: PresetPreviewTableProps<T>) {
  return (
    <div className="grid grid-cols-3">
      {entries.map((entry) => {
        const value = entry.renderValue(preset);
        return (
          <Fragment key={entry.name}>
            <div>{entry.name}</div>
            <div>=</div>
            <div>
              {value === undefined
                ? "N/A"
                : renderUnitSyntax(value, entry.unit)}{" "}
              {renderUnitSymbol(entry.unit)}
            </div>
          </Fragment>
        );
      })}
    </div>
  );
}
