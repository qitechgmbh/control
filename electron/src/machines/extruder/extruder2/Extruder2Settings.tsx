import React, { useState } from "react";
import { Page } from "@/components/Page";
import { ControlCard } from "@/control/ControlCard";
import { Label } from "@/control/Label";
import { SelectionGroupBoolean } from "@/control/SelectionGroup";
import { EditValue } from "@/control/EditValue";
import { roundToDecimals } from "@/lib/decimal";
import { useExtruder2 } from "./useExtruder";
import { ControlGrid } from "@/control/ControlGrid";

export function Extruder2SettingsPage() {
  const {
    state,
    defaultState,
    setInverterRotationDirection,
    resetInverter,
    setExtruderPressureLimit,
    setExtruderPressureLimitEnabled,
    setPressurePidKp,
    setPressurePidKi,
    setPressurePidKd,
    setTemperaturePidKpFront,
    setTemperaturePidKiFront,
    setTemperaturePidKdFront,
    setTemperaturePidKpMiddle,
    setTemperaturePidKiMiddle,
    setTemperaturePidKdMiddle,
    setTemperaturePidKpBack,
    setTemperaturePidKiBack,
    setTemperaturePidKdBack,
  } = useExtruder2();

  const [showAdvanced, setShowAdvanced] = useState(false);

  return (
    <Page>
      <ControlCard className="bg-red" title="Inverter Settings">
        <Label label="Rotation Direction">
          <SelectionGroupBoolean
            value={state?.rotation_state.forward}
            optionTrue={{ children: "Forward" }}
            optionFalse={{ children: "Backward" }}
            onChange={setInverterRotationDirection}
          />
        </Label>

        <Label label="Reset Inverter">
          <button
            onClick={resetInverter}
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
            value={state?.extruder_settings_state.pressure_limit}
            defaultValue={defaultState?.extruder_settings_state.pressure_limit}
            unit="bar"
            title="Nozzle Pressure Limit"
            min={0}
            max={350}
            renderValue={(value) => roundToDecimals(value, 0)}
            onChange={setExtruderPressureLimit}
          />
        </Label>
        <Label label="Nozzle Pressure Limit Enabled">
          <SelectionGroupBoolean
            value={state?.extruder_settings_state.pressure_limit_enabled}
            optionTrue={{ children: "Enabled" }}
            optionFalse={{ children: "Disabled" }}
            onChange={setExtruderPressureLimitEnabled}
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
                value={state?.pid_settings.pressure.kp}
                defaultValue={defaultState?.pid_settings.pressure.kp}
                min={0}
                max={100}
                step={0.01}
                renderValue={(v) => roundToDecimals(v, 2)}
                onChange={setPressurePidKp}
                title="Pressure PID KP"
              />
            </Label>
            <Label label="Ki">
              <EditValue
                value={state?.pid_settings.pressure.ki}
                defaultValue={defaultState?.pid_settings.pressure.ki}
                min={0}
                max={100}
                step={0.01}
                renderValue={(v) => roundToDecimals(v, 2)}
                onChange={setPressurePidKi}
                title="Pressure PID KI"
              />
            </Label>
            <Label label="Kd">
              <EditValue
                value={state?.pid_settings.pressure.kd}
                defaultValue={defaultState?.pid_settings.pressure.kd}
                min={0}
                max={100}
                step={0.01}
                renderValue={(v) => roundToDecimals(v, 2)}
                onChange={setPressurePidKd}
                title="Pressure PID KD"
              />
            </Label>
          </ControlCard>
          <ControlGrid>
            <ControlCard title="Temperature PID Settings (Front) ">
              <Label label="Kp">
                <EditValue
                  value={state?.pid_settings.temperature.front.kp}
                  defaultValue={defaultState?.pid_settings.temperature.front.kp}
                  min={0}
                  max={100}
                  step={0.01}
                  renderValue={(v) => roundToDecimals(v, 2)}
                  onChange={setTemperaturePidKpFront}
                  title="Temperature PID KP"
                />
              </Label>
              <Label label="Ki">
                <EditValue
                  value={state?.pid_settings.temperature.front.ki}
                  defaultValue={defaultState?.pid_settings.temperature.front.ki}
                  min={0}
                  max={100}
                  step={0.01}
                  renderValue={(v) => roundToDecimals(v, 2)}
                  onChange={setTemperaturePidKiFront}
                  title="Temperature PID KI"
                />
              </Label>
              <Label label="Kd">
                <EditValue
                  value={state?.pid_settings.temperature.front.kd}
                  defaultValue={defaultState?.pid_settings.temperature.front.kd}
                  min={0}
                  max={100}
                  step={0.01}
                  renderValue={(v) => roundToDecimals(v, 2)}
                  onChange={setTemperaturePidKdFront}
                  title="Temperature PID KD"
                />
              </Label>
            </ControlCard>
            <ControlCard title="Temperature PID Settings (Middle) ">
              <Label label="Kp">
                <EditValue
                  value={state?.pid_settings.temperature.middle.kp}
                  defaultValue={
                    defaultState?.pid_settings.temperature.middle.kp
                  }
                  min={0}
                  max={100}
                  step={0.01}
                  renderValue={(v) => roundToDecimals(v, 2)}
                  onChange={setTemperaturePidKpMiddle}
                  title="Temperature PID KP"
                />
              </Label>
              <Label label="Ki">
                <EditValue
                  value={state?.pid_settings.temperature.middle.ki}
                  defaultValue={
                    defaultState?.pid_settings.temperature.middle.ki
                  }
                  min={0}
                  max={100}
                  step={0.01}
                  renderValue={(v) => roundToDecimals(v, 2)}
                  onChange={setTemperaturePidKiMiddle}
                  title="Temperature PID KI"
                />
              </Label>
              <Label label="Kd">
                <EditValue
                  value={state?.pid_settings.temperature.middle.kd}
                  defaultValue={
                    defaultState?.pid_settings.temperature.middle.kd
                  }
                  min={0}
                  max={100}
                  step={0.01}
                  renderValue={(v) => roundToDecimals(v, 2)}
                  onChange={setTemperaturePidKdMiddle}
                  title="Temperature PID KD"
                />
              </Label>
            </ControlCard>
            <ControlCard title="Temperature PID Settings (Back) ">
              <Label label="Kp">
                <EditValue
                  value={state?.pid_settings.temperature.back.kp}
                  defaultValue={defaultState?.pid_settings.temperature.back.kp}
                  min={0}
                  max={100}
                  step={0.01}
                  renderValue={(v) => roundToDecimals(v, 2)}
                  onChange={setTemperaturePidKpBack}
                  title="Temperature PID KP"
                />
              </Label>
              <Label label="Ki">
                <EditValue
                  value={state?.pid_settings.temperature.back.ki}
                  defaultValue={defaultState?.pid_settings.temperature.back.ki}
                  min={0}
                  max={100}
                  step={0.01}
                  renderValue={(v) => roundToDecimals(v, 2)}
                  onChange={setTemperaturePidKiBack}
                  title="Temperature PID KI"
                />
              </Label>
              <Label label="Kd">
                <EditValue
                  value={state?.pid_settings.temperature.back.kd}
                  defaultValue={defaultState?.pid_settings.temperature.back.kd}
                  min={0}
                  max={100}
                  step={0.01}
                  renderValue={(v) => roundToDecimals(v, 2)}
                  onChange={setTemperaturePidKdBack}
                  title="Temperature PID KD"
                />
              </Label>
            </ControlCard>
          </ControlGrid>
        </>
      )}
    </Page>
  );
}
