import React, { useState } from "react";
import { TouchInput } from "@/components/touch/TouchInput";
import { TouchButton } from "@/components/touch/TouchButton";
import { ControlGrid } from "@/control/ControlGrid";
import { Page } from "@/components/Page";
import { usePresets, UsePresetsParams } from "@/lib/preset/usePresets";
import { Preset } from "@/lib/preset/preset";
import { PresetCard } from "./PresetCard";
import { PresetPreviewEntry } from "./PresetPreviewTable";

type PresetsPageProps<T> = UsePresetsParams<T> & {
  applyPreset: (preset: Preset<T>) => void;
  previewEntries: PresetPreviewEntry<T>[];
};

export function PresetsPage<T>({
  applyPreset,
  machine_identification,
  readCurrentState,
  schemaVersion,
  previewEntries,
  defaultData,
}: PresetsPageProps<T>) {
  const presets = usePresets<T>({
    machine_identification,
    readCurrentState,
    schemaVersion,
    defaultData,
  });

  const [newName, setNewName] = useState("");

  const handleOverwritePreset = (preset: Preset<T>) => {
    const msg = `Are you sure you want to overwrite the preset "${preset.name}" with the current settings? This cannot be undone.`;

    if (!confirm(msg)) {
      return;
    }

    presets.updateFromCurrentState(preset);
  };

  const handleDeletePreset = (preset: Preset<T>) => {
    const msg = `Are you sure you want to delete the preset "${preset.name}"? This cannot be undone.`;

    if (!confirm(msg)) {
      return;
    }

    presets.remove(preset);
  };

  const handleNewPreset = () => presets.createFromCurrentState(newName);

  return (
    <Page>
      <TouchInput
        placeholder="Preset Name"
        onChange={(e) => setNewName(e.target.value)}
      />
      <TouchButton onClick={handleNewPreset}>Create new Preset</TouchButton>
      <ControlGrid columns={2}>
        {presets.get().map((preset) => (
          <PresetCard
            key={preset.id}
            preset={preset}
            onOverwrite={handleOverwritePreset}
            onApply={applyPreset}
            onDelete={handleDeletePreset}
            previewEntries={previewEntries}
            isReadOnly={preset.isLatestPreset}
            isActive={presets.isActive(preset)}
          />
        ))}

        {presets.defaultPreset !== undefined && (
          <PresetCard
            preset={presets.defaultPreset}
            onOverwrite={handleOverwritePreset}
            onApply={applyPreset}
            onDelete={handleDeletePreset}
            previewEntries={previewEntries}
            isReadOnly
            hideDate
            isActive={presets.isActive(presets.defaultPreset)}
          />
        )}
      </ControlGrid>
    </Page>
  );
}
