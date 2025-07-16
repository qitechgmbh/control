import { ControlCard } from "@/control/ControlCard";
import { Page } from "@/components/Page";
import React from "react";
import { ControlGrid } from "@/control/ControlGrid";
import { TimeSeriesValueNumeric } from "@/control/TimeSeriesValue";
import { EditValue } from "@/control/EditValue";
import { useMock2 } from "./useMock2";
import { SelectionGroup } from "@/control/SelectionGroup";
import { Mode } from "./mock2Namespace";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { Icon } from "@/components/Icon";
import { MachineSelector } from "@/components/MachineConnectionDropdown";

export function Mock2ControlPage() {
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
  } = useMock2();

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
        <MachineSelector
          machines={filteredMachines}
          selectedMachine={selectedMachine}
          connectedMachineState={state?.connected_machine_state}
          setConnectedMachine={setConnectedMachine}
        />
      </ControlGrid>
    </Page>
  );
}
