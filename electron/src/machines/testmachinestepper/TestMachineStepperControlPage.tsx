import { ControlCard } from "@/control/ControlCard";
import { Page } from "@/components/Page";
import React from "react";
import { ControlGrid } from "@/control/ControlGrid";
import { useTestMachineStepper } from "./useTestMachineStepper";
import { Label } from "@/control/Label";
import { EditValue } from "@/control/EditValue";
import { roundToDecimals } from "@/lib/decimal";

export function TestMachineStepperControlPage() {
  const { state, setTargetSpeed } = useTestMachineStepper();

  return (
    <Page>
      <ControlGrid columns={2}>
        <ControlCard title="Stepper Speed">
          <Label label="Target Speed">
            <EditValue
              value={state?.target_speed}
              unit="m/min"
              title="Target Speed"
              defaultValue={0}
              min={-25000}
              max={25000}
              step={0.1}
              renderValue={(value) => roundToDecimals(value, 1)}
              onChange={setTargetSpeed}
            />
          </Label>
        </ControlCard>
      </ControlGrid>
    </Page>
  );
}
