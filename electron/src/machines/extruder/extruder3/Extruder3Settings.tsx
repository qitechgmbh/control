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
    setInverterTargetPressure,
    setPressurePidKp,
    setPressurePidKi,
    setPressurePidKd,
    setTemperaturePidValue,
    setTemperatureTargetEnabled,
    startPressurePidAutoTune,
    stopPressurePidAutoTune,
  } = useExtruder3();

  const [showAdvanced, setShowAdvanced] = useState(false);
  const [tuneDelta, setTuneDelta] = useState(1.0);
  const [frequencyStepHz, setFrequencyStepHz] = useState(5.0);

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
        <Label label="Nozzle Temperature Target Enabled">
          <SelectionGroupBoolean
            value={
              state?.extruder_settings_state.nozzle_temperature_target_enabled
            }
            optionTrue={{ children: "Enabled" }}
            optionFalse={{ children: "Disabled" }}
            onChange={setTemperatureTargetEnabled}
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
          <ControlGrid columns={2}>
            <ControlCard title="Pressure PID Settings">
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
            <ControlCard title="Pressure PID Auto-Tune">
              <Label label="Target Pressure">
                <EditValue
                  value={state?.pressure_state.target_bar}
                  defaultValue={defaultState?.pressure_state.target_bar}
                  unit="bar"
                  title="Target Pressure for Tuning"
                  description="The pressure setpoint around which the tuner will oscillate"
                  min={0}
                  max={350}
                  renderValue={(v) => roundToDecimals(v, 1)}
                  onChange={setInverterTargetPressure}
                />
              </Label>
              <Label label="Tune Delta">
                <EditValue
                  value={tuneDelta}
                  defaultValue={1.0}
                  unit="bar"
                  title="Tune Delta"
                  description="Allowed pressure oscillation band around target"
                  min={0.1}
                  max={10}
                  step={0.1}
                  renderValue={(v) => roundToDecimals(v, 1)}
                  onChange={setTuneDelta}
                />
              </Label>
              <Label label="Frequency Step">
                <EditValue
                  value={frequencyStepHz}
                  defaultValue={5.0}
                  title="Frequency Step (Hz)"
                  description="Inverter frequency deviation around operating point"
                  min={1}
                  max={20}
                  step={0.5}
                  renderValue={(v) => roundToDecimals(v, 1)}
                  onChange={setFrequencyStepHz}
                />
              </Label>
              <Label label="Actions">
                <div className="flex gap-4">
                  <button
                    onClick={() => startPressurePidAutoTune(tuneDelta, frequencyStepHz)}
                    disabled={state?.pid_autotune_state.state === "running"}
                    className="inline-block w-fit rounded bg-blue-600 px-4 py-4 text-base text-white hover:bg-blue-700 disabled:opacity-50"
                  >
                    Start Auto-Tune
                  </button>
                  <button
                    onClick={stopPressurePidAutoTune}
                    disabled={state?.pid_autotune_state.state !== "running"}
                    className="inline-block w-fit rounded bg-red-600 px-4 py-4 text-base text-white hover:bg-red-700 disabled:opacity-50"
                  >
                    Stop
                  </button>
                </div>
              </Label>
              <Label label="Status">
                <div className="flex flex-col gap-2">
                  <span className="text-base capitalize">
                    {(state?.pid_autotune_state.state ?? "not_started").replace(/_/g, " ")}
                  </span>
                  <div className="h-3 w-full rounded bg-slate-200">
                    <div
                      className="h-3 rounded bg-blue-500 transition-all"
                      style={{ width: `${state?.pid_autotune_state.progress ?? 0}%` }}
                    />
                  </div>
                  <span className="text-sm text-muted-foreground">
                    {roundToDecimals(state?.pid_autotune_state.progress ?? 0, 1)}%
                  </span>
                </div>
              </Label>
              {state?.pid_autotune_state.result && (
                <Label label="Result">
                  <div className="flex flex-col gap-2">
                    <span className="text-sm">
                      Kp: {roundToDecimals(state.pid_autotune_state.result.kp, 4)}&nbsp;&nbsp;
                      Ki: {roundToDecimals(state.pid_autotune_state.result.ki, 4)}&nbsp;&nbsp;
                      Kd: {roundToDecimals(state.pid_autotune_state.result.kd, 4)}
                    </span>
                    <button
                      onClick={() => {
                        const r = state.pid_autotune_state.result!;
                        setPressurePidKp(r.kp);
                        setPressurePidKi(r.ki);
                        setPressurePidKd(r.kd);
                      }}
                      className="inline-block w-fit rounded bg-green-600 px-4 py-4 text-base text-white hover:bg-green-700"
                    >
                      Apply to PID Settings
                    </button>
                  </div>
                </Label>
              )}
            </ControlCard>
          </ControlGrid>
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
