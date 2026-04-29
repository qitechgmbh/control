import { ControlCard } from "@/control/ControlCard";
import { Page } from "@/components/Page";
import React from "react";
import { ControlGrid } from "@/control/ControlGrid";
import { useTestMachineStepper } from "./useTestMachineStepper";
import { Label } from "@/control/Label";
import { EditValue } from "@/control/EditValue";
import { roundToDecimals } from "@/lib/decimal";
import { SelectionGroup } from "@/control/SelectionGroup";
import {
  AccelerationFaktor,
  Frequency,
  Mode,
} from "./testMachineStepperNamespace";

export function TestMachineStepperControlPage() {
  const {
    state,
    setTargetSpeed,
    setFreq,
    setAccFactor,
    setMode,
    isDisabled,
    isLoading,
  } = useTestMachineStepper();

  return (
    <Page>
      <ControlGrid columns={2}>
        <ControlCard className="bg-red" title="Mode">
          <SelectionGroup<Mode>
            value={state?.mode_state.mode}
            disabled={isDisabled}
            loading={isLoading}
            onChange={setMode}
            orientation="vertical"
            className="grid h-full grid-cols-2 gap-2"
            options={{
              Standby: {
                children: "Standby",
                icon: "lu:Power",
                isActiveClassName: "bg-green-600",
                className: "h-full",
              },
              Hold: {
                children: "Hold",
                icon: "lu:CirclePause",
                isActiveClassName: "bg-green-600",
                className: "h-full",
              },
              Turn: {
                children: "Turn",
                icon: "lu:ChevronsLeft",
                isActiveClassName: "bg-green-600",
                className: "h-full",
              },
            }}
          />
        </ControlCard>
        <ControlCard title="Stepper Speed">
          <Label label="Target Speed">
            <EditValue
              value={state?.target_speed}
              title="Target Speed"
              defaultValue={0}
              min={-25000}
              max={25000}
              step={1}
              renderValue={(value) => roundToDecimals(value, 0)}
              onChange={setTargetSpeed}
            />
          </Label>
        </ControlCard>
        <ControlCard className="bg-red" title="Frequency Prescaler">
          <SelectionGroup<Frequency>
            value={state?.frequency_state.frequency}
            disabled={isDisabled}
            loading={isLoading}
            onChange={setFreq}
            orientation="vertical"
            className="grid h-full grid-cols-2 gap-2"
            options={{
              Default: {
                children: "Default",
                icon: "lu:Power",
                isActiveClassName: "bg-green-600",
                className: "h-full",
              },
              Low: {
                children: "Low",
                icon: "lu:CirclePause",
                isActiveClassName: "bg-green-600",
                className: "h-full",
              },
              Mid: {
                children: "Mid",
                icon: "lu:ChevronsLeft",
                isActiveClassName: "bg-green-600",
                className: "h-full",
              },
              High: {
                children: "High",
                icon: "lu:ChevronsLeft",
                isActiveClassName: "bg-green-600",
                className: "h-full",
              },
            }}
          />
        </ControlCard>
        <ControlCard className="bg-red" title="Acceleration Faktor">
          <SelectionGroup<AccelerationFaktor>
            value={state?.acceleration_state.factor}
            disabled={isDisabled}
            loading={isLoading}
            onChange={setAccFactor}
            orientation="vertical"
            className="grid h-full grid-cols-2 gap-2"
            options={{
              Default: {
                children: "Default",
                icon: "lu:Power",
                isActiveClassName: "bg-green-600",
                className: "h-full",
              },
              Low: {
                children: "Low",
                icon: "lu:CirclePause",
                isActiveClassName: "bg-green-600",
                className: "h-full",
              },
              Mid: {
                children: "Mid",
                icon: "lu:ChevronsLeft",
                isActiveClassName: "bg-green-600",
                className: "h-full",
              },
              High: {
                children: "High",
                icon: "lu:ChevronsLeft",
                isActiveClassName: "bg-green-600",
                className: "h-full",
              },
            }}
          />
        </ControlCard>
      </ControlGrid>
    </Page>
  );
}
