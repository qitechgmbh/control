import { ControlCard } from "@/control/ControlCard";
import { Page } from "@/components/Page";
import React from "react";
import { ControlGrid } from "@/control/ControlGrid";
import { useTestMachineStepper } from "./useTestMachineStepper";
import { Label } from "@/control/Label";
import { EditValue } from "@/control/EditValue";
import { roundToDecimals } from "@/lib/decimal";
import { TouchButton } from "@/components/touch/TouchButton";

export function TestMachineStepperControlPage() {
  const { state, setTargetSpeed, setEnabled, setFreq, setAccFreq } =
    useTestMachineStepper();

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
          </Label>
        </ControlCard>
        <ControlCard height={2} title="Frequency Prescaler">
          <TouchButton
            variant="outline"
            icon="lu:CirclePower"
            onClick={() => setFreq(0)}
            disabled={false}
            isLoading={false}
          >
            Default (00)
          </TouchButton>
          <TouchButton
            variant="outline"
            icon="lu:CirclePower"
            onClick={() => setFreq(1)}
            disabled={false}
            isLoading={false}
          >
            Low (01)
          </TouchButton>
          <TouchButton
            variant="outline"
            icon="lu:CirclePower"
            onClick={() => setFreq(2)}
            disabled={false}
            isLoading={false}
          >
            Mid (10)
          </TouchButton>
          <TouchButton
            variant="outline"
            icon="lu:CirclePower"
            onClick={() => setFreq(3)}
            disabled={false}
            isLoading={false}
          >
            High (11)
          </TouchButton>
        </ControlCard>
        <ControlCard height={2} title="Acceleration Factor">
          <TouchButton
            variant="outline"
            icon="lu:CirclePower"
            onClick={() => setAccFreq(0)}
            disabled={false}
            isLoading={false}
          >
            Default (00)
          </TouchButton>
          <TouchButton
            variant="outline"
            icon="lu:CirclePower"
            onClick={() => setAccFreq(1)}
            disabled={false}
            isLoading={false}
          >
            Low (01)
          </TouchButton>
          <TouchButton
            variant="outline"
            icon="lu:CirclePower"
            onClick={() => setAccFreq(2)}
            disabled={false}
            isLoading={false}
          >
            Mid (10)
          </TouchButton>
          <TouchButton
            variant="outline"
            icon="lu:CirclePower"
            onClick={() => setAccFreq(3)}
            disabled={false}
            isLoading={false}
          >
            High (11)
          </TouchButton>
        </ControlCard>
      </ControlGrid>
    </Page>
  );
}
