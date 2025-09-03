import React, { useState } from "react";
import { Page } from "@/components/Page";
import { ControlCard } from "@/control/ControlCard";
import { Label } from "@/control/Label";
import { useLaser1 } from "./useLaser1";
import { MachineSelector } from "@/components/MachineConnectionDropdown";

export function Laser1SettingsPage() {
  const {
    state,
    defaultState,
    selectedMachine,
    filteredMachines,
    setConnectedWinder,
    disconnectWinder,
  } = useLaser1();

  return (
    <Page>
      <ControlCard title="Speed by Diameter Settings">
        <Label label="Associated Winder">
          <MachineSelector
            machines={filteredMachines}
            selectedMachine={selectedMachine}
            connectedMachineState={state?.connected_winder_state}
            setConnectedMachine={setConnectedWinder}
            clearConnectedMachine={() => {
              if (!selectedMachine) return;
              setConnectedWinder({
                machine_identification: { vendor: 0, machine: 0 },
                serial: 0,
              });
              disconnectWinder(selectedMachine.machine_identification_unique);
            }}
          />
        </Label>
      </ControlCard>
    </Page>
  );
}
