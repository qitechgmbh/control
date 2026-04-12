import { Page } from "@/components/Page";
import { ControlGrid } from "@/control/ControlGrid";
import { ControlCard } from "@/control/ControlCard";
import { SelectionGroup } from "@/control/SelectionGroup";
import { Label } from "@/control/Label";
import React from "react";
import { useWagoWinderSmokeTestMachine } from "./useWagoWinderSmokeTestMachine";

const WINDER2_SPEED_PRESETS = [
  {
    key: "stop",
    label: "Validate / Stop",
    mmPerSecond: 0,
    velocity: 0,
  },
  {
    key: "escape",
    label: "EscapeEndstop",
    mmPerSecond: 10,
    velocity: 183,
  },
  {
    key: "fineDistance",
    label: "FindEndstopFineDistancing",
    mmPerSecond: 2,
    velocity: 37,
  },
  {
    key: "coarse",
    label: "FindEndstopCoarse",
    mmPerSecond: -100,
    velocity: -1829,
  },
  {
    key: "fineSeek",
    label: "FindEndtopFine",
    mmPerSecond: -2,
    velocity: -37,
  },
] as const;

type VelocityPresetKey = (typeof WINDER2_SPEED_PRESETS)[number]["key"];
type CoordinateJogKey = "Stop" | "+X Slow" | "-X Slow";

const COORDINATE_JOG_PRESETS: Record<CoordinateJogKey, number> = {
  Stop: 0,
  "+X Slow": 37,
  "-X Slow": -37,
};

const defaultState = {
  enabled: false,
  target_velocity: 0,
  actual_velocity: 0,
  target_acceleration: 10000,
  freq_range_sel: 0,
  acc_range_sel: 0,
  mode: null,
  ready: false,
  stop2n_ack: false,
  start_ack: false,
  speed_mode_ack: false,
  standstill: false,
  on_speed: false,
  direction_positive: false,
  error: false,
  reset: false,
  position: 0,
  raw_position: 0,
  di1: false,
  di2: false,
  status_byte1: 0,
  status_byte2: 0,
  status_byte3: 0,
  control_byte1: 0,
  control_byte2: 0,
  control_byte3: 0,
};

export function WagoWinderSmokeTestMachineControlPage() {
  const { state, setStepperEnabled, setStepperVelocity, setStepperPosition } =
    useWagoWinderSmokeTestMachine();

  const safeState = state ?? defaultState;
  const velocityPreset =
    WINDER2_SPEED_PRESETS.find(
      (preset) => preset.velocity === safeState.target_velocity,
    )?.key ?? "stop";

  const statusBoxes = [
    { label: "Ready", active: safeState.ready },
    { label: "Stop2N Ack", active: safeState.stop2n_ack },
    { label: "Start Ack", active: safeState.start_ack },
    { label: "Speed Ack", active: safeState.speed_mode_ack },
    { label: "Standstill", active: safeState.standstill },
    { label: "On Speed", active: safeState.on_speed },
    { label: "Dir Pos", active: safeState.direction_positive },
    { label: "Error", active: safeState.error },
    { label: "Reset", active: safeState.reset },
    { label: "DI1", active: safeState.di1 },
    { label: "DI2", active: safeState.di2 },
  ];

  return (
    <Page>
      <ControlGrid columns={2}>
        <ControlCard title="671 Stepper">
          <div className="space-y-4">
            <Label label="Enable">
              <SelectionGroup<"Enabled" | "Disabled">
                value={safeState.enabled ? "Enabled" : "Disabled"}
                orientation="horizontal"
                options={{
                  Disabled: { children: "Disabled" },
                  Enabled: { children: "Enabled" },
                }}
                onChange={(value) => setStepperEnabled(value === "Enabled")}
              />
            </Label>

            <Label label="Velocity Preset">
              <SelectionGroup<VelocityPresetKey>
                value={velocityPreset}
                orientation="horizontal"
                options={Object.fromEntries(
                  WINDER2_SPEED_PRESETS.map((preset) => [
                    preset.key,
                    {
                      children: `${preset.label} (${preset.velocity}, ${preset.mmPerSecond} mm/s)`,
                    },
                  ]),
                )}
                onChange={(value) =>
                  setStepperVelocity(
                    WINDER2_SPEED_PRESETS.find((preset) => preset.key === value)
                      ?.velocity ?? 0,
                  )
                }
              />
            </Label>

            <Label label="X Coordinate Test">
              <div className="space-y-3">
                <div className="grid grid-cols-2 gap-4 text-sm">
                  <div>Logical X: {safeState.position}</div>
                  <div>Raw X: {safeState.raw_position}</div>
                </div>

                <SelectionGroup<CoordinateJogKey>
                  value={
                    (Object.entries(COORDINATE_JOG_PRESETS).find(
                      ([, velocity]) => velocity === safeState.target_velocity,
                    )?.[0] as CoordinateJogKey | undefined) ?? "Stop"
                  }
                  orientation="horizontal"
                  options={{
                    Stop: { children: "Stop (0)" },
                    "+X Slow": { children: "+X Slow (37)" },
                    "-X Slow": { children: "-X Slow (-37)" },
                  }}
                  onChange={(value) =>
                    setStepperVelocity(COORDINATE_JOG_PRESETS[value])
                  }
                />

                <SelectionGroup<"Keep" | "Zero Here">
                  value="Keep"
                  orientation="horizontal"
                  options={{
                    Keep: { children: "Keep Offset" },
                    "Zero Here": { children: "Zero Here" },
                  }}
                  onChange={(value) => {
                    if (value === "Zero Here") {
                      setStepperPosition(0);
                    }
                  }}
                />
              </div>
            </Label>

            <div className="grid grid-cols-3 gap-2 text-xs">
              {statusBoxes.map((item) => (
                <div
                  key={item.label}
                  className={`rounded border px-2 py-2 text-center ${
                    item.active
                      ? "border-green-500 bg-green-50 text-green-900"
                      : "border-zinc-300 bg-zinc-50 text-zinc-700"
                  }`}
                >
                  <div className="font-medium">{item.label}</div>
                  <div>{item.active ? "1" : "0"}</div>
                </div>
              ))}
            </div>

            <div className="grid grid-cols-2 gap-4 text-sm">
              <div>Target Vel: {safeState.target_velocity}</div>
              <div>Actual Vel: {safeState.actual_velocity}</div>
              <div>Mode: {safeState.mode ?? "None"}</div>
              <div>Logical Position: {safeState.position}</div>
              <div>Raw Position: {safeState.raw_position}</div>
              <div>
                C1: 0x{safeState.control_byte1.toString(16).padStart(2, "0")}
              </div>
              <div>
                C2: 0x{safeState.control_byte2.toString(16).padStart(2, "0")}
              </div>
              <div>
                C3: 0x{safeState.control_byte3.toString(16).padStart(2, "0")}
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
          </div>
        </ControlCard>
      </ControlGrid>
    </Page>
  );
}
