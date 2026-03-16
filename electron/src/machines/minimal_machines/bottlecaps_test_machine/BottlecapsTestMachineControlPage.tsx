import { Page } from "@/components/Page";
import { ControlCard } from "@/control/ControlCard";
import { ControlGrid } from "@/control/ControlGrid";
import React from "react";
import { useBottlecapsTestMachine } from "./useBottlecapsTestMachine";
import { TouchButton } from "@/components/touch/TouchButton";
import { EditValue } from "@/control/EditValue";
import { Label } from "@/control/Label";
import { roundToDecimals } from "@/lib/decimal";
import { SelectionGroup } from "@/control/SelectionGroup";
import { Badge } from "lucide-react";

export function BottlecapsTestMachineControlPage() {
  const {
    state,
    setOverrideInput,
    setStepperTargetSpeed,
    setStepperEnabled,
    setStepperFreq,
    setStepperAccFreq,
  } = useBottlecapsTestMachine();

  const safeState = state ?? {
    inputs: [false, false, false, false, false, false, false, false],
    override_inputs: [false, false, false, false, false, false, false, false],
    stepper_target_speed: 0,
    stepper_enabled: 0,
    stepper_freq: 0,
    stepper_acc_freq: 0,
  };

  return (
    <Page>
      <ControlGrid columns={2}>
        <ControlCard title="Digital Inputs">
          <div className="grid grid-cols-2 gap-6">
            {safeState.inputs.map((input, index) => (
              <Label key={index} label={`Input ${index + 1}`}>
                <div className="flex h-full items-center justify-center">
                  <Badge
                    className={`text-md ${input ? "bg-green-600" : "bg-gray-400"}`}
                  >
                    {input ? "HIGH" : "LOW"}
                  </Badge>
                </div>
              </Label>
            ))}
          </div>
        </ControlCard>
        <ControlCard title="Input Overrides">
          <div className="grid grid-cols-2 gap-6">
            {safeState.override_inputs.map((output, index) => (
              <Label key={index} label={`Input ${index + 1}`}>
                <SelectionGroup<"On" | "Off">
                  value={output ? "On" : "Off"}
                  orientation="vertical"
                  className="grid h-full grid-cols-2 gap-2"
                  options={{
                    Off: {
                      children: "Off",
                      icon: "lu:CirclePause",
                      isActiveClassName: "bg-red-600",
                      className: "h-full",
                    },
                    On: {
                      children: "On",
                      icon: "lu:CirclePlay",
                      isActiveClassName: "bg-green-600",
                      className: "h-full",
                    },
                  }}
                  onChange={(value) => setOverrideInput(index, value === "On")}
                />
              </Label>
            ))}
          </div>
        </ControlCard>
        <ControlCard title="Stepper Speed">
          <Label label="Target Speed">
            <EditValue
              value={safeState.stepper_target_speed}
              title="Target Speed"
              defaultValue={0}
              min={-25000}
              max={25000}
              step={1}
              renderValue={(value) => roundToDecimals(value, 0)}
              onChange={setStepperTargetSpeed}
            />
            <TouchButton
              variant="outline"
              icon="lu:CirclePower"
              onClick={() => setStepperEnabled(true)}
              disabled={false}
              isLoading={false}
            >
              Enable
            </TouchButton>
            <TouchButton
              variant="outline"
              icon="lu:CirclePower"
              onClick={() => setStepperEnabled(false)}
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
            onClick={() => setStepperFreq(0)}
            disabled={false}
            isLoading={false}
          >
            Default (00)
          </TouchButton>
          <TouchButton
            variant="outline"
            icon="lu:CirclePower"
            onClick={() => setStepperFreq(1)}
            disabled={false}
            isLoading={false}
          >
            Low (01)
          </TouchButton>
          <TouchButton
            variant="outline"
            icon="lu:CirclePower"
            onClick={() => setStepperFreq(2)}
            disabled={false}
            isLoading={false}
          >
            Mid (10)
          </TouchButton>
          <TouchButton
            variant="outline"
            icon="lu:CirclePower"
            onClick={() => setStepperFreq(3)}
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
            onClick={() => setStepperAccFreq(0)}
            disabled={false}
            isLoading={false}
          >
            Default (00)
          </TouchButton>
          <TouchButton
            variant="outline"
            icon="lu:CirclePower"
            onClick={() => setStepperAccFreq(1)}
            disabled={false}
            isLoading={false}
          >
            Low (01)
          </TouchButton>
          <TouchButton
            variant="outline"
            icon="lu:CirclePower"
            onClick={() => setStepperAccFreq(2)}
            disabled={false}
            isLoading={false}
          >
            Mid (10)
          </TouchButton>
          <TouchButton
            variant="outline"
            icon="lu:CirclePower"
            onClick={() => setStepperAccFreq(3)}
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
