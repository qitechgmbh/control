import React, { useState } from "react";
import { Page } from "@/components/Page";
import { ControlCard } from "@/control/ControlCard";
import { Label } from "@/control/Label";
import { SelectionGroupBoolean } from "@/control/SelectionGroup";
import { EditValue } from "@/control/EditValue";
import { roundToDecimals } from "@/lib/decimal";
import { useExtruder3 } from "./useExtruder";
import { ControlGrid } from "@/control/ControlGrid";

export function Extruder3SettingsPage() {
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
    setTemperaturePidValue,
  } = useExtruder3();

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
                  step={0.001}
                  renderValue={(v) => roundToDecimals(v, 3)}
                  onChange={(v) => setTemperaturePidValue("front", "kp", v)}
                  title="Temperature PID KP"
                />
              </Label>
              <Label label="Ki">
                <EditValue
                  value={state?.pid_settings.temperature.front.ki}
                  defaultValue={defaultState?.pid_settings.temperature.front.ki}
                  min={0}
                  max={100}
                  step={0.001}
                  renderValue={(v) => roundToDecimals(v, 3)}
                  onChange={(v) => setTemperaturePidValue("front", "ki", v)}
                  title="Temperature PID KI"
                />
              </Label>
              <Label label="Kd">
                <EditValue
                  value={state?.pid_settings.temperature.front.kd}
                  defaultValue={defaultState?.pid_settings.temperature.front.kd}
                  min={0}
                  max={100}
                  step={0.001}
                  renderValue={(v) => roundToDecimals(v, 3)}
                  onChange={(v) => setTemperaturePidValue("front", "kd", v)}
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
                  step={0.001}
                  renderValue={(v) => roundToDecimals(v, 3)}
                  onChange={(v) => setTemperaturePidValue("middle", "kp", v)}
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
                  step={0.001}
                  renderValue={(v) => roundToDecimals(v, 3)}
                  onChange={(v) => setTemperaturePidValue("middle", "ki", v)}
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
                  step={0.001}
                  renderValue={(v) => roundToDecimals(v, 3)}
                  onChange={(v) => setTemperaturePidValue("middle", "kd", v)}
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
                  step={0.001}
                  renderValue={(v) => roundToDecimals(v, 3)}
                  onChange={(v) => setTemperaturePidValue("back", "kp", v)}
                  title="Temperature PID KP"
                />
              </Label>
              <Label label="Ki">
                <EditValue
                  value={state?.pid_settings.temperature.back.ki}
                  defaultValue={defaultState?.pid_settings.temperature.back.ki}
                  min={0}
                  max={100}
                  step={0.001}
                  renderValue={(v) => roundToDecimals(v, 3)}
                  onChange={(v) => setTemperaturePidValue("back", "ki", v)}
                  title="Temperature PID KI"
                />
              </Label>
              <Label label="Kd">
                <EditValue
                  value={state?.pid_settings.temperature.back.kd}
                  defaultValue={defaultState?.pid_settings.temperature.back.kd}
                  min={0}
                  max={100}
                  step={0.001}
                  renderValue={(v) => roundToDecimals(v, 3)}
                  onChange={(v) => setTemperaturePidValue("back", "kd", v)}
                  title="Temperature PID KD"
                />
              </Label>
            </ControlCard>
            <ControlCard title="Temperature PID Settings (Nozzle) ">
              <Label label="Kp">
                <EditValue
                  value={state?.pid_settings.temperature.nozzle.kp}
                  defaultValue={
                    defaultState?.pid_settings.temperature.nozzle.kp
                  }
                  min={0}
                  max={100}
                  step={0.001}
                  renderValue={(v) => roundToDecimals(v, 3)}
                  onChange={(v) => setTemperaturePidValue("nozzle", "kp", v)}
                  title="Temperature PID KP"
                />
              </Label>
              <Label label="Ki">
                <EditValue
                  value={state?.pid_settings.temperature.nozzle.ki}
                  defaultValue={
                    defaultState?.pid_settings.temperature.nozzle.ki
                  }
                  min={0}
                  max={100}
                  step={0.001}
                  renderValue={(v) => roundToDecimals(v, 3)}
                  onChange={(v) => setTemperaturePidValue("nozzle", "ki", v)}
                  title="Temperature PID KI"
                />
              </Label>
              <Label label="Kd">
                <EditValue
                  value={state?.pid_settings.temperature.nozzle.kd}
                  defaultValue={
                    defaultState?.pid_settings.temperature.nozzle.kd
                  }
                  min={0}
                  max={100}
                  step={0.001}
                  renderValue={(v) => roundToDecimals(v, 3)}
                  onChange={(v) => setTemperaturePidValue("nozzle", "kd", v)}
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
