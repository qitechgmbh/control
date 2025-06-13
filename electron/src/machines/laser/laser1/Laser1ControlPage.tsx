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
    laserDiameter: laserDiameter,
    laserState,
    laserSetTargetDiameter,
    laserSetLowerTolerance,
    laserSetHigherTolerance,
  } = useLaser1();

  // Controlled local states synced with laserState
  const targetDiameter = laserState?.data?.target_diameter ?? 0;
  const lowerTolerance = laserState?.data?.lower_tolerance ?? 0;
  const higherTolerance = laserState?.data?.higher_tolerance ?? 0;
  return (
    <Page>
      <ControlGrid columns={2}>
        <ControlCard title="Diameter Measurement">
          <DiameterVisualisation
            targetDiameter={targetDiameter}
            lowTolerance={lowerTolerance}
            highTolerance={higherTolerance}
            diameter={laserDiameter}
          />
          <div className="flex flex-row items-center gap-6">
            <TimeSeriesValueNumeric
              label="Current Diameter"
              unit="mm"
              timeseries={laserDiameter}
              renderValue={(value) => value.toFixed(3)}
            />
          </div>
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
                  laserSetLowerTolerance(val);
                }
                laserSetTargetDiameter(val);
              }}
              defaultValue={0}
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
              onChange={(val) => laserSetLowerTolerance(val)}
              defaultValue={0}
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
              onChange={(val) => laserSetHigherTolerance(val)}
              defaultValue={0}
            />
          </Label>
        </ControlCard>
      </ControlGrid>
    </Page>
  );
}
