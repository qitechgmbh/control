import {
  DropdownMenu,
  DropdownMenuTrigger,
  DropdownMenuContent,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuItem,
} from "@/components/ui/dropdown-menu";

import { useMachines } from "@/client/useMachines";
import { Button } from "@/components/ui/button";
import { useState } from "react";
import React from "react";
import { MachineIdentification } from "@/machines/types";

interface MachineSelectDropdownProps {
  machineIdentification: MachineIdentification;
  selectedMachine: string | null;
  onSelect: (machineSlug: string) => void;
}

export function MachineSelectDropdown({
  machineIdentification,
  selectedMachine,
  onSelect,
}: MachineSelectDropdownProps) {
  const machines = useMachines();
  const filteredMachines = machines.filter(
    (m) =>
      m.machine_identification_unique.machine_identification.machine ===
        machineIdentification.machine &&
      m.machine_identification_unique.machine_identification.vendor ===
        machineIdentification.vendor,
  );

  const getSelectedLabel = () =>
    filteredMachines.find((m) => m.name === selectedMachine)?.name ??
    "Select a Machine";

  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>
        <Button variant="outline">{getSelectedLabel()}</Button>
      </DropdownMenuTrigger>
      <DropdownMenuContent align="end">
        <DropdownMenuLabel>Available Machines</DropdownMenuLabel>
        <DropdownMenuSeparator />
        {filteredMachines.map((machine) => (
          <DropdownMenuItem
            key={machine.slug}
            onClick={() => onSelect(machine.slug)}
            className={selectedMachine === machine.slug ? "bg-blue-50" : ""}
          >
            {machine.name}
          </DropdownMenuItem>
        ))}
      </DropdownMenuContent>
    </DropdownMenu>
  );
}
