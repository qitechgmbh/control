import { ControlCard } from "@/control/ControlCard";
import { Page } from "@/components/Page";
import React from "react";
import { ControlGrid } from "@/control/ControlGrid";
import { TimeSeriesValueNumeric } from "@/control/TimeSeriesValue";

import { useXtremZebra1 } from "./useXtremZebra";
import { EditValue } from "@/control/EditValue";
import { Label } from "@/control/Label";
import { TouchButton } from "@/components/touch/TouchButton";

export function XtremZebraControlPage() {
  const {
    state,
    defaultState,
    total_weight,
    current_weight,
    plate1_counter,
    plate2_counter,
    plate3_counter,
    setTolerance,
    setPlate1Target,
    setPlate2Target,
    setPlate3Target,
    setTare,
    zeroCounters,
    isDisabled,
    isLoading,
  } = useXtremZebra1();

  return (
    <Page>
      <ControlGrid columns={2}>
        <ControlCard title="Current Weight">
          <TimeSeriesValueNumeric
            label=""
            unit="kg"
            timeseries={current_weight}
            renderValue={(value) => value.toFixed(1)}
          />
        </ControlCard>
        <ControlCard title="Total Weight">
          <TimeSeriesValueNumeric
            label=""
            unit="kg"
            timeseries={total_weight}
            renderValue={(value) => value.toFixed(1)}
          />
        </ControlCard>
        <ControlCard title="Plate Counter">
          <TimeSeriesValueNumeric
            label="Plate 1"
            unit="pcs"
            timeseries={plate1_counter}
            renderValue={(value) => value.toFixed(1)}
          />
          <TimeSeriesValueNumeric
            label="Plate 2"
            unit="pcs"
            timeseries={plate2_counter}
            renderValue={(value) => value.toFixed(1)}
          />
          <TimeSeriesValueNumeric
            label="Plate 3"
            unit="pcs"
            timeseries={plate3_counter}
            renderValue={(value) => value.toFixed(1)}
          />
          <Label label="Counters">
            <TouchButton
              variant="outline"
              icon="lu:CircleOff"
              onClick={zeroCounters}
            >
              Zero Counters
            </TouchButton>
          </Label>
        </ControlCard>
        <ControlCard title="Set Target Weight and Tolerance ">
          <Label label="Set Tolerance">
            <EditValue
              title="Set Tolerance"
              value={state?.xtrem_zebra_state.tolerance}
              unit="kg"
              step={0.1}
              min={0}
              max={100}
              renderValue={(value) => value.toFixed(2)}
              onChange={(val) => setTolerance(val)}
              defaultValue={defaultState?.xtrem_zebra_state.tolerance}
            />
          </Label>
          <Label label="Set Plate 1 Target">
            <EditValue
              title="Set Plate 1 Target"
              value={state?.xtrem_zebra_state.plate1_target}
              unit="kg"
              step={0.1}
              min={0}
              max={1000}
              renderValue={(value) => value.toFixed(2)}
              onChange={(val) => setPlate1Target(val)}
              defaultValue={defaultState?.xtrem_zebra_state.plate1_target}
            />
          </Label>
          <Label label="Set Plate 2 Target">
            <EditValue
              title="Set Plate 2 Target"
              value={state?.xtrem_zebra_state.plate2_target}
              unit="kg"
              step={0.1}
              min={0}
              max={1000}
              renderValue={(value) => value.toFixed(2)}
              onChange={(val) => setPlate2Target(val)}
              defaultValue={defaultState?.xtrem_zebra_state.plate2_target}
            />
          </Label>
          <Label label="Set Plate 3 Target">
            <EditValue
              title="Set Plate 3 Target"
              value={state?.xtrem_zebra_state.plate3_target}
              unit="kg"
              step={0.1}
              min={0}
              max={1000}
              renderValue={(value) => value.toFixed(2)}
              onChange={(val) => setPlate3Target(val)}
              defaultValue={defaultState?.xtrem_zebra_state.plate3_target}
            />
          </Label>
          <Label label="Tare">
            <TouchButton variant="outline" icon="lu:Scale" onClick={setTare}>
              Tare Scales
            </TouchButton>
          </Label>
        </ControlCard>
      </ControlGrid>
    </Page>
  );
}
