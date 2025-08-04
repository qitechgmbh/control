import React from "react";
import { z } from "zod";
import { useLaser1 } from "./useLaser1";
import { Preset, PresetData } from "@/lib/preset/preset";
import { laser1 } from "@/machines/properties";
import { PresetPreviewEntries } from "@/components/preset/PresetPreviewTable";
import { PresetsPage } from "@/components/preset/PresetsPage";

const laser1PresetDataSchema = z
  .object({
    targetDiameter: z.number(),
    lowerTolerance: z.number(),
    higherTolerance: z.number(),
  })
  .partial();

type Laser1 = typeof laser1PresetDataSchema;

const schemas = new Map([[1, laser1PresetDataSchema]]);

const previewEntries: PresetPreviewEntries<Laser1> = [
  {
    name: "Target Diameter",
    unit: "mm",
    renderValue: (data: PresetData<Laser1>) => data?.targetDiameter?.toFixed(3),
  },
  {
    name: "Lower Tolerance",
    unit: "mm",
    renderValue: (data: PresetData<Laser1>) => data?.lowerTolerance?.toFixed(3),
  },
  {
    name: "Higher Tolerance",
    unit: "mm",
    renderValue: (data: PresetData<Laser1>) =>
      data?.higherTolerance?.toFixed(3),
  },
];

export function Laser1PresetsPage() {
  const {
    setTargetDiameter,
    setLowerTolerance,
    setHigherTolerance,
    defaultState,
    state,
  } = useLaser1();

  const applyPreset = (preset: Preset<Laser1>) => {
    const targetDiameter = preset.data.targetDiameter ?? 1.75;
    const lowerTolerance = preset.data?.lowerTolerance ?? 0.05;
    const higherTolerance = preset.data?.higherTolerance ?? 0.05;

    setTargetDiameter(targetDiameter);
    setLowerTolerance(lowerTolerance);
    setHigherTolerance(higherTolerance);
  };

  const toPresetData = (s: typeof state): PresetData<Laser1> => ({
    targetDiameter: s?.laser_state.target_diameter,
    lowerTolerance: s?.laser_state.lower_tolerance,
    higherTolerance: s?.laser_state.higher_tolerance,
  });

  return (
    <PresetsPage
      machine_identification={laser1.machine_identification}
      currentState={toPresetData(state)}
      schemas={schemas}
      schemaVersion={1}
      applyPreset={applyPreset}
      previewEntries={previewEntries}
      defaultState={toPresetData(defaultState)}
    />
  );
}
