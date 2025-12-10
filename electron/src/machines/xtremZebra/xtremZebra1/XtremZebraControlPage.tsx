import { ControlCard } from "@/control/ControlCard";
import { EditString } from "@/control/EditString";

import { Page } from "@/components/Page";
import React from "react";
import { ControlGrid } from "@/control/ControlGrid";
import { TimeSeriesValueNumeric } from "@/control/TimeSeriesValue";

import { useXtremZebra1 } from "./useXtremZebra";
import { EditValue } from "@/control/EditValue";
import { Label } from "@/control/Label";
import { TouchButton } from "@/components/touch/TouchButton";
import { start } from "repl";
import { DisplayValue } from "@/control/DisplayValue";

export function XtremZebraControlPage() {
  const {
    state,
    defaultState,
    totalWeight,
    currentWeight,
    plate1Counter,
    plate2Counter,
    plate3Counter,
    setTolerance,
    setPlate1Target,
    setPlate2Target,
    setPlate3Target,
    setTare,
    start,

    setPassword,
    setStringValue,

    zeroCounters,
    clearLights,
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
            timeseries={currentWeight}
            renderValue={(value) => value.toFixed(1)}
          />
        </ControlCard>
        <ControlCard title="Total Weight">
          <TimeSeriesValueNumeric
            label=""
            unit="kg"
            timeseries={totalWeight}
            renderValue={(value) => value.toFixed(1)}
          />
        </ControlCard>
        <ControlCard title="Plate Counter">
          <DisplayValue
            title="Target Quantity"
            description="The amount of plates we expect"
            unit="pcs"
            value={state?.xtrem_zebra_state.target_quantity}
            renderValue={(v) => v.toFixed(1)}
          />
          <TimeSeriesValueNumeric
            label="Plate 1"
            unit="pcs"
            timeseries={plate1Counter}
            renderValue={(value) => value.toFixed(1)}
          />
          <TimeSeriesValueNumeric
            label="Plate 2"
            unit="pcs"
            timeseries={plate2Counter}
            renderValue={(value) => value.toFixed(1)}
          />
          <TimeSeriesValueNumeric
            label="Plate 3"
            unit="pcs"
            timeseries={plate3Counter}
            renderValue={(value) => value.toFixed(1)}
          />
          <Label label="Counters">
            <TouchButton
              variant="outline"
              icon="lu:RotateCcw"
              onClick={zeroCounters}
            >
              Zero Counters
            </TouchButton>
          </Label>
          <Label label="Lights">
            <TouchButton
              variant="outline"
              icon="lu:RotateCcw"
              onClick={clearLights}
            >
              Clear Lights
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

        <ControlCard title="Access & Configuration">
          <Label label="Password">
            <EditString
              title="Enter Password"
              value={state?.configuration.password ?? ""}
              onChange={(val) => setPassword(val)}
            />
          </Label>
          <Label label="Configuration String">
            <EditString
              title="Configuration String"
              value={state?.configuration.config_string ?? ""}
              onChange={(val) => setStringValue(val)}
            />
          </Label>
          <Label label="Start Weighted Item">
            <TouchButton
              variant="outline"
              icon="lu:ArrowBigRight"
              onClick={start}
            >
              Start
            </TouchButton>
          </Label>
        </ControlCard>
      </ControlGrid>
    </Page>
  );
}
