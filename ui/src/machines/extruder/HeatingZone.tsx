import { ControlCard } from "@ui/control/ControlCard";
import { EditValue } from "@ui/control/EditValue";
import { Label } from "@ui/control/Label";
import { roundToDecimals } from "@ui/lib/decimal";
import React from "react";
import { HeatingState } from "./extruder2/extruder2Namespace";
import { TimeSeries } from "@ui/lib/timeseries";
import { TimeSeriesValueNumeric } from "@ui/control/TimeSeriesValue";
import { StatusBadge } from "@ui/control/StatusBadge";

type Props = {
  title: string;
  heatingState?: HeatingState;
  heatingTimeSeries: TimeSeries;
  heatingPower: TimeSeries;
  min: number;
  max: number;
  onChangeTargetTemp?: (temperature: number) => void;
  targetTemperatureEnabled: boolean;
};
export function HeatingZone({
  title,
  min,
  max,
  heatingState,
  heatingTimeSeries,
  heatingPower,
  onChangeTargetTemp,
  targetTemperatureEnabled,
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
        {targetTemperatureEnabled && (
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
        )}
      </div>

      <TimeSeriesValueNumeric
        label="Heating Power"
        unit="W"
        renderValue={(value) =>
          heatingState?.wiring_error ? "0.0" : roundToDecimals(value, 0)
        }
        timeseries={heatingPower}
      />

      {heatingState?.wiring_error && targetTemperatureEnabled && (
        <StatusBadge variant="error">
          Cant Read Temperature! Check Temperature Sensor Wiring!
        </StatusBadge>
      )}
    </ControlCard>
  );
}
