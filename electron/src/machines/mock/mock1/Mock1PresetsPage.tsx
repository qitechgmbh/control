import React from "react";
import { useMock1 } from "./useMock";
import { mock1 } from "@/machines/properties";

import { PresetsPage } from "@/components/preset/PresetsPage";
import { Preset } from "@/lib/preset/preset";

type Mock1PresetData = {
  frequency1: number;
  frequency2: number;
  frequency3: number;
};

function renderPreview(preset: Preset<Mock1PresetData>) {
  return (
    <>
      Frequency 1 = {preset.data?.frequency1 || "N/A"} mHz <br />
      Frequency 2 = {preset.data?.frequency2 || "N/A"} mHz <br />
      Frequency 3 = {preset.data?.frequency3 || "N/A"} mHz
    </>
  );
}

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
      renderPreview={renderPreview}
    />
  );
}
