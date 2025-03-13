import { Icon, IconName } from "@/components/Icon";
import React from "react";
import { getUnitIcon, Unit } from "./units";
import { Skeleton } from "@/components/ui/skeleton";

type Props = {
  unit: Unit;
  value: number;
  icon?: IconName;
  renderValue: (value: number) => string;
};

export function ControlValue({ unit, value, icon, renderValue }: Props) {
  return (
    <div className="flex flex-row items-center gap-4">
      <Icon name={icon ?? getUnitIcon(unit)} />
      <div className="flex flex-row items-center gap-2">
        <span className="font-mono text-4xl font-bold">
          {renderValue(value)}
        </span>
        <span>{unit}</span>
      </div>
      <Skeleton className="h-16 flex-1 bg-gray-50"></Skeleton>
    </div>
  );
}
