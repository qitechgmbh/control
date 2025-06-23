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
    inverterReset,
    rotationState,
    extruderSetPressureLimit,
    extruderSetPressureLimitIsEnabled,
    pressureLimitState,
    pressureLimitEnabledState,
    pressurePidSettings,
    setPressurePid,
  } = useExtruder2();

  const [showAdvanced, setShowAdvanced] = useState(false);

  const [pid2, setPid2] = useState<{
    kp: number;
    ki: number;
    kd: number;
  } | null>(null);

  useEffect(() => {
    if (!pid2 && pressurePidSettings) {
      setPid2(pressurePidSettings);
    }
  }, [pressurePidSettings, pid2]);

  if (!pid2) {
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

        <Label label="Reset Inverter">
          <button
            onClick={inverterReset}
            className="inline-block w-fit max-w-max rounded bg-red-600 px-4 py-4 text-base whitespace-nowrap text-white hover:bg-red-700"
            style={{ minWidth: "auto", width: "fit-content" }}
          >
            Reset Inverter
          </button>
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
          <ControlCard title="Pressure PID Settings ">
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
