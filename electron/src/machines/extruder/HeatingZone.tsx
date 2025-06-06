import { ControlCard } from "@/control/ControlCard";
import { EditValue } from "@/control/EditValue";
import { Label } from "@/control/Label";
import { roundToDecimals } from "@/lib/decimal";
import React from "react";
import { Heating } from "./extruder2/extruder2Namespace";
import { TimeSeries } from "@/lib/timeseries";
import { TimeSeriesValueNumeric } from "@/control/TimeSeriesValue";
import { StatusBadge } from "@/control/StatusBadge";
import { Icon } from "@/components/Icon";

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
      <div className="mb-4 flex flex-col gap-4">
        <div className="flex gap-4">
          <TimeSeriesValueNumeric
            label="Temperature"
            unit="C"
            renderValue={(value) =>
              heatingState?.wiring_error ? "0.0" : roundToDecimals(value, 1)
            }
            timeseries={heatingTimeSeries}
          />

          <div className="flex items-center space-x-2">
            <Icon
              name="lu:Flame"
              className={`h-5 w-5 ${heating ? "text-orange-500" : "text-gray-400"}`}
            />
          </div>
        </div>

        <Label label="Target Temperature">
          <EditValue
            value={targetTemperature}
            defaultValue={150}
            min={50}
            max={300}
            unit="C"
            title="Target Temperature"
            renderValue={(value) => roundToDecimals(value, 1)}
            onChange={onChangeTargetTemp}
          />
        </Label>
      </div>

      {heatingState?.wiring_error && (
        <StatusBadge variant="error">
          Cant Read Temperature! Check Temperature Sensor Wiring!
        </StatusBadge>
      )}
    </ControlCard>
  );
}
