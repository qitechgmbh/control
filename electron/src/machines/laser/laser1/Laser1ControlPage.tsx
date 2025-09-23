import { ControlCard } from "@/control/ControlCard";
import { Page } from "@/components/Page";
import React from "react";
import { ControlGrid } from "@/control/ControlGrid";
import { TimeSeriesValueNumeric } from "@/control/TimeSeriesValue";

import { EditValue } from "@/control/EditValue";
import { Label } from "@/control/Label";

import { useLaser1 } from "./useLaser1";

import { DiameterVisualisation } from "../DiameterVisualisation";

export function Laser1ControlPage() {
  const {
    diameter,
    x_diameter,
    y_diameter,
    roundness,
    state,
    defaultState,
    setTargetDiameter,
    setLowerTolerance,
    setHigherTolerance,
  } = useLaser1();

  // Extract values from consolidated state
  const targetDiameter = state?.laser_state?.target_diameter ?? 0;
  const lowerTolerance = state?.laser_state?.lower_tolerance ?? 0;
  const higherTolerance = state?.laser_state?.higher_tolerance ?? 0;
  return (
    <Page>
      <ControlGrid columns={2}>
        <ControlCard title="Diameter Measurement">
          <DiameterVisualisation
            targetDiameter={targetDiameter}
            lowTolerance={lowerTolerance}
            highTolerance={higherTolerance}
            diameter={diameter}
            x_diameter={x_diameter}
            y_diameter={y_diameter}
          />
          <div className="flex flex-row items-center gap-6">
            <TimeSeriesValueNumeric
              label="Diameter"
              unit="mm"
              timeseries={diameter}
              renderValue={(value) => value.toFixed(3)}
            />
          </div>
          {x_diameter?.current && (
            <div className="flex flex-row items-center gap-6">
              <TimeSeriesValueNumeric
                label="X-Diameter"
                unit="mm"
                timeseries={x_diameter}
                renderValue={(value) => value.toFixed(3)}
              />
            </div>
          )}
          {y_diameter?.current && (
            <div className="flex flex-row items-center gap-6">
              <TimeSeriesValueNumeric
                label="Y-Diameter"
                unit="mm"
                timeseries={y_diameter}
                renderValue={(value) => value.toFixed(3)}
              />
            </div>
          )}
          {roundness?.current && (
            <div className="flex flex-row items-center gap-6">
              <TimeSeriesValueNumeric
                label="Roundness"
                icon="lu:Circle"
                timeseries={roundness}
                renderValue={(value) => value.toFixed(3)}
              />
            </div>
          )}
        </ControlCard>
        <ControlCard title="Settings">
          <Label label="Set Target Diameter">
            <EditValue
              title="Set Target Diameter"
              value={targetDiameter}
              unit="mm"
              step={0.01}
              min={0}
              max={5}
              renderValue={(value) => value.toFixed(2)}
              onChange={(val) => {
                if (val < lowerTolerance) {
                  setLowerTolerance(val);
                }
                setTargetDiameter(val);
              }}
              defaultValue={defaultState?.laser_state.target_diameter}
            />
          </Label>
          <Label label="Set Lower Tolerance">
            <EditValue
              title="Set Lower Tolerance"
              value={lowerTolerance}
              unit="mm"
              step={0.01}
              min={0}
              max={Math.min(targetDiameter, 1)}
              renderValue={(value) => value.toFixed(2)}
              onChange={(val) => setLowerTolerance(val)}
              defaultValue={defaultState?.laser_state.lower_tolerance}
            />
          </Label>
          <Label label="Set Higher Tolerance">
            <EditValue
              title="Set Higher Tolerance"
              value={higherTolerance}
              unit="mm"
              step={0.01}
              min={0}
              max={1}
              renderValue={(value) => value.toFixed(2)}
              onChange={(val) => setHigherTolerance(val)}
              defaultValue={defaultState?.laser_state.higher_tolerance}
            />
          </Label>
        </ControlCard>
      </ControlGrid>
    </Page>
  );
}
