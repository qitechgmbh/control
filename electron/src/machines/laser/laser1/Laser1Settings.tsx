import React, { useState } from "react";
import { Page } from "@/components/Page";
import { ControlCard } from "@/control/ControlCard";
import { Label } from "@/control/Label";
import { SelectionGroupBoolean } from "@/control/SelectionGroup";
import { EditValue } from "@/control/EditValue";
import { roundToDecimals } from "@/lib/decimal";
import { useLaser1 } from "./useLaser1";
import { MachineSelector } from "@/components/MachineConnectionDropdown";

export function Laser1SettingsPage() {
  const {
    state,
    defaultState,
    setSpeedPidKp,
    setSpeedPidKi,
    setSpeedPidKd,
    setSpeedPidDead,
    selectedMachine,
    filteredMachines,
    setConnectedWinder,
    disconnectWinder,
  } = useLaser1();

  const [showAdvanced, setShowAdvanced] = useState(false);

  return (
    <Page>
      <ControlCard title="Diameter Settings">
        <Label label="Show Advanced PID Settings">
          <SelectionGroupBoolean
            value={showAdvanced}
            optionTrue={{
              children: "Show",
              disabled: false,
            }}
            optionFalse={{ children: "Hide" }}
            onChange={setShowAdvanced}
          />
        </Label>
      </ControlCard>

      {showAdvanced && (
        <>
          <ControlCard title="Speed PID Settings ">
            <Label label="Kp">
              <EditValue
                value={state?.pid_settings.speed.kp}
                defaultValue={defaultState?.pid_settings.speed.kp}
                min={0}
                max={100}
                step={0.01}
                renderValue={(v) => roundToDecimals(v, 2)}
                onChange={setSpeedPidKp}
                title="Speed PID KP"
              />
            </Label>
            <Label label="Ki">
              <EditValue
                value={state?.pid_settings.speed.ki}
                defaultValue={defaultState?.pid_settings.speed.ki}
                min={0}
                max={100}
                step={0.01}
                renderValue={(v) => roundToDecimals(v, 2)}
                onChange={setSpeedPidKi}
                title="Speed PID KI"
              />
            </Label>
            <Label label="Kd">
              <EditValue
                value={state?.pid_settings.speed.kd}
                defaultValue={defaultState?.pid_settings.speed.kd}
                min={0}
                max={100}
                step={0.01}
                renderValue={(v) => roundToDecimals(v, 2)}
                onChange={setSpeedPidKd}
                title="Speed PID KD"
              />
            </Label>
            <Label label="Dead">
              <EditValue
                value={state?.pid_settings.speed.dead}
                defaultValue={defaultState?.pid_settings.speed.dead}
                min={0}
                max={100}
                step={0.01}
                renderValue={(v) => roundToDecimals(v, 2)}
                onChange={setSpeedPidKd}
                title="Speed PID Dead"
              />
            </Label>
          </ControlCard>
        </>
      )}
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
    </Page>
  );
}
