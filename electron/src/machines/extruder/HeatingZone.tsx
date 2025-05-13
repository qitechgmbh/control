import { ControlCard } from "@/control/ControlCard";
import { EditValue } from "@/control/EditValue";
import { Label } from "@/control/Label";
import { roundToDecimals } from "@/lib/decimal";
import { Flame } from "lucide-react"; // Or any other icon library
import React from "react";
import { Heating } from "./extruder2/extruder2Namespace";
import { TimeSeries } from "@/lib/timeseries";
import { TimeSeriesValueNumeric } from "@/control/TimeSeriesValue";

type Props = {
  title: string;
  heatingState: Heating | undefined;
  heatingTimeSeries: TimeSeries;
  onChangeTargetTemp?: (temperature: number) => void;
};

export function HeatingZone({
  title,
  heatingState,
  heatingTimeSeries,
  onChangeTargetTemp,
}: Props) {
  const targetTemperature = heatingState?.target_temperature ?? 150;
  const heating = heatingState?.heating ?? false;

  return (
    <ControlCard className="bg-red" title={title}>
      <div className="mb-4 flex gap-4">
        <Label label="Target Temperature">
          <EditValue
            value={targetTemperature}
            defaultValue={150}
            min={50}
            max={330}
            unit="C"
            title="Target Temperature"
            renderValue={(value) => roundToDecimals(value, 1)}
            onChange={onChangeTargetTemp}
          />
        </Label>

        <TimeSeriesValueNumeric
          label="Temperature"
          unit="C"
          renderValue={(value) => roundToDecimals(value, 1)}
          timeseries={heatingTimeSeries}
        />
        <label className="flex items-center space-x-2">
          <Flame
            className={`h-5 w-5 ${heating ? "text-orange-500" : "text-gray-400"}`}
          />
        </label>
      </div>
    </ControlCard>
  );
}
