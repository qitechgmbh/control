import { Page } from "@/components/Page";
import { ControlCard } from "@/control/ControlCard";
import { ControlGrid } from "@/control/ControlGrid";
import { Label } from "@/control/Label";
import { SelectionGroupBoolean } from "@/control/SelectionGroup";
import React from "react";
import { useMixerV1 } from "./useMixerV1";

export function MixerV1SettingsPage() {
  const { state, setHopperAForward, setHopperBForward } = useMixerV1();

  return (
    <Page>
      <ControlCard title="Motor Direction">
        <ControlGrid columns={2}>
          <Label label="Dispenser 2 Feeder">
            <SelectionGroupBoolean
              value={state?.hopper_a_state.forward}
              optionTrue={{ children: "Forward", icon: "lu:RotateCw" }}
              optionFalse={{ children: "Reverse", icon: "lu:RotateCcw" }}
              onChange={setHopperAForward}
            />
          </Label>
          <Label label="Dispenser 1 Feeder">
            <SelectionGroupBoolean
              value={state?.hopper_b_state.forward}
              optionTrue={{ children: "Forward", icon: "lu:RotateCw" }}
              optionFalse={{ children: "Reverse", icon: "lu:RotateCcw" }}
              onChange={setHopperBForward}
            />
          </Label>
        </ControlGrid>
      </ControlCard>
    </Page>
  );
}
