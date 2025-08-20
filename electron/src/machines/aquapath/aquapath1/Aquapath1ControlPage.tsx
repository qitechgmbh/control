import { Page } from "@/components/Page";
import { useAquapath1 } from "./useAquapath1";
import { ControlCard } from "@/control/ControlCard";
import { ControlGrid } from "@/control/ControlGrid";
import React from "react";
import { TimeSeriesValueNumeric } from "@/control/TimeSeriesValue";
import { roundToDecimals } from "@/lib/decimal";
import { Label } from "@/control/Label";
import { EditValue } from "@/control/EditValue";

export function Aquapath1ControlPage() {
  // use optimistic state
  const {
    state,
    defaultState,
    fanRpm,
    waterTemperature,
    flowRate,
    setTargetTemperature,
  } = useAquapath1();

  return (
    <Page>
      <ControlGrid>
        <ControlCard title="Aquapath">
          <Label label="Temperature">
            <TimeSeriesValueNumeric
              label="Current Temperature"
              unit="C"
              timeseries={waterTemperature}
              renderValue={(value) => roundToDecimals(value, 0)}
            />
            <EditValue
              value={state?.target_temperature}
              unit="C"
              title="Target Temperature"
              min={0}
              max={80}
              defaultValue={defaultState?.target_temperature}
              minLabel="Bot"
              maxLabel="Top"
              renderValue={(value) => roundToDecimals(value, 0)}
              onChange={setTargetTemperature}
            />
          </Label>
        </ControlCard>
        <ControlCard title="Fan">
          <TimeSeriesValueNumeric
            label="Fan Speed"
            unit="rpm"
            timeseries={fanRpm}
            renderValue={(value) => roundToDecimals(value, 0)}
          />
        </ControlCard>
        <ControlCard title="Water">
          <TimeSeriesValueNumeric
            label="Flow Rate"
            unit="m/min"
            timeseries={flowRate}
            renderValue={(value) => roundToDecimals(value, 0)}
          />
        </ControlCard>
      </ControlGrid>
    </Page>
  );
}
