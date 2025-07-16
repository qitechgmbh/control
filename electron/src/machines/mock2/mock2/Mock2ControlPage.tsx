import { ControlCard } from "@/control/ControlCard";
import { Page } from "@/components/Page";
import React from "react";
import { ControlGrid } from "@/control/ControlGrid";
import { TimeSeriesValueNumeric } from "@/control/TimeSeriesValue";
import { EditValue } from "@/control/EditValue";
import { useMock2 } from "./useMock2";
import { SelectionGroup } from "@/control/SelectionGroup";
import { Mode } from "./mock2Namespace";
import { MachineSelector } from "@/components/MachineConnectionDropdown";

export function Mock2ControlPage() {
  const {
    state,
    defaultState,
    sineWave,
    selectedMachine,
    filteredMachines,
    setFrequency,
    setConnectedMachineFrequency,
    setMode,
    isDisabled,
    setConnectedMachine,
    disconnectMachine,
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

        <ControlCard title="Connected Mock Frequency">
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
              onChange={setConnectedMachineFrequency}
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
