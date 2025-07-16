import {
  DropdownMenu,
  DropdownMenuTrigger,
  DropdownMenuContent,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuItem,
} from "@/components/ui/dropdown-menu";

import React from "react";

import { Icon } from "@/components/Icon";
import { ControlCard } from "@/control/ControlCard";

interface MachineIdentificationUnique {
  machine_identification: {
    vendor: number;
    machine: number;
  };
  serial: number;
}

interface Machine {
  name: string;
  machine_identification_unique: MachineIdentificationUnique;
}

interface MachineSelectorProps {
  machines: Machine[];
  selectedMachine?: Machine | null;
  connectedMachineState?: {
    machine_identification_unique?: {
      machine_identification: {
        vendor: number;
        machine: number;
      };
      serial: number;
    } | null;
  };
  setConnectedMachine: (machine: MachineIdentificationUnique) => void;
  title?: string;
}

export const MachineSelector: React.FC<MachineSelectorProps> = ({
  machines,
  selectedMachine,
  connectedMachineState,
  setConnectedMachine,
  title = "Machine",
}) => {
  return (
    <ControlCard title={title}>
      <DropdownMenu>
        <DropdownMenuTrigger asChild>
          <button className="flex items-center gap-2 rounded border px-4 py-2 text-left">
            <Icon name="lu:Settings" className="text-xl" />
            <span>
              {selectedMachine?.name ?? "Select a Machine"}{" "}
              {selectedMachine?.machine_identification_unique.serial ?? ""}
            </span>
          </button>
        </DropdownMenuTrigger>
        <DropdownMenuContent align="start">
          <DropdownMenuLabel>Available Machines</DropdownMenuLabel>
          <DropdownMenuSeparator />
          {machines.map((machine) => {
            const isSelected =
              connectedMachineState?.machine_identification_unique
                ?.machine_identification.machine ===
              machine.machine_identification_unique.machine_identification
                .machine;

            return (
              <DropdownMenuItem
                key={machine.machine_identification_unique.serial}
                onClick={() =>
                  setConnectedMachine(machine.machine_identification_unique)
                }
                className={`flex min-h-[48px] cursor-pointer items-center gap-2 px-4 py-2${
                  isSelected ? "bg-blue-50" : ""
                }`}
              >
                <Icon name="lu:Settings" className="text-lg" />
                <span>
                  {machine.name} – Serial:{" "}
                  {machine.machine_identification_unique.serial}
                </span>
              </DropdownMenuItem>
            );
          })}
        </DropdownMenuContent>
      </DropdownMenu>
    </ControlCard>
  );
};
