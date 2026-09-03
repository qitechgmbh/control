import { Page } from "@/components/Page";
import { ControlGrid } from "@/control/ControlGrid";
import React from "react";
import { MachineSelector } from "@/components/MachineConnectionDropdown";
import { useBuffer1 } from "./useBuffer1";

export function Buffer1SettingsPage() {
  const {
    state,
    selectedMachine,
    filteredMachines,
    setConnectedMachine,
    disconnectMachine,
  } = useBuffer1();

  return (
    <Page>
      <ControlGrid>
        <MachineSelector
          machines={filteredMachines}
          selectedMachine={selectedMachine}
          connectedMachineState={state?.connected_machine_state}
          setConnectedMachine={setConnectedMachine}
          clearConnectedMachine={() => {
            if (!selectedMachine) return;
            setConnectedMachine({
              machine_identification: { vendor: 0, machine: 0 },
              serial: 0,
            });
            disconnectMachine(selectedMachine.machine_identification_unique);
          }}
        />
      </ControlGrid>
    </Page>
  );
}
