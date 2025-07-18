import React, { Fragment } from "react";
import { PresetData, PresetSchema } from "@/lib/preset/preset";
import { renderUnitSymbol, renderUnitSyntax, Unit } from "@/control/units";

export const previewSeparator = undefined;

type PreviewSeparator = typeof previewSeparator;

type PresetPreviewEntry<T extends PresetSchema> = {
  name: string;
  unit?: Unit;
  renderValue: (data: PresetData<T>) => string | undefined;
};

export type PresetPreviewEntries<T extends PresetSchema> = (
  | PresetPreviewEntry<T>
  | PreviewSeparator
)[];

export type PresetPreviewTableProps<T extends PresetSchema> = {
  data: PresetData<T>;
  entries: PresetPreviewEntries<T>;
};

export function PresetPreviewTable<T extends PresetSchema>({
  data,
  entries,
}: PresetPreviewTableProps<T>) {
  return (
    <div className="grid grid-cols-[60%_10%_30%]">
      {entries.map((entry, i) => {
        if (entry === previewSeparator) {
          return (
            <Fragment key={i}>
              <div className="h-[1em]" />
              <div />
              <div />
            </Fragment>
          );
        }

        const value = entry.renderValue(data);
        return (
          <Fragment key={entry.name}>
            <div>{entry.name}</div>
            <div>=</div>
            <div>
              {value === undefined
                ? "N/A"
                : renderUnitSyntax(value, entry.unit)}
              {entry.unit && " " + renderUnitSymbol(entry.unit)}
            </div>
          </Fragment>
        );
      })}
    </div>
  );
}
