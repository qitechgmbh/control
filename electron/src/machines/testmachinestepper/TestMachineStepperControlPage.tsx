import { ControlCard } from "@/control/ControlCard";
import { Page } from "@/components/Page";
import React from "react";
import { ControlGrid } from "@/control/ControlGrid";
import { useTestMachineStepper } from "./useTestMachineStepper";
import { Label } from "@/control/Label";
import { EditValue } from "@/control/EditValue";
import { roundToDecimals } from "@/lib/decimal";
import { TouchButton } from "@/components/touch/TouchButton";
import { fa } from "zod/v4/locales";

export function TestMachineStepperControlPage() {
  const { state, setTargetSpeed, setEnabled } = useTestMachineStepper();

  return (
    <Page>
      <ControlGrid columns={2}>
        <ControlCard title="Stepper Speed">
          <Label label="Target Speed">
            <EditValue
              value={state?.target_speed}
              unit="rpm"
              title="Target Speed"
              defaultValue={0}
              min={-25000}
              max={25000}
              step={1}
              renderValue={(value) => roundToDecimals(value, 0)}
              onChange={setTargetSpeed}
            />
          </Label>
          <TouchButton
            variant="outline"
            icon="lu:CirclePower"
            onClick={() => setEnabled(true)}
            disabled={false}
            isLoading={false}
          >
            Enable
          </TouchButton>
          <TouchButton
            variant="outline"
            icon="lu:CirclePower"
            onClick={() => setEnabled(false)}
            disabled={false}
            isLoading={false}
          >
            Disable
          </TouchButton>
        </ControlCard>
      </ControlGrid>
    </Page>
  );
}
