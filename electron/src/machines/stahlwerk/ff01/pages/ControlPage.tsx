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
          <TimeSeriesValueNumeric
            label="Current Weight"
            unit="kg"
            renderValue={(v) => v.toFixed(0)}
            timeseries={weightPrev}
          />
          <TimeSeriesValueNumeric
            label="Peak Weight"
            unit="kg"
            renderValue={(v) => v.toFixed(0)}
            timeseries={weightPeak}
          />    
        </ControlCard>

        <ControlCard title="Service Info">
          <DisplayValue
            title="Target Quantity"
            icon="lu:Tally1"
            unit="pcs"
            value={state?.plates_counted}
            renderValue={(v) => v.toFixed(0)}
          />

          (
            <div className="flex flex-col gap-1">
              <span>{"Active Workorder: " + state?.current_workorder }</span>
            </div>
          )

          <DisplayValue
            title="Active Workorder"
            icon="lu:Tally1"
            unit="pcs"
            value={state?.plates_counted}
            renderValue={(v) => v.toFixed(0)}
          />
        </ControlCard>

        <ControlCard title="Configuration">
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
            <TouchButton variant="outline" icon="lu:Scale" onClick={tare}>
              Tare Scales
            </TouchButton>
          </Label>
        </ControlCard>
      </ControlGrid>
    </Page>
  );
}
