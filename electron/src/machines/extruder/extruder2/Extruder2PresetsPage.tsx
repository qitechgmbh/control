import React from "react";
import { PresetsPage } from "@/components/preset/PresetsPage";
import { extruder2 } from "@/machines/properties";
import { useExtruder2 } from "./useExtruder";
import { z } from "zod";
import {
  PresetPreviewEntries,
  previewSeparator,
} from "@/components/preset/PresetPreviewTable";
import { Preset } from "@/lib/preset/preset";
import { TuneZone } from "../temperatureAutoTuneSchema";

const extruder1PresetDataSchema = z
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

    // Per-zone temperature PID gains, so a tuning session survives a restart.
    // The schema is `.partial()`, so these are optional and presets saved before they existed
    // still parse — no schemaVersion bump needed.
    pidTempFrontKp: z.number(),
    pidTempFrontKi: z.number(),
    pidTempFrontKd: z.number(),
    pidTempMiddleKp: z.number(),
    pidTempMiddleKi: z.number(),
    pidTempMiddleKd: z.number(),
    pidTempBackKp: z.number(),
    pidTempBackKi: z.number(),
    pidTempBackKd: z.number(),
    pidTempNozzleKp: z.number(),
    pidTempNozzleKi: z.number(),
    pidTempNozzleKd: z.number(),
  })
  .partial();

type Extruder2 = z.infer<typeof extruder1PresetDataSchema>;

const schemas = new Map([[1, extruder1PresetDataSchema]]);

/** Compact one-line rendering of a zone's three gains, so the preview stays readable. */
const formatZoneGains = (kp?: number, ki?: number, kd?: number) =>
  kp === undefined
    ? undefined
    : `${kp.toPrecision(3)} / ${(ki ?? 0).toPrecision(3)} / ${(kd ?? 0).toPrecision(3)}`;

const previewEntries: PresetPreviewEntries<Extruder2> = [
  {
    name: "Target Front Temperature",
    unit: "C",
    renderValue: (data: Extruder2) =>
      data.targetFrontHeatingTemperature?.toFixed(1),
  },
  {
    name: "Target Middle Temperature",
    unit: "C",
    renderValue: (data: Extruder2) =>
      data.targetMiddleHeatingTemperature?.toFixed(1),
  },
  {
    name: "Target Back Temperature",
    unit: "C",
    renderValue: (data: Extruder2) =>
      data.targetBackHeatingTemperature?.toFixed(1),
  },
  {
    name: "Target Nozzle Temperature",
    unit: "C",
    renderValue: (data: Extruder2) =>
      data.targetNozzleHeatingTemperature?.toFixed(1),
  },
  previewSeparator,
  {
    name: "Inverter Regulation",
    renderValue: (data: Extruder2) => data.inverterRegulation,
  },
  {
    name: "Target Inverter RPM",
    unit: "rpm",
    renderValue: (data: Extruder2) => data.targetInverterRpm?.toFixed(2),
  },
  {
    name: "Target Inverter Pressure",
    unit: "bar",
    renderValue: (data: Extruder2) => data.targetInverterPressure?.toFixed(1),
  },
  {
    name: "Inverter Direction",
    renderValue: (data: Extruder2) => data.inverterRotationDirection,
  },
  previewSeparator,
  {
    name: "Pressure Limit",
    unit: "bar",
    renderValue: (data: Extruder2) => data.pressureLimit?.toFixed(1),
  },
  {
    name: "Enable Pressure Limit",
    renderValue: (data: Extruder2) =>
      data.pressureLimitEnabled ? "on" : "off",
  },
  previewSeparator,
  {
    name: "PID Pressue Kp",
    renderValue: (data: Extruder2) => data.pidPressureKp?.toFixed(4),
  },
  {
    name: "PID Pressue Ki",
    renderValue: (data: Extruder2) => data.pidPressureKi?.toFixed(4),
  },
  {
    name: "PID Pressue Kd",
    renderValue: (data: Extruder2) => data.pidPressureKd?.toFixed(4),
  },
  previewSeparator,
  {
    name: "PID Temp Front (kp/ki/kd)",
    renderValue: (data: Extruder2) =>
      formatZoneGains(
        data.pidTempFrontKp,
        data.pidTempFrontKi,
        data.pidTempFrontKd,
      ),
  },
  {
    name: "PID Temp Middle (kp/ki/kd)",
    renderValue: (data: Extruder2) =>
      formatZoneGains(
        data.pidTempMiddleKp,
        data.pidTempMiddleKi,
        data.pidTempMiddleKd,
      ),
  },
  {
    name: "PID Temp Back (kp/ki/kd)",
    renderValue: (data: Extruder2) =>
      formatZoneGains(
        data.pidTempBackKp,
        data.pidTempBackKi,
        data.pidTempBackKd,
      ),
  },
  {
    name: "PID Temp Nozzle (kp/ki/kd)",
    renderValue: (data: Extruder2) =>
      formatZoneGains(
        data.pidTempNozzleKp,
        data.pidTempNozzleKi,
        data.pidTempNozzleKd,
      ),
  },
];

export function Extruder2PresetsPage() {
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
    setTemperaturePidZone,
  } = useExtruder2();

  /** Apply a zone's gains, falling back to the machine defaults for older presets. */
  const applyZoneGains = (
    zone: TuneZone,
    gains: { kp?: number; ki?: number; kd?: number },
  ) => {
    const fallback = defaultState?.pid_settings.temperature[zone];
    setTemperaturePidZone(zone, {
      kp: gains.kp ?? fallback?.kp ?? 0.16,
      ki: gains.ki ?? fallback?.ki ?? 0.0,
      kd: gains.kd ?? fallback?.kd ?? 0.008,
    });
  };

  const toPresetData = (s?: typeof state): Extruder2 => ({
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

    pidTempFrontKp: s?.pid_settings.temperature.front.kp,
    pidTempFrontKi: s?.pid_settings.temperature.front.ki,
    pidTempFrontKd: s?.pid_settings.temperature.front.kd,
    pidTempMiddleKp: s?.pid_settings.temperature.middle.kp,
    pidTempMiddleKi: s?.pid_settings.temperature.middle.ki,
    pidTempMiddleKd: s?.pid_settings.temperature.middle.kd,
    pidTempBackKp: s?.pid_settings.temperature.back.kp,
    pidTempBackKi: s?.pid_settings.temperature.back.ki,
    pidTempBackKd: s?.pid_settings.temperature.back.kd,
    pidTempNozzleKp: s?.pid_settings.temperature.nozzle.kp,
    pidTempNozzleKi: s?.pid_settings.temperature.nozzle.ki,
    pidTempNozzleKd: s?.pid_settings.temperature.nozzle.kd,
  });

  const defaults = toPresetData(defaultState);

  const applyPreset = (preset: Preset<Extruder2>) => {
    setFrontHeatingTemperature(
      preset?.data?.targetFrontHeatingTemperature ?? 150.0,
    );
    setMiddleHeatingTemperature(
      preset?.data?.targetMiddleHeatingTemperature ?? 150.0,
    );
    setBackHeatingTemperature(
      preset?.data?.targetBackHeatingTemperature ?? 150.0,
    );
    setNozzleHeatingTemperature(
      preset?.data?.targetNozzleHeatingTemperature ?? 150.0,
    );

    setInverterTargetRpm(preset?.data?.targetInverterRpm ?? 0);
    setInverterTargetPressure(preset?.data?.targetInverterPressure ?? 0);
    setInverterRegulation(preset?.data?.inverterRegulation === "RPM");

    setInverterRotationDirection(
      preset?.data?.inverterRotationDirection === "Forward",
    );

    setExtruderPressureLimit(preset?.data?.pressureLimit ?? 100.0);
    setExtruderPressureLimitEnabled(preset?.data?.pressureLimitEnabled ?? true);

    setPressurePidKp(preset?.data?.pidPressureKp ?? 0.16);
    setPressurePidKi(preset?.data?.pidPressureKi ?? 0.0);
    setPressurePidKd(preset?.data?.pidPressureKd ?? 0.008);

    // One mutation per zone rather than one per gain, so a preset does not fire twelve requests
    // that race on optimistic state. Presets saved before these fields existed fall back to the
    // machine defaults.
    applyZoneGains("front", {
      kp: preset?.data?.pidTempFrontKp,
      ki: preset?.data?.pidTempFrontKi,
      kd: preset?.data?.pidTempFrontKd,
    });
    applyZoneGains("middle", {
      kp: preset?.data?.pidTempMiddleKp,
      ki: preset?.data?.pidTempMiddleKi,
      kd: preset?.data?.pidTempMiddleKd,
    });
    applyZoneGains("back", {
      kp: preset?.data?.pidTempBackKp,
      ki: preset?.data?.pidTempBackKi,
      kd: preset?.data?.pidTempBackKd,
    });
    applyZoneGains("nozzle", {
      kp: preset?.data?.pidTempNozzleKp,
      ki: preset?.data?.pidTempNozzleKi,
      kd: preset?.data?.pidTempNozzleKd,
    });
  };

  return (
    <PresetsPage
      machine_identification={extruder2.machine_identification}
      currentState={toPresetData(state)}
      schemas={schemas}
      schemaVersion={1}
      applyPreset={applyPreset}
      previewEntries={previewEntries}
      defaultState={defaults}
    />
  );
}
