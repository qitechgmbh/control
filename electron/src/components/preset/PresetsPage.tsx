import React from "react";
import { ControlGrid } from "@/control/ControlGrid";
import { Page } from "@/components/Page";
import { usePresets, UsePresetsParams } from "@/lib/preset/usePresets";
import { Preset, PresetSchema } from "@/lib/preset/preset";
import { PresetCard } from "./PresetCard";
import { PresetPreviewEntries } from "./PresetPreviewTable";
import { NewPresetDialog } from "./NewPresetDialog";
import { downloadJson } from "@/lib/download";
import { JsonFileInput } from "../FileInput";

type PresetsPageProps<T extends PresetSchema> = UsePresetsParams<T> & {
  applyPreset: (preset: Preset<T>) => void;
  previewEntries: PresetPreviewEntries<T>;
};

export function PresetsPage<T extends PresetSchema>({
  applyPreset,
  machine_identification,
  currentState,
  schemas,
  schemaVersion,
  previewEntries,
  defaultState,
}: PresetsPageProps<T>) {
  const presets = usePresets<T>({
    machine_identification,
    currentState,
    schemas,
    schemaVersion,
    defaultState,
  });

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

  const handleExport = (preset: Preset<T>) => {
    const data = { ...preset, id: undefined };
    const filename = `${preset.name}.preset.json`;
    downloadJson(data, filename);
  };

  return (
    <Page>
      <ControlGrid columns={2}>
        <NewPresetDialog
          previewEntries={previewEntries}
          onSave={presets.createFromCurrentState}
          currentState={currentState}
        />
        <JsonFileInput onJson={presets.import} icon="lu:Upload">
          Import Preset from File
        </JsonFileInput>
        {presets.get().map((preset) => {
          const isLatest = presets.isLatest(preset);
          return (
            <PresetCard
              key={preset.id}
              preset={preset}
              onOverwrite={handleOverwritePreset}
              onApply={applyPreset}
              onDelete={handleDeletePreset}
              onExport={isLatest ? undefined : handleExport}
              previewEntries={previewEntries}
              isReadOnly={isLatest}
              isActive={presets.isActive(preset)}
            />
          );
        })}
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
