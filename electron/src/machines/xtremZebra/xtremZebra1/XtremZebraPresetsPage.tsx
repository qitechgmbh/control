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
    plate1Target: z.number(),
    plate2Target: z.number(),
    plate3Target: z.number(),
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
  {
    name: "Plate 1 Target",
    unit: "kg",
    renderValue: (data: PresetData<XtremZebra>) =>
      data?.plate1Target?.toFixed(1),
  },
  {
    name: "Plate 2 Target",
    unit: "kg",
    renderValue: (data: PresetData<XtremZebra>) =>
      data?.plate2Target?.toFixed(1),
  },
  {
    name: "Plate 3 Target",
    unit: "kg",
    renderValue: (data: PresetData<XtremZebra>) =>
      data?.plate3Target?.toFixed(1),
  },
];

export function XtremZebraPresetsPage() {
  const {
    setTolerance,
    setPlate1Target,
    setPlate2Target,
    setPlate3Target,
    defaultState,
    state,
  } = useXtremZebra1();

  const applyPreset = (preset: Preset<XtremZebra>) => {
    const tolerance = preset.data.tolerance ?? 0.3;
    const plate1Target = preset.data.plate1Target ?? 10;
    const plate2Target = preset.data.plate2Target ?? 20;
    const plate3Target = preset.data.plate3Target ?? 20;

    setTolerance(tolerance);
    setPlate1Target(plate1Target);
    setPlate2Target(plate2Target);
    setPlate3Target(plate3Target);
  };

  const toPresetData = (s: typeof state): PresetData<XtremZebra> => ({
    tolerance: s?.xtrem_zebra_state.tolerance,
    plate1Target: s?.xtrem_zebra_state.plate1_target,
    plate2Target: s?.xtrem_zebra_state.plate2_target,
    plate3Target: s?.xtrem_zebra_state.plate3_target,
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
