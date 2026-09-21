import React, { useState } from "react";
import { Page } from "@/components/Page";
import { ControlCard } from "@/control/ControlCard";
import { Label } from "@/control/Label";
import {
  SelectionGroup,
  SelectionGroupBoolean,
} from "@/control/SelectionGroup";
import { EditValue } from "@/control/EditValue";
import { roundToDecimals } from "@/lib/decimal";
import { useExtruder3 } from "./useExtruder";
import { ControlGrid } from "@/control/ControlGrid";
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import { HeatingAlgorithm } from "./extruder3Namespace";

const TEMPERATURE_ZONES = [
  ["front", "Front"],
  ["middle", "Middle"],
  ["back", "Back"],
  ["nozzle", "Nozzle"],
] as const;

const PID_GAINS = [
  ["kp", "Kp"],
  ["ki", "Ki"],
  ["kd", "Kd"],
] as const;

const HEATING_ALGORITHM_DESCRIPTIONS: Record<HeatingAlgorithm, string> = {
  ObserverPi:
    "PI on an estimated metal temperature plus a feed-forward; Kd is unused. Almost no overshoot, when tuned correctly.",
  Pid: "Normal PID algorithm on raw sensor readings.",
};

const HEATING_ALGORITHM_CONFIRMATION =
  "Switching the heating algorithm resets all temperature PID gains to that algorithm's defaults. Continue?";

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
    setHeatingAlgorithm,
    setTemperatureTargetEnabled,
    startPressurePidAutoTune,
    stopPressurePidAutoTune,
  } = useExtruder3();

  const [showAdvanced, setShowAdvanced] = useState(false);
  const [tuneDelta, setTuneDelta] = useState(1.0);
  const [frequencyStepHz, setFrequencyStepHz] = useState(2.5);

  const heatingAlgorithm = state?.extruder_settings_state.heating_algorithm;
  // The default state holds the gains of the algorithm the machine started
  // with, so they are only defaults while that algorithm is still running.
  const temperaturePidDefaults =
    defaultState?.extruder_settings_state.heating_algorithm === heatingAlgorithm
      ? defaultState?.pid_settings.temperature
      : undefined;

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
              <Alert className="mt-2 border-yellow-500/50 bg-yellow-500/10">
                <AlertTitle className="text-yellow-600">
                  Read the Manual First
                </AlertTitle>
                <AlertDescription>
                  Please read section 2.3.1 Adaptive Pressure PID Auto-Tuning in
                  the manual for important prerequisites and step-by-step
                  instructions before using this feature.
                </AlertDescription>
              </Alert>
              <Label label="Target Pressure">
                <EditValue
                  value={state?.pressure_state.target_bar}
                  defaultValue={defaultState?.pressure_state.target_bar}
                  unit="bar"
                  title="Target Pressure for Tuning"
                  description="The pressure setpoint around which the tuner will oscillate"
                  min={0}
                  max={40}
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
                  max={5}
                  step={0.1}
                  renderValue={(v) => roundToDecimals(v, 1)}
                  onChange={setTuneDelta}
                />
              </Label>
              <Label label="Frequency Step">
                <EditValue
                  value={frequencyStepHz}
                  defaultValue={2.5}
                  title="Frequency Step (Hz)"
                  description="Inverter frequency deviation around operating point"
                  min={1}
                  max={5}
                  step={0.25}
                  renderValue={(v) => roundToDecimals(v, 2)}
                  onChange={setFrequencyStepHz}
                />
              </Label>
              <Label label="Actions">
                {state?.regulation_state.uses_rpm !== false && (
                  <p className="mb-2 text-sm text-amber-600">
                    Pressure regulation mode must be active to start auto-tune.
                  </p>
                )}
                <div className="flex gap-4">
                  <button
                    onClick={() =>
                      startPressurePidAutoTune(tuneDelta, frequencyStepHz)
                    }
                    disabled={
                      state?.regulation_state.uses_rpm !== false ||
                      state?.pid_autotune_state.state === "running"
                    }
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
                    {(state?.pid_autotune_state.state ?? "not_started").replace(
                      /_/g,
                      " ",
                    )}
                  </span>
                  <div className="h-3 w-full rounded bg-slate-200">
                    <div
                      className="h-3 rounded bg-blue-500 transition-all"
                      style={{
                        width: `${state?.pid_autotune_state.progress ?? 0}%`,
                      }}
                    />
                  </div>
                  <span className="text-muted-foreground text-sm">
                    {roundToDecimals(
                      state?.pid_autotune_state.progress ?? 0,
                      1,
                    )}
                    %
                  </span>
                </div>
              </Label>
              {state?.pid_autotune_state.result && (
                <Label label="Result">
                  <span className="text-sm">
                    Kp: {roundToDecimals(state.pid_autotune_state.result.kp, 4)}
                    &nbsp;&nbsp; Ki:{" "}
                    {roundToDecimals(state.pid_autotune_state.result.ki, 4)}
                    &nbsp;&nbsp; Kd:{" "}
                    {roundToDecimals(state.pid_autotune_state.result.kd, 4)}
                  </span>
                </Label>
              )}
            </ControlCard>
          </ControlGrid>
          <ControlCard title="Heating Algorithm">
            <SelectionGroup<HeatingAlgorithm>
              value={heatingAlgorithm}
              onChange={setHeatingAlgorithm}
              options={{
                ObserverPi: {
                  children: "Observer PI (V2 default)",
                  confirmation: HEATING_ALGORITHM_CONFIRMATION,
                },
                Pid: {
                  children: "Plain PID",
                  confirmation: HEATING_ALGORITHM_CONFIRMATION,
                },
              }}
            />
            {heatingAlgorithm && (
              <p className="text-sm text-gray-500">
                {HEATING_ALGORITHM_DESCRIPTIONS[heatingAlgorithm]}
              </p>
            )}
          </ControlCard>
          <ControlGrid>
            {TEMPERATURE_ZONES.map(([zone, zoneLabel]) => (
              <ControlCard
                key={zone}
                title={`Temperature PID Settings (${zoneLabel})`}
              >
                {PID_GAINS.map(([gain, gainLabel]) => (
                  <Label key={gain} label={gainLabel}>
                    <EditValue
                      value={state?.pid_settings.temperature[zone][gain]}
                      defaultValue={temperaturePidDefaults?.[zone][gain]}
                      min={0}
                      max={1}
                      step={0.0001}
                      exactEditValue
                      renderValue={(v) => roundToDecimals(v, 4)}
                      onChange={(v) => setTemperaturePidValue(zone, gain, v)}
                      title={`Temperature PID ${gainLabel.toUpperCase()}`}
                      // The observer's loop is a PI; its Kd is never used
                      disabled={
                        gain === "kd" && heatingAlgorithm === "ObserverPi"
                      }
                    />
                  </Label>
                ))}
              </ControlCard>
            ))}
          </ControlGrid>
        </>
      )}
    </Page>
  );
}
