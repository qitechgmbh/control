import { Page } from "@/components/Page";
import { ControlCard } from "@/control/ControlCard";
import { Label } from "@/control/Label";
import { SelectionGroupBoolean } from "@/control/SelectionGroup";
import React from "react";
import { useExtruder2 } from "./useExtruder";

export function Extruder2SettingsPage() {
  const { inverterSetRotation, rotationState } = useExtruder2();
  return (
    <Page>
      <ControlCard className="bg-red" title="Inverter Settings">
        <Label label="Rotation Direction">
          <SelectionGroupBoolean
            value={rotationState}
            optionTrue={{ children: "Backward" }}
            optionFalse={{ children: "Forward" }}
            onChange={inverterSetRotation}
          />
        </Label>
      </ControlCard>
    </Page>
  );
}
