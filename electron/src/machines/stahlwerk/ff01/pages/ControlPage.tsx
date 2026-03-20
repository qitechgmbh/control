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
    clearTare,
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
            renderValue={(v) => v.toFixed(2)}
            timeseries={weightPrev}
          />
          <TimeSeriesValueNumeric
            label="Peak Weight"
            unit="kg"
            renderValue={(v) => v.toFixed(2)}
            timeseries={weightPeak}
          />    
        </ControlCard>

        <ControlCard title="Service Information">
          <span>{"Workorder: " + state?.current_entry?.doc_entry }</span>
          <span>{"Line Number: " + state?.current_entry?.line_number }</span>
          <span>{"Item Code: " + state?.current_entry?.item_code }</span>
          <span>{"Warehouse Code: " + state?.current_entry?.whs_code }</span>
          <span>Weight Bounds:</span>
          <span>{" - Minimum: " + state?.current_entry?.weight_bounds.min }</span>
          <span>{" - Maximum: " + state?.current_entry?.weight_bounds.max }</span>
          <span>{" - Desired: " + state?.current_entry?.weight_bounds.desired }</span>

          <DisplayValue
            title="Counted Plates"
            icon="lu:Cuboid"
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
              isLoading={isLoading}
              disabled={isDisabled}
            >
              Clear Lights
            </TouchButton>
          </Label>
          <Label label="Scales">
            <TouchButton 
            variant="outline" 
            icon="lu:Scale" 
            onClick={tare}
            isLoading={isLoading}
            disabled={isDisabled}
            >
              Tare
            </TouchButton>
            <TouchButton 
              variant="outline" 
              icon="lu:RotateCcw" 
              onClick={clearTare}
              isLoading={isLoading}
              disabled={isDisabled}
              >
                Clear Tare
            </TouchButton>
          </Label>
        </ControlCard>
      </ControlGrid>
    </Page>
  );
}
