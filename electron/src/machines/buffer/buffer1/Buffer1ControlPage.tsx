import { Page } from "@/components/Page";
import { ControlGrid } from "@/control/ControlGrid";
import React from "react";

import { useBuffer1 } from "./useBuffer1";
import { ControlCard } from "@/control/ControlCard";
import { SelectionGroup } from "@/control/SelectionGroup";
import { MachineSelector } from "@/components/MachineConnectionDropdown";

export function Buffer1ControlPage() {
  const {
    state,
    setBufferMode,
    selectedMachine,
    filteredMachines,
    setConnectedMachine,
    disconnectMachine,
  } = useBuffer1();

  return (
    <Page>
      <ControlGrid>
        <ControlCard className="bg-red" title="Mode">
          <SelectionGroup<"Standby" | "FillingBuffer" | "EmptyingBuffer">
            value={state?.mode_state.mode}
            orientation="vertical"
            className="grid h-full grid-cols-2 gap-2"
            options={{
              Standby: {
                children: "Standby",
                icon: "lu:CirclePause",
                isActiveClassName: "bg-green-600",
                className: "h-full",
              },
              FillingBuffer: {
                children: "FillingBuffer",
                icon: "lu:Flame",
                isActiveClassName: "bg-green-600",
                className: "h-full",
              },
              EmptyingBuffer: {
                children: "EmptyingBuffer",
                icon: "lu:ArrowBigLeftDash",
                isActiveClassName: "bg-green-600",
                className: "h-full",
              },
            }}
            onChange={setBufferMode}
          />
        </ControlCard>
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
