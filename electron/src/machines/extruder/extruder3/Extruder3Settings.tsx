import React, { useState } from "react";
import { Page } from "@/components/Page";
import { ControlCard } from "@/control/ControlCard";
import { Label } from "@/control/Label";
import { SelectionGroupBoolean } from "@/control/SelectionGroup";
import { EditValue } from "@/control/EditValue";
import { roundToDecimals } from "@/lib/decimal";
import { useExtruder3 } from "./useExtruder";
import { ControlGrid } from "@/control/ControlGrid";
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";

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
    startThermalCouplingTest,
    stopThermalCouplingTest,
  } = useExtruder3();

  const [showAdvanced, setShowAdvanced] = useState(false);
  const [tuneDelta, setTuneDelta] = useState(1.0);
  const [frequencyStepHz, setFrequencyStepHz] = useState(2.5);
  const [couplingStepSize, setCouplingStepSize] = useState(0.15);
  const [couplingMaxSettleDurationSecs, setCouplingMaxSettleDurationSecs] =
    useState(900);
  const [couplingMaxStepDurationSecs, setCouplingMaxStepDurationSecs] =
    useState(1800);
  const [
    couplingSettleThresholdCPerMin,
    setCouplingSettleThresholdCPerMin,
  ] = useState(0.5);
  const [couplingCheckIntervalSecs, setCouplingCheckIntervalSecs] =
    useState(30);
  const [couplingSettleToleranceC, setCouplingSettleToleranceC] =
    useState(0.5);

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
                    {roundToDecimals(state.pid_autotune_state.result.ki, 6)}
                    &nbsp;&nbsp; Kd:{" "}
                    {roundToDecimals(state.pid_autotune_state.result.kd, 6)}
                  </span>
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
                  step={0.000001}
                  renderValue={(v) => roundToDecimals(v, 6)}
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
                  step={0.000001}
                  renderValue={(v) => roundToDecimals(v, 6)}
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
                  max={10}
                  step={0.000001}
                  renderValue={(v) => roundToDecimals(v, 6)}
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
                  step={0.000001}
                  renderValue={(v) => roundToDecimals(v, 6)}
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
                  step={0.000001}
                  renderValue={(v) => roundToDecimals(v, 6)}
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
                  step={0.000001}
                  renderValue={(v) => roundToDecimals(v, 6)}
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
                  step={0.000001}
                  renderValue={(v) => roundToDecimals(v, 6)}
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
                  step={0.000001}
                  renderValue={(v) => roundToDecimals(v, 6)}
                  onChange={(v) => setTemperaturePidValue("nozzle", "kd", v)}
                  title="Temperature PID KD"
                />
              </Label>
            </ControlCard>
          </ControlGrid>

          <ControlCard title="Thermal Coupling Test (Debug)">
            <Alert className="mt-2 border-yellow-500/50 bg-yellow-500/10">
              <AlertTitle className="text-yellow-600">
                Debug tool — read before running
              </AlertTitle>
              <AlertDescription>
                Steps each of the 4 zones' duty cycle in turn (all 4 held
                open-loop) and measures how every zone's temperature responds,
                to help decide how to tune each zone's PID. Requires the
                extruder to already be in <b>Heat</b> mode at a stable
                temperature. Each dwell ends as soon as temperatures actually
                settle (not on a fixed timer) — the durations below are only
                safety ceilings, so a slow-diffusing barrel gets as long as it
                genuinely needs, up to{" "}
                {Math.round(
                  ((couplingMaxSettleDurationSecs +
                    couplingMaxStepDurationSecs) *
                    4) /
                    60,
                )}{" "}
                minutes worst case. The over-temperature cutoff stays active
                throughout, and you can stop it at any time.
              </AlertDescription>
            </Alert>
            <Label label="Duty Step">
              <EditValue
                value={couplingStepSize}
                defaultValue={0.15}
                title="Duty Step"
                description="Duty-cycle step applied to the zone under test (0-1)"
                min={0.05}
                max={0.5}
                step={0.01}
                renderValue={(v) => roundToDecimals(v, 2)}
                onChange={setCouplingStepSize}
              />
            </Label>
            <Label label="Settle Threshold">
              <EditValue
                value={couplingSettleThresholdCPerMin}
                defaultValue={0.5}
                title="Settle Threshold (°C/min)"
                description="Only used for the very first dwell: max rate of change to count as stable"
                min={0.05}
                max={5}
                step={0.05}
                renderValue={(v) => roundToDecimals(v, 2)}
                onChange={setCouplingSettleThresholdCPerMin}
              />
            </Label>
            <Label label="Return Tolerance">
              <EditValue
                value={couplingSettleToleranceC}
                defaultValue={0.5}
                unit="C"
                title="Return Tolerance"
                description="After reverting a step, how close to the pre-step temperature counts as fully returned"
                min={0.1}
                max={5}
                step={0.1}
                renderValue={(v) => roundToDecimals(v, 1)}
                onChange={setCouplingSettleToleranceC}
              />
            </Label>
            <Label label="Check Interval">
              <EditValue
                value={couplingCheckIntervalSecs}
                defaultValue={30}
                unit="s"
                title="Check Interval"
                description="How often to sample the rate-of-change check"
                min={5}
                max={300}
                step={5}
                renderValue={(v) => roundToDecimals(v, 0)}
                onChange={setCouplingCheckIntervalSecs}
              />
            </Label>
            <Label label="Max Settle Duration">
              <EditValue
                value={couplingMaxSettleDurationSecs}
                defaultValue={900}
                unit="s"
                title="Max Settle Duration"
                description="Safety ceiling: give up waiting to settle/return-to-baseline after this long"
                min={60}
                max={3600}
                step={30}
                renderValue={(v) => roundToDecimals(v, 0)}
                onChange={setCouplingMaxSettleDurationSecs}
              />
            </Label>
            <Label label="Max Step Duration">
              <EditValue
                value={couplingMaxStepDurationSecs}
                defaultValue={1800}
                unit="s"
                title="Max Step Duration"
                description="Safety ceiling: give up waiting for the stepped response to settle after this long"
                min={60}
                max={7200}
                step={30}
                renderValue={(v) => roundToDecimals(v, 0)}
                onChange={setCouplingMaxStepDurationSecs}
              />
            </Label>
            <Label label="Actions">
              {state?.mode_state.mode !== "Heat" && (
                <p className="mb-2 text-sm text-amber-600">
                  Extruder must be in Heat mode to start the coupling test.
                </p>
              )}
              <div className="flex gap-4">
                <button
                  onClick={() =>
                    startThermalCouplingTest({
                      stepSize: couplingStepSize,
                      maxSettleDurationSecs: couplingMaxSettleDurationSecs,
                      maxStepDurationSecs: couplingMaxStepDurationSecs,
                      settleThresholdCPerMin: couplingSettleThresholdCPerMin,
                      checkIntervalSecs: couplingCheckIntervalSecs,
                      settleToleranceC: couplingSettleToleranceC,
                    })
                  }
                  disabled={
                    state?.mode_state.mode !== "Heat" ||
                    state?.thermal_coupling_test_state.state === "settling" ||
                    state?.thermal_coupling_test_state.state === "stepping" ||
                    state?.thermal_coupling_test_state.state === "starting"
                  }
                  className="inline-block w-fit rounded bg-blue-600 px-4 py-4 text-base text-white hover:bg-blue-700 disabled:opacity-50"
                >
                  Start Coupling Test
                </button>
                <button
                  onClick={stopThermalCouplingTest}
                  disabled={
                    state?.thermal_coupling_test_state.state === "idle" ||
                    state?.thermal_coupling_test_state.state === undefined
                  }
                  className="inline-block w-fit rounded bg-red-600 px-4 py-4 text-base text-white hover:bg-red-700 disabled:opacity-50"
                >
                  Stop
                </button>
              </div>
            </Label>
            <Label label="Status">
              <div className="flex flex-col gap-2">
                <span className="text-base capitalize">
                  {(state?.thermal_coupling_test_state.state ?? "idle").replace(
                    /_/g,
                    " ",
                  )}
                  {state?.thermal_coupling_test_state.zone_under_test &&
                    ` — ${state.thermal_coupling_test_state.zone_under_test}`}
                  {` (${state?.thermal_coupling_test_state.zones_completed ?? 0}/4 zones done)`}
                  {(state?.thermal_coupling_test_state.state === "settling" ||
                    state?.thermal_coupling_test_state.state ===
                      "stepping") &&
                    (state.thermal_coupling_test_state.stable
                      ? " — stable ✓"
                      : " — waiting to settle…")}
                </span>
                {state?.thermal_coupling_test_state.duration_secs ? (
                  <>
                    <div className="h-3 w-full rounded bg-slate-200">
                      <div
                        className="h-3 rounded bg-blue-500 transition-all"
                        style={{
                          width: `${Math.min(
                            100,
                            (100 *
                              state.thermal_coupling_test_state.elapsed_secs) /
                              state.thermal_coupling_test_state.duration_secs,
                          )}%`,
                        }}
                      />
                    </div>
                    <span className="text-muted-foreground text-sm">
                      {roundToDecimals(
                        state.thermal_coupling_test_state.elapsed_secs,
                        0,
                      )}
                      s / {state.thermal_coupling_test_state.duration_secs}s
                      max
                    </span>
                  </>
                ) : null}
                {state?.thermal_coupling_test_state.error && (
                  <span className="text-sm text-red-600">
                    {state.thermal_coupling_test_state.error}
                  </span>
                )}
              </div>
            </Label>
            {state?.thermal_coupling_test_state.result && (
              <>
                <Label label="Gain Matrix (°C per unit duty step)">
                  <ThermalCouplingMatrixTable
                    zones={state.thermal_coupling_test_state.result.zones}
                    matrix={state.thermal_coupling_test_state.result.gain_matrix}
                    decimals={2}
                  />
                </Label>
                <Label label="Relative Gain Array">
                  <ThermalCouplingMatrixTable
                    zones={state.thermal_coupling_test_state.result.zones}
                    matrix={state.thermal_coupling_test_state.result.rga_matrix}
                    decimals={2}
                  />
                </Label>
              </>
            )}
          </ControlCard>
        </>
      )}
    </Page>
  );
}

function ThermalCouplingMatrixTable({
  zones,
  matrix,
  decimals,
}: {
  zones: readonly string[];
  matrix: number[][];
  decimals: number;
}) {
  return (
    <table className="border-collapse text-sm">
      <thead>
        <tr>
          <th className="border px-2 py-1 text-muted-foreground">
            out ↓ / in →
          </th>
          {zones.map((zone) => (
            <th key={zone} className="border px-2 py-1 capitalize">
              {zone}
            </th>
          ))}
        </tr>
      </thead>
      <tbody>
        {matrix.map((row, i) => (
          <tr key={zones[i]}>
            <th className="border px-2 py-1 text-left capitalize">
              {zones[i]}
            </th>
            {row.map((value, j) => (
              <td
                key={zones[j]}
                className={`border px-2 py-1 text-right ${
                  i === j ? "font-semibold" : ""
                }`}
              >
                {Number.isNaN(value) ? "—" : roundToDecimals(value, decimals)}
              </td>
            ))}
          </tr>
        ))}
      </tbody>
    </table>
  );
}
