import React from "react";
import { useMock1 } from "./useMock";
import { mock1 } from "@/machines/properties";

import { PresetsPage } from "@/components/preset/PresetsPage";
import { Preset } from "@/lib/preset/preset";
import { PresetPreviewEntry } from "@/components/preset/PresetPreviewTable";

type Mock1PresetData = {
  frequency1: number;
  frequency2: number;
  frequency3: number;
};

const previewEntries: PresetPreviewEntry<Mock1PresetData>[] = [
  {
    name: "Freuqcy 1",
    unit: "mHz",
    renderValue: (preset: Preset<Mock1PresetData>) =>
      preset.data.frequency1?.toFixed(3),
  },
  {
    name: "Freuqcy 2",
    unit: "mHz",
    renderValue: (preset: Preset<Mock1PresetData>) =>
      preset.data.frequency2?.toFixed(3),
  },
  {
    name: "Freuqcy 3",
    unit: "mHz",
    renderValue: (preset: Preset<Mock1PresetData>) =>
      preset.data.frequency3?.toFixed(3),
  },
];

export function Mock1PresetsPage() {
  const { mockSetFrequency1, mockSetFrequency2, mockSetFrequency3, mockState } =
    useMock1();

  const applyPreset = (preset: Preset<Mock1PresetData>) => {
    const frequency1 = preset.data?.frequency1 ?? 100;
    const frequency2 = preset.data?.frequency2 ?? 200;
    const frequency3 = preset.data?.frequency2 ?? 500;

    mockSetFrequency1(frequency1);
    mockSetFrequency2(frequency2);
    mockSetFrequency3(frequency3);
  };

  const readCurrentState = () => mockState?.data ?? {};

  return (
    <PresetsPage
      machine_identification={mock1.machine_identification}
      readCurrentState={readCurrentState}
      schemaVersion={1}
      applyPreset={applyPreset}
      previewEntries={previewEntries}
    />
  );
}
