import { Page } from "@/components/Page";
import { ControlCard } from "@/control/ControlCard";
import { ControlGrid } from "@/control/ControlGrid";
import { EditValue } from "@/control/EditValue";
import { Label } from "@/control/Label";
import { useAquapath1 } from "./useAquapath";
import React from "react";

export function Aquapath1SettingsPage() {
  const {
    state,
    setFrontRevolutions,
    setBackRevolutions,
    setFrontHeatingTolerance,
    setFrontCoolingTolerance,
    setBackHeatingTolerance,
    setBackCoolingTolerance,
  } = useAquapath1();

  return (
    <Page>
      <ControlGrid columns={2}>
        <ControlCard title="Front Fan Revolutions">
          <Label label="Set Max Revolution Speed">
            <EditValue
              title="Set Target Revolution Speed"
              min={0}
              value={state?.fan_states.front.revolutions}
              max={100}
              unit="%"
              renderValue={(value) => value.toFixed(1)}
              onChange={(val) => {
                setFrontRevolutions(val);
              }}
            />
          </Label>
        </ControlCard>

        <ControlCard title="Back Fan Revolutions">
          <Label label="Set Max Revolution Speed">
            <EditValue
              title="Set Target Revolution Speed"
              min={0}
              value={state?.fan_states.back.revolutions}
              max={100}
              unit="%"
              renderValue={(value) => value.toFixed(1)}
              onChange={(val) => {
                setBackRevolutions(val);
              }}
            />
          </Label>
        </ControlCard>

        <ControlCard>
          <Label label="Set Back Temperature Tolerances">
            <EditValue
              title="Set Heating Tolerance"
              min={0}
              value={state?.tolerance_states.back.heating}
              max={10}
              unit="C"
              renderValue={(value) => value.toFixed(1)}
              onChange={(val) => {
                setBackHeatingTolerance(val);
              }}
            />
          </Label>

          <Label label="Set Back Temperature Tolerances">
            <EditValue
              title="Set Cooling Tolerance"
              min={0}
              value={state?.tolerance_states.back.heating}
              max={10}
              unit="C"
              renderValue={(value) => value.toFixed(1)}
              onChange={(val) => {
                setBackCoolingTolerance(val);
              }}
            />
          </Label>
        </ControlCard>

        <ControlCard>
          <Label label="Set Heating Tolerance">
            <EditValue
              title="Set Target Revolution Speed"
              min={0}
              value={state?.tolerance_states.front.heating}
              max={100}
              unit="C"
              renderValue={(value) => value.toFixed(1)}
              onChange={(val) => {
                setFrontHeatingTolerance(val);
              }}
            />
          </Label>

          <Label label="Set Heating Tolerance">
            <EditValue
              title="Set Target Revolution Speed"
              min={0}
              value={state?.tolerance_states.front.cooling}
              max={100}
              unit="C"
              renderValue={(value) => value.toFixed(1)}
              onChange={(val) => {
                setFrontCoolingTolerance(val);
              }}
            />
          </Label>
       </ControlCard>
      </ControlGrid>
    </Page>
  );
}
