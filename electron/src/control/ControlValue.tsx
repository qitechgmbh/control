import { Icon, IconName } from "@/components/Icon";
import React from "react";
import { getUnitIcon, Unit } from "./units";
import { Skeleton } from "@/components/ui/skeleton";
import { Label } from "./Label";

type Props<T> = {
  label: string;
  unit?: Unit;
  value: T;
  icon?: IconName;
  renderValue?: (value: T) => string;
};

type AllowedT = number | boolean;

function _ControlValue<T extends AllowedT>({
  unit,
  value,
  icon,
  label,
  renderValue,
}: Props<T>) {
  return (
    <div className="bg-red flex flex-row items-center gap-4">
      <Label label={label}>
        <div className="flex flex-row items-center gap-4">
          <Icon name={icon ?? (unit ? getUnitIcon(unit) : undefined)} />
          <div className="flex flex-row items-center gap-2">
            <span className="font-mono text-4xl font-bold">
              {renderValue ? renderValue(value) : value.toString()}
            </span>
            <span>{unit}</span>
          </div>
        </div>
      </Label>
      <Skeleton className="h-16 flex-1 bg-neutral-100"></Skeleton>
    </div>
  );
}

export function ControlValueNumeric(props: Props<number>) {
  return <_ControlValue {...props} />;
}

export function ControlValueBoolean(props: Omit<Props<boolean>, "unit">) {
  return <_ControlValue {...props} />;
}
