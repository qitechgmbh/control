import React from "react";
import { z } from "zod";
import { useXtremZebra1 } from "./useXtremZebra";
import { Preset, PresetData } from "@/lib/preset/preset";
import { xtremZebra1 } from "@/machines/properties";
import { PresetPreviewEntries } from "@/components/preset/PresetPreviewTable";
import { PresetsPage } from "@/components/preset/PresetsPage";

const xtremZebraDataSchema = z
  .object({
    tolerance: z.number(),
  })
  .partial();

type XtremZebra = typeof xtremZebraDataSchema;

const schemas = new Map([[1, xtremZebraDataSchema]]);

const previewEntries: PresetPreviewEntries<XtremZebra> = [
  {
    name: "Tolerance",
    unit: "kg",
    renderValue: (data: PresetData<XtremZebra>) => data?.tolerance?.toFixed(1),
  },
];

export function XtremZebraPresetsPage() {
  const { setTolerance, defaultState, state } = useXtremZebra1();

  const applyPreset = (preset: Preset<XtremZebra>) => {
    const tolerance = preset.data.tolerance ?? 0.3;

    setTolerance(tolerance);
  };

  const toPresetData = (s: typeof state): PresetData<XtremZebra> => ({
    tolerance: s?.xtrem_zebra_state.tolerance,
  });

  return (
    <PresetsPage
      machine_identification={xtremZebra1.machine_identification}
      currentState={toPresetData(state)}
      schemas={schemas}
      schemaVersion={1}
      applyPreset={applyPreset}
      previewEntries={previewEntries}
      defaultState={toPresetData(defaultState)}
    />
  );
}
