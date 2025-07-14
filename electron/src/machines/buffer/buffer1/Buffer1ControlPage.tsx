import { Page } from "@/components/Page";
import { TouchButton } from "@/components/touch/TouchButton";
import { ControlGrid } from "@/control/ControlGrid";
import { Label } from "@/control/Label";
import React, { useEffect, useState } from "react";

import { Mode } from "./buffer1Namespace";
import { useBuffer1 } from "./useBuffer1";
import { ControlCard } from "@/control/ControlCard";
import { SelectionGroup } from "@/control/SelectionGroup";
import { TimeSeriesValueNumeric } from "@/control/TimeSeriesValue";
import { EditValue } from "@/control/EditValue";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { useMachines } from "@/client/useMachines";
import { Icon } from "@/components/Icon";
import { VENDOR_QITECH } from "@/machines/properties";

export function Buffer1ControlPage() {
  const {
    state,
    sine_wave,

    filteredMachines,
    selectedMachine,

    setBufferFrequency,
    setBufferMode,
    setConnectedMachine,
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

        <ControlCard title="Sine Wave">
          <TimeSeriesValueNumeric
            label="Current Value"
            timeseries={sine_wave}
            renderValue={(value) => value.toFixed(3)}
          />
        </ControlCard>
        <ControlCard title="Sine Wave">
          <TimeSeriesValueNumeric
            label="Current Value"
            timeseries={sine_wave}
            renderValue={(value) => value.toFixed(3)}
          />
        </ControlCard>
        <ControlCard title="Sine Wave">
          <TimeSeriesValueNumeric
            label="Current Value"
            timeseries={sine_wave}
            renderValue={(value) => value.toFixed(3)}
          />
        </ControlCard>
        <ControlCard title="Sine Wave">
          <TimeSeriesValueNumeric
            label="Current Value"
            timeseries={sine_wave}
            renderValue={(value) => value.toFixed(3)}
          />
        </ControlCard>
        <ControlCard title="Frequency">
          <div className="flex flex-col gap-4">
            <EditValue
              title="Frequency"
              unit="mHz"
              value={state?.sinewave_state.frequency}
              defaultValue={500}
              min={0.0}
              max={999}
              step={0.1}
              renderValue={(value) => value.toFixed(1)}
              onChange={setBufferFrequency}
            />
          </div>
        </ControlCard>
        <ControlCard title="Machine">
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
              {filteredMachines.map((machine) => (
                <DropdownMenuItem
                  key={machine.name}
                  onClick={() =>
                    setConnectedMachine(machine.machine_identification_unique)
                  }
                  className={`flex min-h-[48px] items-center gap-2 px-4 py-2 ${
                    state?.connected_machine_state.machine_identification_unique
                      ?.machine_identification.machine ===
                    machine.machine_identification_unique.machine_identification
                      .machine
                      ? "bg-blue-50"
                      : ""
                  }`}
                >
                  <Icon name="lu:Settings" className="text-lg" />
                  <span>
                    {machine.name} – Serial:{" "}
                    {machine.machine_identification_unique.serial}
                  </span>
                </DropdownMenuItem>
              ))}
            </DropdownMenuContent>
          </DropdownMenu>
        </ControlCard>
      </ControlGrid>
    </Page>
  );
}
