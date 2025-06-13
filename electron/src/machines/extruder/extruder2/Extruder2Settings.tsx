import React, { useEffect, useState } from "react";
import { Page } from "@/components/Page";
import { ControlCard } from "@/control/ControlCard";
import { Label } from "@/control/Label";
import { SelectionGroupBoolean } from "@/control/SelectionGroup";
import { EditValue } from "@/control/EditValue";
import { roundToDecimals } from "@/lib/decimal";
import { useExtruder2 } from "./useExtruder";

export function Extruder2SettingsPage() {
  const {
    inverterSetRotation,
    rotationState,
    extruderSetPressureLimit,
    extruderSetPressureLimitIsEnabled,
    pressureLimitState,
    pressureLimitEnabledState,
    pressurePidSettings,
    temperaturePidSettings,
    setPressurePid,
    setTemperaturePid,
  } = useExtruder2();

  const [showAdvanced, setShowAdvanced] = useState(false);

  const [pid1, setPid1] = useState<{
    kp: number;
    ki: number;
    kd: number;
  } | null>(null);
  const [pid2, setPid2] = useState<{
    kp: number;
    ki: number;
    kd: number;
  } | null>(null);

  useEffect(() => {
    if (!pid1 && temperaturePidSettings) {
      setPid1(temperaturePidSettings);
    }
    if (!pid2 && pressurePidSettings) {
      setPid2(pressurePidSettings);
    }
  }, [temperaturePidSettings, pressurePidSettings, pid1, pid2]);

  if (!pid1 || !pid2) {
    return <div>Loading PID settings...</div>;
  }

  return (
    <Page>
      <ControlCard className="bg-red" title="Inverter Settings">
        <Label label="Rotation Direction">
          <SelectionGroupBoolean
            value={rotationState}
            optionTrue={{ children: "Forward" }}
            optionFalse={{ children: "Backward" }}
            onChange={inverterSetRotation}
          />
        </Label>
      </ControlCard>

      <ControlCard className="bg-red" title="Extruder Settings">
        <Label label="Nozzle Pressure Limit">
          <EditValue
            value={pressureLimitState}
            defaultValue={0}
            unit="bar"
            title="Nozzle Pressure Limit"
            min={0}
            max={350}
            renderValue={(value) => roundToDecimals(value, 0)}
            onChange={extruderSetPressureLimit}
          />
        </Label>
        <Label label="Nozzle Pressure Limit Enabled">
          <SelectionGroupBoolean
            value={pressureLimitEnabledState}
            optionTrue={{ children: "Enabled" }}
            optionFalse={{ children: "Disabled" }}
            onChange={extruderSetPressureLimitIsEnabled}
          />
        </Label>
        <Label label="Show Advanced PID Settings">
          <SelectionGroupBoolean
            value={showAdvanced}
            optionTrue={{ children: "Show" }}
            optionFalse={{ children: "Hide" }}
            onChange={setShowAdvanced}
          />
        </Label>
      </ControlCard>

      {showAdvanced && (
        <>
          <ControlCard title="Temperature PID Settings">
            <Label label="Kp">
              <EditValue
                value={pid1.kp}
                defaultValue={0}
                min={0}
                max={100}
                step={0.01}
                renderValue={(v) => roundToDecimals(v, 2)}
                onChange={(v) => setPid1({ ...pid1, kp: v })}
                title="Temperature PID KP"
              />
            </Label>
            <Label label="Ki">
              <EditValue
                value={pid1.ki}
                defaultValue={0}
                min={0}
                max={100}
                step={0.01}
                renderValue={(v) => roundToDecimals(v, 2)}
                onChange={(v) => setPid1({ ...pid1, ki: v })}
                title="Temperature PID KI"
              />
            </Label>
            <Label label="Kd">
              <EditValue
                value={pid1.kd}
                defaultValue={0}
                min={0}
                max={100}
                step={0.01}
                renderValue={(v) => roundToDecimals(v, 2)}
                onChange={(v) => setPid1({ ...pid1, kd: v })}
                title="Temperature PID KD"
              />
            </Label>

            <div className="mt-2 flex justify-end">
              <button
                className="rounded bg-blue-600 px-4 py-2 text-white"
                onClick={() => setTemperaturePid(pid1)}
              >
                Save Temperature PID
              </button>
            </div>
          </ControlCard>

          <ControlCard title="Pressure PID Settings Group 2">
            <Label label="Kp">
              <EditValue
                value={pid2.kp}
                defaultValue={0}
                min={0}
                max={100}
                step={0.01}
                renderValue={(v) => roundToDecimals(v, 2)}
                onChange={(v) => setPid2({ ...pid2, kp: v })}
                title="Pressure PID KP"
              />
            </Label>
            <Label label="Ki">
              <EditValue
                value={pid2.ki}
                defaultValue={0}
                min={0}
                max={100}
                step={0.01}
                renderValue={(v) => roundToDecimals(v, 2)}
                onChange={(v) => setPid2({ ...pid2, ki: v })}
                title="Pressure PID KI"
              />
            </Label>
            <Label label="Kd">
              <EditValue
                value={pid2.kd}
                defaultValue={0}
                min={0}
                max={100}
                step={0.01}
                renderValue={(v) => roundToDecimals(v, 2)}
                onChange={(v) => setPid2({ ...pid2, kd: v })}
                title="Pressure PID KD"
              />
            </Label>

            <div className="mt-2 flex justify-end">
              <button
                className="rounded bg-blue-600 px-4 py-2 text-white"
                onClick={() => setPressurePid(pid2)}
              >
                Save Pressure PID
              </button>
            </div>
          </ControlCard>
        </>
      )}
    </Page>
  );
}
