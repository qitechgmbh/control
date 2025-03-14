import { ControlCard } from "@/control/ControlCard";
import {
  ControlValueNumeric,
  ControlValueBoolean,
} from "@/control/ControlValue";
import { EditValue } from "@/control/EditValue";
import { Label } from "@/control/Label";
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
      <ControlValueBoolean
        label="Heating"
        icon="lu:Flame"
        value={heating}
        renderValue={(value) => (value === true ? "ON" : "OFF")}
      />
      <ControlValueNumeric
        label="Temperature"
        unit="C"
        value={temperature}
        renderValue={(value) => value.toFixed(0)}
      />

      <Label label="Target Temperature">
        <EditValue
          value={targetTemperature}
          defaultValue={150}
          min={50}
          max={330}
          unit="C"
          title="Target Temperature"
          renderValue={(value) => value.toFixed(0)}
        />
      </Label>
    </ControlCard>
  );
}
