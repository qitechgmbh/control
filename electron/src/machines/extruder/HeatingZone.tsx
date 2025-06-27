import { ControlCard } from "@/control/ControlCard";
import { EditValue } from "@/control/EditValue";
import { Label } from "@/control/Label";
import { roundToDecimals } from "@/lib/decimal";
import React from "react";
import { TimeSeries } from "@/lib/timeseries";
import { TimeSeriesValueNumeric } from "@/control/TimeSeriesValue";
import { StatusBadge } from "@/control/StatusBadge";

type Props = {
  title: string;
  heatingTarget: number | undefined;
  heatingTimeSeries: TimeSeries;
  heatingPower: TimeSeries;
  heatingWiringError: boolean | null;
  onChangeTargetTemp?: (temperature: number) => void;
};
export function HeatingZone({
  title,
  heatingTarget,
  heatingWiringError,
  heatingTimeSeries,
  heatingPower,
  onChangeTargetTemp,
}: Props) {
  const targetTemperature = heatingTarget ?? 150;
  const wiringError = heatingWiringError ?? false;

  return (
    <ControlCard className="bg-red" title={title}>
      <div className="mb-4 flex flex-col gap-4">
        <div className="flex gap-4">
          <TimeSeriesValueNumeric
            label="Temperature"
            unit="C"
            renderValue={(value) =>
              wiringError ? "0.0" : roundToDecimals(value, 1)
            }
            timeseries={heatingTimeSeries}
          />
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

      <TimeSeriesValueNumeric
        label="Heating Power"
        unit="W"
        renderValue={(value) =>
          wiringError ? "0.0" : roundToDecimals(value, 0)
        }
        timeseries={heatingPower}
      />

      {wiringError && (
        <StatusBadge variant="error">
          Cant Read Temperature! Check Temperature Sensor Wiring!
        </StatusBadge>
      )}
    </ControlCard>
  );
}
