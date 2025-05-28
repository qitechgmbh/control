import { Icon, IconName } from "@/components/Icon";
import React from "react";
import {
  getUnitIcon,
  renderUndefinedValue,
  renderUnitSymbol,
  Unit,
} from "./units";
import { Label } from "./Label";
import { TimeSeries } from "@/lib/timeseries";
import { MiniGraph } from "@/helpers/mini_graph";

type Props = {
  label: string;
  unit?: Unit;
  timeseries: TimeSeries;
  icon?: IconName;
  renderValue?: (value: number) => string;
};

function _TimeSeriesValue({
  unit,
  timeseries,
  icon,
  label,
  renderValue,
}: Props) {
  const value = timeseries.current?.value;

  return (
    <div className="bg-red flex flex-row items-center gap-4">
      <div className="flex-1">
        <Label label={label}>
          <div className="flex flex-row items-center gap-4">
            <Icon
              name={icon ?? (unit ? getUnitIcon(unit) : undefined)}
              className="size-7"
            />
            <div className="flex flex-row items-center gap-2">
              <span className="font-mono text-4xl font-bold">
                {renderUndefinedValue(value, unit, renderValue)}
              </span>
              <span>{renderUnitSymbol(unit)}</span>
            </div>
          </div>
        </Label>
      </div>
      <MiniGraph newData={timeseries} />
    </div>
  );
}

export function TimeSeriesValueNumeric(props: Props) {
  return <_TimeSeriesValue {...props} />;
}
