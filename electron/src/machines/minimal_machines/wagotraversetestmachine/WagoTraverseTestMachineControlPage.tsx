import { Page } from "@/components/Page";
import { ControlCard } from "@/control/ControlCard";
import { ControlGrid } from "@/control/ControlGrid";
import { EditValue } from "@/control/EditValue";
import { Label } from "@/control/Label";
import { SelectionGroup } from "@/control/SelectionGroup";
import { roundToDecimals } from "@/lib/decimal";
import React from "react";
import { useWagoTraverseTestMachine } from "./useWagoTraverseTestMachine";

const defaultState = {
  enabled: false,
  mode: "Standby",
  control_mode: "Idle",
  controller_state: "NotHomed",
  is_homed: false,
  speed_mode_ack: false,
  di1: false,
  di2: false,
  switch_output_on: false,
  target_velocity_register: 0,
  target_speed_steps_per_second: 0,
  actual_velocity_register: 0,
  actual_speed_steps_per_second: 0,
  actual_speed_mm_per_second: 0,
  reference_mode_ack: false,
  reference_ok: false,
  busy: false,
  target_acceleration: 10000,
  speed_scale: 1,
  direction_multiplier: 1,
  freq_range_sel: 1,
  acc_range_sel: 0,
  raw_position_steps: 0,
  wrapper_position_steps: 0,
  raw_position_mm: 0,
  wrapper_position_mm: 0,
  controller_position_mm: null,
  limit_inner_mm: 22,
  limit_outer_mm: 92,
  manual_speed_mm_per_second: 0,
  manual_velocity_register: 5000,
  control_byte1: 0,
  control_byte2: 0,
  control_byte3: 0,
  status_byte1: 0,
  status_byte2: 0,
  status_byte3: 0,
};

export function WagoTraverseTestMachineControlPage() {
  const {
    state,
    setMode,
    setEnabled,
    setManualSpeedMmPerSecond,
    setManualVelocityRegister,
    jogRawPositive,
    jogRawNegative,
    jogRawPositiveFast,
    jogRawNegativeFast,
    jogMmPositive,
    jogMmNegative,
    jogMmPositiveFast,
    jogMmNegativeFast,
    stop,
    setSwitchOutput,
    gotoHome,
    gotoLimitInner,
    gotoLimitOuter,
    forceNotHomed,
    setPositionMm,
    setLimitInnerMm,
    setLimitOuterMm,
    setSpeedScale,
    setDirectionMultiplier,
    setFreqRangeSel,
    setAccRangeSel,
    setAcceleration,
  } = useWagoTraverseTestMachine();

  const safeState = state ?? defaultState;
  const isManualRaw = safeState.control_mode === "ManualVelocityRegister";
  const isManualMm = safeState.control_mode === "ManualMmPerSecond";
  const isManual = isManualRaw || isManualMm;
  const isController = safeState.control_mode === "Controller";
  const displayPositionMm =
    safeState.controller_position_mm ?? safeState.wrapper_position_mm;

  return (
    <Page>
      <ControlGrid columns={2}>
        <ControlCard title="Traverse">
          <div className="space-y-4">
            <Label label="Enable">
              <SelectionGroup<"Standby" | "Hold">
                value={safeState.mode === "Hold" ? "Hold" : "Standby"}
                orientation="horizontal"
                options={{
                  Standby: { children: "Standby" },
                  Hold: { children: "Hold" },
                }}
                onChange={(value) => setMode(value)}
              />
            </Label>

            <div className="grid grid-cols-2 gap-4 text-sm">
              <div>State: {safeState.controller_state}</div>
              <div>Mode: {safeState.mode}</div>
              <div>Drive: {safeState.control_mode}</div>
              <div>Homed: {safeState.is_homed ? "Yes" : "No"}</div>
              <div
                className={`rounded px-3 py-2 text-white ${safeState.speed_mode_ack ? "bg-green-600" : "bg-red-600"}`}
              >
                Speed Ack: {safeState.speed_mode_ack ? "On" : "Off"}
              </div>
              <div>
                Target reg:{" "}
                {roundToDecimals(safeState.target_velocity_register, 0)}
              </div>
              <div>
                Actual reg:{" "}
                {roundToDecimals(safeState.actual_velocity_register, 0)}
              </div>
              <div
                className={`rounded px-3 py-2 text-white ${safeState.reference_mode_ack ? "bg-green-600" : "bg-slate-500"}`}
              >
                Ref Ack: {safeState.reference_mode_ack ? "On" : "Off"}
              </div>
              <div
                className={`rounded px-3 py-2 text-white ${safeState.di1 ? "bg-green-600" : "bg-red-600"}`}
              >
                DI1: {safeState.di1 ? "On" : "Off"}
              </div>
              <div
                className={`rounded px-3 py-2 text-white ${safeState.di2 ? "bg-green-600" : "bg-red-600"}`}
              >
                DI2: {safeState.di2 ? "On" : "Off"}
              </div>
              <div
                className={`rounded px-3 py-2 text-white ${safeState.switch_output_on ? "bg-blue-600" : "bg-slate-500"}`}
              >
                Switch DO: {safeState.switch_output_on ? "On" : "Off"}
              </div>
              <div
                className={`rounded px-3 py-2 text-white ${safeState.reference_ok ? "bg-green-600" : "bg-slate-500"}`}
              >
                Ref OK: {safeState.reference_ok ? "On" : "Off"}
              </div>
              <div
                className={`rounded px-3 py-2 text-white ${safeState.busy ? "bg-amber-600" : "bg-slate-500"}`}
              >
                Busy: {safeState.busy ? "On" : "Off"}
              </div>
              <div>Position mm: {roundToDecimals(displayPositionMm, 3)}</div>
              <div>
                Actual mm/s:{" "}
                {roundToDecimals(safeState.actual_speed_mm_per_second, 2)}
              </div>
              <div>
                Inner limit mm: {roundToDecimals(safeState.limit_inner_mm, 3)}
              </div>
              <div>
                Outer limit mm: {roundToDecimals(safeState.limit_outer_mm, 3)}
              </div>
            </div>

            <div className="grid grid-cols-2 gap-2">
              <button
                className={`rounded px-3 py-2 text-sm text-white ${!safeState.enabled ? "bg-slate-400" : safeState.switch_output_on ? "bg-blue-700" : "bg-slate-700"}`}
                disabled={!safeState.enabled}
                onClick={() => setSwitchOutput(!safeState.switch_output_on)}
              >
                {safeState.switch_output_on ? "Release DI2" : "Trigger DI2"}
              </button>
              <button
                className={`rounded px-3 py-2 text-sm text-white ${!safeState.enabled || isManual ? "bg-slate-400" : "bg-slate-700"}`}
                disabled={!safeState.enabled || isManual}
                onClick={gotoHome}
              >
                Go Home
              </button>
              <button
                className={`rounded px-3 py-2 text-sm text-white ${!safeState.enabled || isManual || !safeState.is_homed ? "bg-slate-400" : "bg-slate-700"}`}
                disabled={!safeState.enabled || isManual || !safeState.is_homed}
                onClick={gotoLimitInner}
              >
                Go Inner
              </button>
              <button
                className={`rounded px-3 py-2 text-sm text-white ${!safeState.enabled || isManual || !safeState.is_homed ? "bg-slate-400" : "bg-slate-700"}`}
                disabled={!safeState.enabled || isManual || !safeState.is_homed}
                onClick={gotoLimitOuter}
              >
                Go Outer
              </button>
              <button
                className={`rounded px-3 py-2 text-sm text-white ${isManual || isController ? "bg-slate-400" : "bg-amber-700"}`}
                disabled={isManual || isController}
                onClick={forceNotHomed}
              >
                Force Not Homed
              </button>
              <button
                className={`rounded px-3 py-2 text-sm text-white ${!safeState.enabled ? "bg-slate-400" : "bg-red-700"}`}
                disabled={!safeState.enabled}
                onClick={stop}
              >
                Stop
              </button>
            </div>
          </div>
        </ControlCard>

        <ControlCard title="Calibration">
          <div className="space-y-4">
            <Label label="Inner Limit mm">
              <EditValue
                value={safeState.limit_inner_mm}
                title="Inner Limit mm"
                defaultValue={22}
                step={0.1}
                renderValue={(value) => roundToDecimals(value, 3)}
                onChange={setLimitInnerMm}
              />
            </Label>

            <Label label="Outer Limit mm">
              <EditValue
                value={safeState.limit_outer_mm}
                title="Outer Limit mm"
                defaultValue={92}
                step={0.1}
                renderValue={(value) => roundToDecimals(value, 3)}
                onChange={setLimitOuterMm}
              />
            </Label>

            <details className="rounded border border-slate-200 p-3">
              <summary className="cursor-pointer text-sm font-medium text-slate-700">
                Advanced
              </summary>

              <div className="mt-4 space-y-4">
                <div className="grid grid-cols-2 gap-4 text-sm">
                  <div>
                    Raw mm: {roundToDecimals(safeState.raw_position_mm, 3)}
                  </div>
                  <div>
                    Wrapper mm:{" "}
                    {roundToDecimals(safeState.wrapper_position_mm, 3)}
                  </div>
                  <div>Raw steps: {safeState.raw_position_steps}</div>
                  <div>
                    Actual reg:{" "}
                    {roundToDecimals(safeState.actual_velocity_register, 0)}
                  </div>
                  <div>
                    C1: 0x
                    {safeState.control_byte1.toString(16).padStart(2, "0")}
                  </div>
                  <div>
                    C2: 0x
                    {safeState.control_byte2.toString(16).padStart(2, "0")}
                  </div>
                  <div>
                    C3: 0x
                    {safeState.control_byte3.toString(16).padStart(2, "0")}
                  </div>
                  <div>
                    S1: 0x{safeState.status_byte1.toString(16).padStart(2, "0")}
                  </div>
                  <div>
                    S2: 0x{safeState.status_byte2.toString(16).padStart(2, "0")}
                  </div>
                  <div>
                    S3: 0x{safeState.status_byte3.toString(16).padStart(2, "0")}
                  </div>
                </div>

                <div className="grid grid-cols-2 gap-2">
                  <button
                    className={`rounded px-3 py-2 text-sm text-white ${!safeState.enabled || isController || isManualMm ? "bg-slate-400" : "bg-slate-700"}`}
                    disabled={!safeState.enabled || isController || isManualMm}
                    onClick={() => jogRawNegative()}
                  >
                    Raw -
                  </button>
                  <button
                    className={`rounded px-3 py-2 text-sm text-white ${!safeState.enabled || isController || isManualMm ? "bg-slate-400" : "bg-slate-700"}`}
                    disabled={!safeState.enabled || isController || isManualMm}
                    onClick={() => jogRawPositive()}
                  >
                    Raw +
                  </button>
                  <button
                    className={`rounded px-3 py-2 text-sm text-white ${!safeState.enabled || isController || isManualMm ? "bg-slate-400" : "bg-slate-700"}`}
                    disabled={!safeState.enabled || isController || isManualMm}
                    onClick={() => jogRawNegativeFast()}
                  >
                    Raw Fast -
                  </button>
                  <button
                    className={`rounded px-3 py-2 text-sm text-white ${!safeState.enabled || isController || isManualMm ? "bg-slate-400" : "bg-slate-700"}`}
                    disabled={!safeState.enabled || isController || isManualMm}
                    onClick={() => jogRawPositiveFast()}
                  >
                    Raw Fast +
                  </button>
                  <button
                    className={`rounded px-3 py-2 text-sm text-white ${!safeState.enabled || isController || isManualRaw ? "bg-slate-400" : "bg-slate-700"}`}
                    disabled={!safeState.enabled || isController || isManualRaw}
                    onClick={() => jogMmNegative()}
                  >
                    mm/s -
                  </button>
                  <button
                    className={`rounded px-3 py-2 text-sm text-white ${!safeState.enabled || isController || isManualRaw ? "bg-slate-400" : "bg-slate-700"}`}
                    disabled={!safeState.enabled || isController || isManualRaw}
                    onClick={() => jogMmPositive()}
                  >
                    mm/s +
                  </button>
                  <button
                    className={`rounded px-3 py-2 text-sm text-white ${!safeState.enabled || isController || isManualRaw ? "bg-slate-400" : "bg-slate-700"}`}
                    disabled={!safeState.enabled || isController || isManualRaw}
                    onClick={() => jogMmNegativeFast()}
                  >
                    mm/s Fast -
                  </button>
                  <button
                    className={`rounded px-3 py-2 text-sm text-white ${!safeState.enabled || isController || isManualRaw ? "bg-slate-400" : "bg-slate-700"}`}
                    disabled={!safeState.enabled || isController || isManualRaw}
                    onClick={() => jogMmPositiveFast()}
                  >
                    mm/s Fast +
                  </button>
                </div>

                <Label label="Set Position mm">
                  <EditValue
                    value={safeState.wrapper_position_mm}
                    title="Set Position mm"
                    defaultValue={0}
                    step={0.1}
                    renderValue={(value) => roundToDecimals(value, 3)}
                    onChange={setPositionMm}
                  />
                </Label>

                <Label label="Manual mm/s">
                  <EditValue
                    value={safeState.manual_speed_mm_per_second}
                    title="Manual Speed mm/s"
                    defaultValue={0}
                    min={-200}
                    max={200}
                    step={1}
                    renderValue={(value) => roundToDecimals(value, 2)}
                    onChange={setManualSpeedMmPerSecond}
                  />
                </Label>

                <Label label="Raw Velocity Register">
                  <EditValue
                    value={safeState.manual_velocity_register}
                    title="Manual Velocity Register"
                    defaultValue={0}
                    min={-25000}
                    max={25000}
                    step={1}
                    renderValue={(value) => roundToDecimals(value, 0)}
                    onChange={setManualVelocityRegister}
                  />
                </Label>

                <Label label="Speed Scale">
                  <EditValue
                    value={safeState.speed_scale}
                    title="Speed Scale"
                    defaultValue={1}
                    min={0.01}
                    max={20}
                    step={0.01}
                    renderValue={(value) => roundToDecimals(value, 3)}
                    onChange={setSpeedScale}
                  />
                </Label>

                <Label label="Direction">
                  <SelectionGroup<"-1" | "1">
                    value={String(safeState.direction_multiplier) as "-1" | "1"}
                    orientation="horizontal"
                    options={{
                      "-1": { children: "-1" },
                      "1": { children: "1" },
                    }}
                    onChange={(value) => setDirectionMultiplier(Number(value))}
                  />
                </Label>

                <Label label="Frequency Range">
                  <SelectionGroup<"0" | "1" | "2" | "3">
                    value={
                      String(safeState.freq_range_sel) as "0" | "1" | "2" | "3"
                    }
                    orientation="horizontal"
                    options={{
                      "0": { children: "0" },
                      "1": { children: "1" },
                      "2": { children: "2" },
                      "3": { children: "3" },
                    }}
                    onChange={(value) => setFreqRangeSel(Number(value))}
                  />
                </Label>

                <Label label="Acceleration Range">
                  <SelectionGroup<"0" | "1" | "2" | "3">
                    value={
                      String(safeState.acc_range_sel) as "0" | "1" | "2" | "3"
                    }
                    orientation="horizontal"
                    options={{
                      "0": { children: "0" },
                      "1": { children: "1" },
                      "2": { children: "2" },
                      "3": { children: "3" },
                    }}
                    onChange={(value) => setAccRangeSel(Number(value))}
                  />
                </Label>

                <Label label="Acceleration">
                  <EditValue
                    value={safeState.target_acceleration}
                    title="Acceleration"
                    defaultValue={10000}
                    min={1}
                    max={32767}
                    step={1}
                    renderValue={(value) => roundToDecimals(value, 0)}
                    onChange={setAcceleration}
                  />
                </Label>

                <button
                  className={`w-full rounded px-3 py-2 text-sm text-white ${!safeState.enabled ? "bg-slate-400" : "bg-slate-700"}`}
                  disabled={!safeState.enabled}
                  onClick={forceNotHomed}
                >
                  Reset Homed State
                </button>
              </div>
            </details>
          </div>
        </ControlCard>
      </ControlGrid>
    </Page>
  );
}
