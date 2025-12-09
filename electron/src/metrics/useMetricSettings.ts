import { useEffect, useState } from "react";

export type MetricsSettings = {
  showJitter: boolean;
  showCpu: boolean;
  showMemory: boolean;
  showIo: boolean;
  showPreemption: boolean;
};

const STORAGE_KEY = "runtime-metrics-settings";

const DEFAULT_SETTINGS: MetricsSettings = {
  showJitter: true,
  showCpu: true,
  showMemory: true,
  showIo: true,
  showPreemption: true,
};

/**
 * Hook to manage which runtime metrics are enabled.
 * Settings are persisted in localStorage so control & graphs stay in sync.
 */
export function useMetricsSettings() {
  const [settings, setSettings] = useState<MetricsSettings>(() => {
    if (typeof window === "undefined") return DEFAULT_SETTINGS;
    try {
      const raw = window.localStorage.getItem(STORAGE_KEY);
      if (!raw) return DEFAULT_SETTINGS;
      const parsed = JSON.parse(raw) as Partial<MetricsSettings>;
      return { ...DEFAULT_SETTINGS, ...parsed };
    } catch {
      return DEFAULT_SETTINGS;
    }
  });

  useEffect(() => {
    if (typeof window === "undefined") return;
    window.localStorage.setItem(STORAGE_KEY, JSON.stringify(settings));
  }, [settings]);

  const updateSetting = (key: keyof MetricsSettings, value: boolean) => {
    setSettings((prev) => ({ ...prev, [key]: value }));
  };

  const anyEnabled = Object.values(settings).some(Boolean);

  return { settings, updateSetting, anyEnabled };
}

// Optional: allow default import too
export default useMetricsSettings;