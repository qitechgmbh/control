import React from "react";
import { PresetsPage } from "@/components/preset/PresetsPage";
import { extruder3 } from "@/machines/properties";
import { useExtruder3 } from "./useExtruder";
import { z } from "zod";
import {
  PresetPreviewEntries,
  previewSeparator,
} from "@/components/preset/PresetPreviewTable";
import { Preset, PresetData } from "@/lib/preset/preset";

const extruder3PresetDataSchema = z
  .object({
    targetFrontHeatingTemperature: z.number(),
    targetMiddleHeatingTemperature: z.number(),
    targetBackHeatingTemperature: z.number(),
    targetNozzleHeatingTemperature: z.number(),

    targetInverterRpm: z.number(),
    targetInverterPressure: z.number(),
    inverterRegulation: z.enum(["RPM", "Pressure"]),

    inverterRotationDirection: z.enum(["Forward", "Backward"]),

    pressureLimit: z.number(),
    pressureLimitEnabled: z.boolean(),

    pidPressureKp: z.number(),
    pidPressureKi: z.number(),
    pidPressureKd: z.number(),
  })
  .partial();

type Extruder3 = typeof extruder3PresetDataSchema;

const schemas = new Map([[1, extruder3PresetDataSchema]]);

const previewEntries: PresetPreviewEntries<Extruder3> = [
  {
    name: "Target Front Temperature",
    unit: "C",
    renderValue: (data: PresetData<Extruder3>) =>
      data.targetFrontHeatingTemperature?.toFixed(1),
  },
  {
    name: "Target Middle Temperature",
    unit: "C",
    renderValue: (data: PresetData<Extruder3>) =>
      data.targetMiddleHeatingTemperature?.toFixed(1),
  },
  {
    name: "Target Back Temperature",
    unit: "C",
    renderValue: (data: PresetData<Extruder3>) =>
      data.targetBackHeatingTemperature?.toFixed(1),
  },
  {
    name: "Target Nozzle Temperature",
    unit: "C",
    renderValue: (data: PresetData<Extruder3>) =>
      data.targetNozzleHeatingTemperature?.toFixed(1),
  },
  previewSeparator,
  {
    name: "Inverter Regulation",
    renderValue: (data: PresetData<Extruder3>) => data.inverterRegulation,
  },
  {
    name: "Target Inverter RPM",
    unit: "rpm",
    renderValue: (data: PresetData<Extruder3>) =>
      data.targetInverterRpm?.toFixed(2),
  },
  {
    name: "Target Inverter Pressure",
    unit: "bar",
    renderValue: (data: PresetData<Extruder3>) =>
      data.targetInverterPressure?.toFixed(1),
  },
  {
    name: "Inverter Direction",
    renderValue: (data: PresetData<Extruder3>) =>
      data.inverterRotationDirection,
  },
  previewSeparator,
  {
    name: "Pressure Limit",
    unit: "bar",
    renderValue: (data: PresetData<Extruder3>) =>
      data.pressureLimit?.toFixed(1),
  },
  {
    name: "Enable Pressure Limit",
    renderValue: (data: PresetData<Extruder3>) =>
      data.pressureLimitEnabled ? "on" : "off",
  },
  previewSeparator,
  {
    name: "PID Pressue Kp",
    renderValue: (data: PresetData<Extruder3>) =>
      data.pidPressureKp?.toFixed(4),
  },
  {
    name: "PID Pressue Ki",
    renderValue: (data: PresetData<Extruder3>) =>
      data.pidPressureKi?.toFixed(4),
  },
  {
    name: "PID Pressue Kd",
    renderValue: (data: PresetData<Extruder3>) =>
      data.pidPressureKd?.toFixed(4),
  },
];

export function Extruder3PresetsPage() {
  const {
    state,
    defaultState,

    setFrontHeatingTemperature,
    setMiddleHeatingTemperature,
    setBackHeatingTemperature,
    setNozzleHeatingTemperature,

    setInverterTargetRpm,
    setInverterTargetPressure,
    setInverterRegulation,
    setInverterRotationDirection,

    setExtruderPressureLimit,
    setExtruderPressureLimitEnabled,

    setPressurePidKd,
    setPressurePidKi,
    setPressurePidKp,
  } = useExtruder3();

  const toPresetData = (s?: typeof state): PresetData<Extruder3> => ({
    targetFrontHeatingTemperature: s?.heating_states.front.target_temperature,
    targetMiddleHeatingTemperature: s?.heating_states.middle.target_temperature,
    targetBackHeatingTemperature: s?.heating_states.back.target_temperature,
    targetNozzleHeatingTemperature: s?.heating_states.nozzle.target_temperature,

    targetInverterRpm: s?.screw_state.target_rpm,
    targetInverterPressure: s?.pressure_state.target_bar,
    inverterRegulation: s?.regulation_state.uses_rpm ? "RPM" : "Pressure",

    inverterRotationDirection: s?.rotation_state?.forward
      ? "Forward"
      : "Backward",

    pressureLimit: s?.extruder_settings_state.pressure_limit,
    pressureLimitEnabled: s?.extruder_settings_state.pressure_limit_enabled,

    pidPressureKp: s?.pid_settings.pressure.kp,
    pidPressureKi: s?.pid_settings.pressure.ki,
    pidPressureKd: s?.pid_settings.pressure.kd,
  });

  const defaults = toPresetData(defaultState);

  const applyPreset = (preset: Preset<Extruder3>) => {
    setFrontHeatingTemperature(
      preset.data.targetFrontHeatingTemperature ?? 150.0,
    );
    setMiddleHeatingTemperature(
      preset.data.targetMiddleHeatingTemperature ?? 150.0,
    );
    setBackHeatingTemperature(
      preset.data.targetBackHeatingTemperature ?? 150.0,
    );
    setNozzleHeatingTemperature(
      preset.data.targetNozzleHeatingTemperature ?? 150.0,
    );

    setInverterTargetRpm(preset.data.targetInverterRpm ?? 0);
    setInverterTargetPressure(preset.data.targetInverterPressure ?? 0);
    setInverterRegulation(preset.data.inverterRegulation === "RPM");

    setInverterRotationDirection(
      preset.data.inverterRotationDirection === "Forward",
    );

    setExtruderPressureLimit(preset.data.pressureLimit ?? 100.0);
    setExtruderPressureLimitEnabled(preset.data.pressureLimitEnabled ?? true);

    setPressurePidKp(preset.data.pidPressureKp ?? 0.16);
    setPressurePidKi(preset.data.pidPressureKi ?? 0.0);
    setPressurePidKd(preset.data.pidPressureKd ?? 0.008);
  };

  return (
    <PresetsPage
      machine_identification={extruder3.machine_identification}
      currentState={toPresetData(state)}
      schemas={schemas}
      schemaVersion={1}
      applyPreset={applyPreset}
      previewEntries={previewEntries}
      defaultState={defaults}
    />
  );
}
