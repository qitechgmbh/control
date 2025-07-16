import { ControlCard } from "@/control/ControlCard";
import { Page } from "@/components/Page";
import React from "react";
import { ControlGrid } from "@/control/ControlGrid";
import { TimeSeriesValueNumeric } from "@/control/TimeSeriesValue";
import { EditValue } from "@/control/EditValue";
import { useMock1 } from "./useMock";
import { SelectionGroup } from "@/control/SelectionGroup";
import { Mode } from "./mock1Namespace";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { Icon } from "@/components/Icon";

export function Mock1ControlPage() {
  const {
    state,
    defaultState,
    sineWave,
    selectedMachine,
    filteredMachines,
    setFrequency,
    setMode,
    isDisabled,
    setConnectedMachine,
  } = useMock1();

  // Controlled local states synced with consolidated state
  const frequency = state?.sine_wave_state?.frequency ?? 1.0;
  const mode = state?.mode_state?.mode ?? "Standby";

  return (
    <Page>
      <ControlGrid>
        <ControlCard title="Sine Wave">
          <TimeSeriesValueNumeric
            label="Current Value"
            timeseries={sineWave}
            renderValue={(value) => value.toFixed(3)}
          />
        </ControlCard>
        <ControlCard title="Sine Wave">
          <TimeSeriesValueNumeric
            label="Current Value"
            timeseries={sineWave}
            renderValue={(value) => value.toFixed(3)}
          />
        </ControlCard>
        <ControlCard title="Sine Wave">
          <TimeSeriesValueNumeric
            label="Current Value"
            timeseries={sineWave}
            renderValue={(value) => value.toFixed(3)}
          />
        </ControlCard>
        <ControlCard title="Sine Wave">
          <TimeSeriesValueNumeric
            label="Current Value"
            timeseries={sineWave}
            renderValue={(value) => value.toFixed(3)}
          />
        </ControlCard>

        <ControlCard title="Frequency">
          <div className="flex flex-col gap-4">
            <EditValue
              title="Frequency"
              unit="mHz"
              value={frequency}
              defaultValue={defaultState?.sine_wave_state.frequency}
              min={0.0}
              max={1000}
              step={0.1}
              renderValue={(value) => value.toFixed(0)}
              onChange={setFrequency}
            />
          </div>
        </ControlCard>

        <ControlCard title="Mode">
          <div className="flex flex-col gap-2">
            <div className="text-sm font-medium">Mode</div>
            <SelectionGroup
              value={mode}
              onChange={(newMode: Mode) => setMode(newMode)}
              disabled={isDisabled}
              options={{
                Standby: { children: "Standby" },
                Running: { children: "Running" },
              }}
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
                  key={machine.machine_identification_unique.serial}
                  onClick={() =>
                    setConnectedMachine(machine.machine_identification_unique)
                  }
                  className={`flex min-h-[48px] cursor-pointer items-center gap-2 px-4 py-2 cursor pointer${
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
