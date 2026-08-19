import { z } from "zod";
import { useEffect, useState } from "react";
import { baseUrl } from "@/client/useClient";

export const materialPresetSchema = z.object({
  abbrev: z.string(),
  name: z.string(),
  bulk_density: z.number(),
  max_moisture_pct: z.number(),
  temp_min: z.number(),
  temp_max: z.number(),
  drying_time_min: z.number(),
  drying_time_max: z.number(),
  specific_air_volume: z.number(),
});

export type MaterialPreset = z.infer<typeof materialPresetSchema>;

export function recommendedTemp(p: MaterialPreset): number {
  return Math.floor((p.temp_min + p.temp_max) / 2);
}

/// Renders a min/max pair as a single value when they're equal, or a range otherwise
/// (e.g. "80°C" vs "80–90°C").
export function formatRange(min: number, max: number, unit: string): string {
  return min === max ? `${min}${unit}` : `${min}–${max}${unit}`;
}

// Fetched once from the backend (the single source of truth for this data - see
// machine_implementations/src/dryer/material_presets.rs) and cached at module scope, since
// the preset list is static for the lifetime of the app and every dryer page needs it.
let cached: MaterialPreset[] | null = null;
let inFlight: Promise<MaterialPreset[]> | null = null;

function fetchMaterialPresets(): Promise<MaterialPreset[]> {
  if (cached) return Promise.resolve(cached);
  if (!inFlight) {
    inFlight = fetch(`${baseUrl}/api/v2/material-presets`)
      .then((res) => res.json())
      .then((data) => {
        const parsed = z.array(materialPresetSchema).parse(data);
        cached = parsed;
        return parsed;
      })
      .finally(() => {
        inFlight = null;
      });
  }
  return inFlight;
}

/// Material presets, fetched from the backend on first use and cached thereafter. Returns
/// an empty array until the fetch resolves.
export function useMaterialPresets(): MaterialPreset[] {
  const [presets, setPresets] = useState<MaterialPreset[]>(cached ?? []);

  useEffect(() => {
    if (cached) {
      setPresets(cached);
      return;
    }
    let cancelled = false;
    fetchMaterialPresets().then((result) => {
      if (!cancelled) setPresets(result);
    });
    return () => {
      cancelled = true;
    };
  }, []);

  return presets;
}
