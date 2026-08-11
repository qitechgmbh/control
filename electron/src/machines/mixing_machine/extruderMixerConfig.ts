import { useEffect, useState } from "react";

export type CalibrationRole = "hopperA" | "hopperB";

export type CalibrationProfile = {
  name: string;
  testRpm: number;
  durationSeconds: number;
  collectedGrams: number;
  rateKgPerHour: number;
  kgPerHourPerRpm: number;
};

export type ExtruderMixerConfig = {
  motorForward: {
    hopperA: boolean;
    hopperB: boolean;
  };
  calibration: Record<CalibrationRole, CalibrationProfile | null>;
};

export type ExtruderMixerPreset = {
  id: string;
  name: string;
  dosageA: number;
  dosageB: number;
  screwRpm: number;
  frontTemperature: number;
  middleTemperature: number;
  backTemperature: number;
  nozzleTemperature: number;
};

const CONFIG_KEY = "extruder_with_mixer_config_v1";
const PRESETS_KEY = "extruder_with_mixer_presets_v1";
const ACTIVE_PRESET_KEY = "extruder_with_mixer_active_preset_v1";
const ACTIVE_CALIBRATION_KEY = "extruder_with_mixer_active_calibration_v1";
const CHANGE_EVENT = "extruder-with-mixer-storage-change";

const defaultConfig: ExtruderMixerConfig = {
  motorForward: {
    hopperA: true,
    hopperB: true,
  },
  calibration: {
    hopperA: null,
    hopperB: null,
  },
};

const defaultPresets: ExtruderMixerPreset[] = [
  {
    id: "standard-production",
    name: "Standard production",
    dosageA: 1,
    dosageB: 1,
    screwRpm: 20,
    frontTemperature: 180,
    middleTemperature: 185,
    backTemperature: 180,
    nozzleTemperature: 185,
  },
];

function emitChange() {
  window.dispatchEvent(new Event(CHANGE_EVENT));
}

function readJson<T>(key: string, fallback: T): T {
  try {
    const value = localStorage.getItem(key);
    return value ? (JSON.parse(value) as T) : fallback;
  } catch {
    return fallback;
  }
}

export function getExtruderMixerConfig(): ExtruderMixerConfig {
  const stored = readJson<ExtruderMixerConfig>(CONFIG_KEY, defaultConfig);
  return {
    motorForward: {
      hopperA: stored.motorForward?.hopperA ?? true,
      hopperB: stored.motorForward?.hopperB ?? true,
    },
    calibration: {
      hopperA: stored.calibration?.hopperA ?? null,
      hopperB: stored.calibration?.hopperB ?? null,
    },
  };
}

export function saveExtruderMixerConfig(config: ExtruderMixerConfig) {
  localStorage.setItem(CONFIG_KEY, JSON.stringify(config));
  emitChange();
}

export function getExtruderMixerPresets(): ExtruderMixerPreset[] {
  return readJson<ExtruderMixerPreset[]>(PRESETS_KEY, defaultPresets);
}

export function saveExtruderMixerPresets(presets: ExtruderMixerPreset[]) {
  localStorage.setItem(PRESETS_KEY, JSON.stringify(presets));
  emitChange();
}

export function getActiveExtruderMixerPreset(): ExtruderMixerPreset {
  const presets = getExtruderMixerPresets();
  const activeId = localStorage.getItem(ACTIVE_PRESET_KEY);
  return (
    presets.find((preset) => preset.id === activeId) ??
    presets[0] ??
    defaultPresets[0]
  );
}

export function setActiveExtruderMixerPreset(id: string) {
  localStorage.setItem(ACTIVE_PRESET_KEY, id);
  emitChange();
}

export function getActiveMixerCalibration(): CalibrationRole | null {
  const role = localStorage.getItem(ACTIVE_CALIBRATION_KEY);
  return role === "hopperA" || role === "hopperB" ? role : null;
}

export function setActiveMixerCalibration(role: CalibrationRole | null) {
  if (role) localStorage.setItem(ACTIVE_CALIBRATION_KEY, role);
  else localStorage.removeItem(ACTIVE_CALIBRATION_KEY);
  emitChange();
}

export function useExtruderMixerStorage() {
  const [, setRevision] = useState(0);
  useEffect(() => {
    const update = () => setRevision((value) => value + 1);
    window.addEventListener(CHANGE_EVENT, update);
    window.addEventListener("storage", update);
    return () => {
      window.removeEventListener(CHANGE_EVENT, update);
      window.removeEventListener("storage", update);
    };
  }, []);

  return {
    config: getExtruderMixerConfig(),
    presets: getExtruderMixerPresets(),
    activePreset: getActiveExtruderMixerPreset(),
    activeCalibration: getActiveMixerCalibration(),
  };
}

export function calculateCalibration(
  name: string,
  testRpm: number,
  durationSeconds: number,
  collectedGrams: number,
): CalibrationProfile {
  const rateKgPerHour = (collectedGrams * 3.6) / durationSeconds;
  return {
    name,
    testRpm,
    durationSeconds,
    collectedGrams,
    rateKgPerHour,
    kgPerHourPerRpm: rateKgPerHour / testRpm,
  };
}
