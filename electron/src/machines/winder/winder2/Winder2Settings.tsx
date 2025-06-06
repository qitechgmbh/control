import { Page } from "@/components/Page";
import { ControlCard } from "@/control/ControlCard";
import { ControlGrid } from "@/control/ControlGrid";
import { EditValue } from "@/control/EditValue";
import React from "react";
import { useWinder2 } from "./useWinder";
import { roundToDecimals } from "@/lib/decimal";
import { Label } from "@/control/Label";
import { SelectionGroupBoolean } from "@/control/SelectionGroup";

export function Winder2SettingPage() {
  const {
    traverseState,
    traverseSetStepSize,
    traverseSetPadding,
    pullerState,
    pullerSetForward,
    pullerStateIsDisabled,
    pullerStateIsLoading,
  } = useWinder2();
  return (
    <Page>
      <ControlGrid>
        <ControlCard title="Traverse">
          <Label label="Step Size">
            <EditValue
              value={traverseState?.data.step_size}
              title={"Step Size"}
              unit="mm"
              step={0.05}
              min={0.1}
              max={10}
              defaultValue={1.0}
              renderValue={(value) => roundToDecimals(value, 2)}
              onChange={(value) => traverseSetStepSize(value)}
            />
          </Label>
          <Label label="Padding">
            <EditValue
              value={traverseState?.data.padding}
              title={"Padding"}
              unit="mm"
              step={0.01}
              min={0}
              max={5}
              defaultValue={0.01}
              renderValue={(value) => roundToDecimals(value, 2)}
              onChange={(value) => traverseSetPadding(value)}
            />
          </Label>
        </ControlCard>

        <ControlCard title="Puller">
          <Label label="Rotation Direction">
            <SelectionGroupBoolean
              value={pullerState?.data.forward}
              disabled={pullerStateIsDisabled}
              loading={pullerStateIsLoading}
              optionFalse={{
                children: "Reverse",
                icon: "lu:RotateCcw",
              }}
              optionTrue={{
                children: "Forward",
                icon: "lu:RotateCw",
              }}
              onChange={(value) => pullerSetForward(value)}
            />
          </Label>
        </ControlCard>
      </ControlGrid>
    </Page>
  );
}
