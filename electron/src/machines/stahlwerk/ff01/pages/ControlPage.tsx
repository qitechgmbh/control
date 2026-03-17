import { ControlCard } from "@/control/ControlCard";

import { Page } from "@/components/Page";
import React from "react";
import { ControlGrid } from "@/control/ControlGrid";
import { TimeSeriesValueNumeric } from "@/control/TimeSeriesValue";

import { useFF01_v1 } from "../use";
import { EditValue } from "@/control/EditValue";
import { Label } from "@/control/Label";
import { TouchButton } from "@/components/touch/TouchButton";
import { start } from "repl";
import { DisplayValue } from "@/control/DisplayValue";

export function ControlPage() {
  const {
    state,
    defaultState,
    weightPeak,
    weightPrev,
    tare,
    clearLights,
    isDisabled,
    isLoading,
  } = useFF01_v1();

  return (
    <Page>
      <ControlGrid columns={2}>
        <ControlCard title="Measurements">
          <Label label={state?.weighted_item.name ?? "Waiting for data..."}>
            <DisplayValue
              title="Target Weight"
              icon="lu:Scale"
              unit="kg"
              value={state?.weighted_item.weight}
              renderValue={(v) => v.toFixed(0)}
            />

            <TimeSeriesValueNumeric
              label="Current Weight"
              unit="kg"
              renderValue={(v) => v.toFixed(0)}
              timeseries={motorScrewRpm}
            />

          </Label>

          

        </ControlCard>


        <ControlCard title="Current">
          <TimeSeriesValueNumeric
            label="Live Weight"
            unit="kg"
            timeseries={currentWeight}
            renderValue={(value) => value.toFixed(1)}
          />
          <TimeSeriesValueNumeric
            label="Total Weight"
            unit="kg"
            timeseries={totalWeight}
            renderValue={(value) => value.toFixed(1)}
          />
        </ControlCard>
        <ControlCard title="Plate Counter">
          <DisplayValue
            title="Target Quantity"
            icon="lu:Tally1"
            unit="pcs"
            value={state?.weighted_item.quantity}
            renderValue={(v) => v.toFixed(1)}
          />
          <TimeSeriesValueNumeric
            label="Current Quantity"
            unit="pcs"
            timeseries={plateCounter}
            renderValue={(value) => value.toFixed(0)}
          />
          <TouchButton
            variant="outline"
            icon="lu:RotateCcw"
            onClick={zeroCounters}
          >
            Zero Counters
          </TouchButton>
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
          <Label label="Set Upper Tolerance">
            <EditValue
              title="Set Tolerance"
              value={state?.xtrem_zebra_state.tolerance}
              unit="kg"
              step={0.1}
              min={0}
              max={1000}
              renderValue={(value) => value.toFixed(2)}
              onChange={(val) => setTolerance(val)}
              defaultValue={defaultState?.xtrem_zebra_state.tolerance}
            />
          </Label>
          <Label label="Set Lower Tolerance">
            <EditValue
              title="Set Tolerance"
              value={state?.xtrem_zebra_state.tolerance}
              unit="kg"
              step={0.1}
              min={0}
              max={1000}
              renderValue={(value) => value.toFixed(2)}
              onChange={(val) => setTolerance(val)}
              defaultValue={defaultState?.xtrem_zebra_state.tolerance}
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
          <Label label="Lights">
            <TouchButton
              variant="outline"
              icon="lu:RotateCcw"
              onClick={clearLights}
            >
              Clear Lights
            </TouchButton>
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
