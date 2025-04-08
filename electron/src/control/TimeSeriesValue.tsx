import { Icon, IconName } from "@/components/Icon";
import React from "react";
import { getUnitIcon, renderUnitSymbol, renderUnitSyntax, Unit } from "./units";
import { Skeleton } from "@/components/ui/skeleton";
import { Label } from "./Label";
import { TimeSeries } from "@/lib/timeseries";

type Props = {
  label: string;
  unit?: Unit;
  timeseries: TimeSeries;
  icon?: IconName;
  renderValue?: (value: number) => string;
};

type AllowedT = number | boolean;

function _TimeSeriesValue({
  unit,
  timeseries,
  icon,
  label,
  renderValue,
}: Props) {
  const value = timeseries.current?.value;
  const _renderValue = (value: number | undefined) => {
    if (value === undefined) {
      return "n/a";
    }
    if (renderValue) {
      return renderValue(value);
    }
    return value.toString();
  };

  return (
    <div className="bg-red flex flex-row items-center gap-4">
      <Label label={label}>
        <div className="flex flex-row items-center gap-4">
          <Icon
            name={icon ?? (unit ? getUnitIcon(unit) : undefined)}
            className="size-7"
          />
          <div className="flex flex-row items-center gap-2">
            <span className="font-mono text-4xl font-bold">
              {renderUnitSyntax(_renderValue(value), unit)}
            </span>
            <span>{renderUnitSymbol(unit)}</span>
          </div>
        </div>
      </Label>
      <Skeleton className="h-16 flex-1 bg-neutral-100"></Skeleton>
    </div>
  );
}

export function TimeSeriesValueNumeric(props: Props) {
  return <_TimeSeriesValue {...props} />;
}
