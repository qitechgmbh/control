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
import { TouchButton } from "./touch/TouchButton";

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
    is_available: boolean;
  };
  setConnectedMachine: (machine: MachineIdentificationUnique) => void;
  clearConnectedMachine?: () => void;
  title?: string;
}

export const MachineSelector: React.FC<MachineSelectorProps> = ({
  machines,
  selectedMachine,
  connectedMachineState,
  setConnectedMachine,
  clearConnectedMachine,
}) => {
  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>
        <TouchButton className="text-md h-max px-6 py-6" variant="outline">
          <div className="flex flex-row items-center gap-2 text-wrap">
            <Icon name="lu:Link2" className="size-6" />
            {selectedMachine?.name ?? "Select a Machine"}{" "}
            {selectedMachine?.machine_identification_unique.serial ?? ""}
          </div>
        </TouchButton>
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
              className={`flex h-max cursor-pointer items-center gap-2 px-6 py-6${
                isSelected ? "bg-blue-50" : ""
              }`}
            >
              {connectedMachineState?.is_available ? (
                <Icon name="lu:Link2" className="size-6 text-green-600" />
              ) : (
                <Icon name="lu:Link2Off" className="size-6 text-black" />
              )}
              <span>
                {machine.name} – Serial:{" "}
                {machine.machine_identification_unique.serial}
              </span>
            </DropdownMenuItem>
          );
        })}

        {clearConnectedMachine && (
          <>
            <DropdownMenuSeparator />
            <DropdownMenuItem
              onClick={clearConnectedMachine}
              className="flex cursor-pointer items-center gap-2 text-red-600 hover:text-red-600"
            >
              <Icon name="lu:X" className="size-6 text-red-600" />
              <span>Disconnect Machine</span>
            </DropdownMenuItem>
          </>
        )}
      </DropdownMenuContent>
    </DropdownMenu>
  );
};
