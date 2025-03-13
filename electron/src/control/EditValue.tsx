import { Icon, IconName } from "@/components/Icon";
import React from "react";
import { Unit } from "./units";
import { TouchButton } from "@/components/TouchButton";
import { Separator } from "@/components/ui/separator";

type Props = {
  unit: Unit;
  value: number;
  title: string;
  icon?: IconName;
  renderValue: (value: number) => string;
};

export function EditValue({ unit, value, renderValue }: Props) {
  return (
    <TouchButton
      className="flex w-min flex-col items-center gap-4"
      variant="outline"
    >
      <div className="flex flex-row items-center gap-2">
        <span className="font-mono text-4xl font-bold">
          {renderValue(value)}
        </span>
        <span>{unit}</span>
      </div>
      <Separator orientation="vertical" className="mx-4" />
      <Icon name="lu:Pencil" size={200} />
    </TouchButton>
  );
}
