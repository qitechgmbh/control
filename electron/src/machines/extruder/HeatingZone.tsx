import { ControlCard } from "@/control/ControlCard";
import { EditValue } from "@/control/EditValue";
import { Label } from "@/control/Label";
import { roundToDecimals } from "@/lib/decimal";
import React from "react";
import { HeatingState } from "./extruder2/extruder2Namespace";
import { TimeSeries } from "@/lib/timeseries";
import { TimeSeriesValueNumeric } from "@/control/TimeSeriesValue";
import { StatusBadge } from "@/control/StatusBadge";

type Props = {
  title: string;
  heatingState?: HeatingState;
  heatingTimeSeries: TimeSeries;
  heatingPower: TimeSeries;
  min: number;
  max: number;
  onChangeTargetTemp?: (temperature: number) => void;
};
export function HeatingZone({
  title,
  min,
  max,
  heatingState,
  heatingTimeSeries,
  heatingPower,
  onChangeTargetTemp,
}: Props) {
  const targetTemperature = heatingState?.target_temperature ?? 150;

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
        </div>

        <Label label="Target Temperature">
          <EditValue
            value={targetTemperature}
            defaultValue={150}
            min={min}
            max={max}
            unit="C"
            title="Target Temperature"
            renderValue={(value) => roundToDecimals(value, 1)}
            onChange={onChangeTargetTemp}
          />
        </Label>
      </div>

      <TimeSeriesValueNumeric
        label="Heating Power"
        unit="W"
        renderValue={(value) =>
          heatingState?.wiring_error ? "0.0" : roundToDecimals(value, 0)
        }
        timeseries={heatingPower}
      />

      {heatingState?.wiring_error && (
        <StatusBadge variant="error">
          Cant Read Temperature! Check Temperature Sensor Wiring!
        </StatusBadge>
      )}
    </ControlCard>
  );
}
