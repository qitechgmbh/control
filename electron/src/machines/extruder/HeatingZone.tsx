import { ControlCard } from "@/control/ControlCard";
import { EditValue } from "@/control/EditValue";
import { Label } from "@/control/Label";
import { ReadOnlyValue } from "@/control/ReadonlyValue";
import { roundToDecimals } from "@/lib/decimal";
import { Flame } from "lucide-react"; // Or any other icon library
import React from "react";
import { Heating } from "./extruder2/extruder2Namespace";

type Props = {
  title: string;
  heatingState: Heating | undefined;
  onChangeTargetTemp?: (temperature: number) => void;
};

export function HeatingZone({
  title,
  heatingState,
  onChangeTargetTemp,
}: Props) {
  const targetTemperature = heatingState?.target_temperature ?? 150;
  const temperature = heatingState?.temperature ?? 0;
  const heating = heatingState?.heating ?? false;

  return (
    <ControlCard className="bg-red" title={title}>
      <div className="mb-4 flex">
        <Label label="Target Temperature">
          <EditValue
            value={targetTemperature}
            defaultValue={150}
            min={50}
            max={330}
            unit="C"
            title="Target Temperature"
            renderValue={(value) => roundToDecimals(value, 0)}
            onChange={onChangeTargetTemp}
          />
        </Label>
        <Label label="Temperature">
          <ReadOnlyValue
            value={temperature}
            unit="C"
            renderValue={(value) => roundToDecimals(value, 0)}
          />
        </Label>
        <label className="flex items-center space-x-2">
          <Flame
            className={`h-5 w-5 ${heating ? "text-orange-500" : "text-gray-400"}`}
          />
        </label>
      </div>
    </ControlCard>
  );
}
