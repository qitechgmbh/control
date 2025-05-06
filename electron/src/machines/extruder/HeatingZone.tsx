import { ControlCard } from "@/control/ControlCard";
import { EditValue } from "@/control/EditValue";
import { Label } from "@/control/Label";
import { roundToDecimals } from "@/lib/decimal";
import React from "react";

type Props = {
  title: string;
  temperature: number;
  heating: boolean;
  targetTemperature: number;
};

export function HeatingZone({
  title,
  temperature,
  heating,
  targetTemperature,
}: Props) {
  return (
    <ControlCard className="bg-red" title={title}>
      <Label label="Target Temperature">
        <EditValue
          value={targetTemperature}
          defaultValue={150}
          min={50}
          max={330}
          unit="C"
          title="Target Temperature"
          renderValue={(value) => roundToDecimals(value, 0)}
        />
      </Label>
    </ControlCard>
  );
}
