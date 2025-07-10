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

const defaultData: Mock1PresetData = {
  frequency1: 100,
  frequency2: 200,
  frequency3: 500,
};

const previewEntries: PresetPreviewEntry<Mock1PresetData>[] = [
  {
    name: "Frequency 1",
    unit: "mHz",
    renderValue: (preset: Preset<Mock1PresetData>) =>
      preset.data.frequency1?.toFixed(3),
  },
  {
    name: "Frequency 2",
    unit: "mHz",
    renderValue: (preset: Preset<Mock1PresetData>) =>
      preset.data.frequency2?.toFixed(3),
  },
  {
    name: "Frequency 3",
    unit: "mHz",
    renderValue: (preset: Preset<Mock1PresetData>) =>
      preset.data.frequency3?.toFixed(3),
  },
];

export function Mock1PresetsPage() {
  const { setFrequency1, setFrequency2, setFrequency3, state } =
    useMock1();

  const applyPreset = (preset: Preset<Mock1PresetData>) => {
    const frequency1 = preset.data?.frequency1 ?? 100;
    const frequency2 = preset.data?.frequency2 ?? 200;
    const frequency3 = preset.data?.frequency3 ?? 500;

    setFrequency1(frequency1);
    setFrequency2(frequency2);
    setFrequency3(frequency3);
  };

  const readCurrentState = () => ({
      frequency1: state?.frequency1,
      frequency2: state?.frequency2,
      frequency3: state?.frequency3,
  });

  return (
    <PresetsPage
      machine_identification={mock1.machine_identification}
      readCurrentState={readCurrentState}
      schemaVersion={1}
      applyPreset={applyPreset}
      previewEntries={previewEntries}
      defaultData={defaultData}
    />
  );
}
