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
        label="Heizen"
        icon="lu:Flame"
        value={heating}
        renderValue={(value) => (value === true ? "ON" : "OFF")}
      />
      <ControlValueNumeric
        label="Temperatur"
        unit="C"
        value={temperature}
        renderValue={(value) => value.toFixed(0) + "°"}
      />

      <Label label="Zieltemperatur">
        <EditValue
          value={targetTemperature}
          unit="C"
          title="Zielgeschwindigkeit"
          renderValue={(value) => value.toFixed(0) + "°"}
        />
      </Label>
    </ControlCard>
  );
}
