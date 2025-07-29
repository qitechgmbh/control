import React from "react";
import { useMock1 } from "./useMock";
import { mock1 } from "@/machines/properties";

import { PresetsPage } from "@/components/preset/PresetsPage";
import { Preset, PresetData } from "@/lib/preset/preset";
import { PresetPreviewEntries } from "@/components/preset/PresetPreviewTable";
import { z } from "zod";

const mock1PresetDataSchema = z
  .object({
    frequency1: z.number(),
    frequency2: z.number(),
    frequency3: z.number(),
  })
  .partial();

type Mock1 = typeof mock1PresetDataSchema;

const schemas = new Map([[1, mock1PresetDataSchema]]);

const previewEntries: PresetPreviewEntries<Mock1> = [
  {
    name: "Frequency 1",
    unit: "mHz",
    renderValue: (data: PresetData<Mock1>) => data?.frequency1?.toFixed(3),
  },
  {
    name: "Frequency 2",
    unit: "mHz",
    renderValue: (data: PresetData<Mock1>) => data?.frequency2?.toFixed(3),
  },
  {
    name: "Frequency 3",
    unit: "mHz",
    renderValue: (data: PresetData<Mock1>) => data?.frequency3?.toFixed(3),
  },
];

export function Mock1PresetsPage() {
  const { setFrequency1, setFrequency2, setFrequency3, defaultState, state } =
    useMock1();

  const applyPreset = (preset: Preset<Mock1>) => {
    const frequency1 = preset.data?.frequency1 ?? 100;
    const frequency2 = preset.data?.frequency2 ?? 200;
    const frequency3 = preset.data?.frequency3 ?? 500;

    setFrequency1(frequency1);
    setFrequency2(frequency2);
    setFrequency3(frequency3);
  };

  const toPresetData = (s: typeof state): PresetData<Mock1> => ({
    frequency1: s?.frequency1 ?? defaultState?.frequency1,
    frequency2: s?.frequency2 ?? defaultState?.frequency2,
    frequency3: s?.frequency3 ?? defaultState?.frequency3,
  });

  return (
    <PresetsPage
      machine_identification={mock1.machine_identification}
      currentState={toPresetData(state)}
      schemas={schemas}
      schemaVersion={1}
      applyPreset={applyPreset}
      previewEntries={previewEntries}
      defaultState={toPresetData(defaultState)}
    />
  );
}
