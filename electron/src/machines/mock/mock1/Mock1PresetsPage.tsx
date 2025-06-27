import { Icon } from "@/components/Icon";
import React, { useState } from "react";
import { Page } from "@/components/Page";
import { useMock1 } from "./useMock";
import { TouchButton } from "@/components/touch/TouchButton";
import { MachineIdentification } from "@/machines/types";
import { mock1 } from "@/machines/properties";
import { ControlGrid } from "@/control/ControlGrid";

import { create } from "zustand";
import { persist } from "zustand/middleware";
import { TouchInput } from "@/components/touch/TouchInput";

const { machine_identification } = mock1;

type Preset<T> = {
  id: number;
  name: string;
  lastModified: Date;
  machine_identification: MachineIdentification;
  schemaVersion: number;
  data: Partial<T>;
};

type Mock1PresetData = {
  frequency1: number;
  frequency2: number;
  frequency3: number;
};

type PresetCardProps<T> = {
  preset: Preset<T>;
  onApply: (preset: Preset<T>) => void;
  onOverwrite: (preset: Preset<T>) => void;
  onDelete: (preset: Preset<T>) => void;
};

function PresetCard<T>({
  preset,
  onApply,
  onOverwrite,
  onDelete,
}: PresetCardProps<T>) {
  return (
    <div className="flex flex-row items-center gap-4 rounded-3xl border border-gray-200 bg-white p-4 shadow">
      <div className="min-w-0 flex-1">
        <div>
          <div className="truncate text-lg font-semibold text-gray-900">
            {preset.name}
          </div>
          <div className="text-sm text-gray-500">
            Last Modified: {preset.lastModified.toDateString() || "N/A"}
          </div>
        </div>
      </div>
      <div className="flex gap-2">
        <TouchButton
          className="flex-shrink-0"
          variant="outline"
          onClick={() => onOverwrite(preset)}
        >
          <Icon name="lu:HardDriveUpload" />
          Overwrite
        </TouchButton>
        <TouchButton
          className="flex-shrink-0"
          variant="outline"
          onClick={() => onApply(preset)}
        >
          <Icon name="lu:HardDriveDownload" />
          Apply
        </TouchButton>
        <TouchButton
          className="flex-shrink-0"
          variant="destructive"
          onClick={() => onDelete(preset)}
        >
          Delete
        </TouchButton>
      </div>
    </div>
  );
}

type PresetState = {
  presets: Preset<any>[];
  removeById: (id: number) => void;
  updateDataById: <T>(id: number, data: Partial<T>) => void;
  create: <T>(
    name: string,
    data: Partial<T>,
    schemaVersion: number,
  ) => Preset<T>;
  loadMockPresets: () => void;
};

const usePresetStore = create<PresetState>()(
  persist(
    (set, get) => ({
      presets: [],

      loadMockPresets: () =>
        set(() => ({
          presets: [
            {
              id: 1,
              name: "Slow",
              lastModified: new Date(1),
              machine_identification,
              schemaVersion: 1,
              data: {
                frequency1: 1,
                frequency2: 2,
                frequency3: 3,
              },
            },
            {
              id: 2,
              name: "Fast",
              lastModified: new Date(99999999999),
              machine_identification,
              schemaVersion: 1,
              data: {
                frequency1: 111,
                frequency2: 222,
                frequency3: 333,
              },
            },
          ],
        })),

      removeById: (id) =>
        set((state) => ({
          presets: state.presets.filter((p) => p.id !== id),
        })),

      create: <T,>(name: string, data: Partial<T>, schemaVersion: number) => {
        const { presets } = get();
        const ids = presets.map((p) => p.id);
        const newId = Math.max(0, ...ids) + 1;

        const newPreset: Preset<T> = {
          id: newId,
          name,
          machine_identification,
          lastModified: new Date(),
          schemaVersion,
          data,
        };

        presets.push(newPreset);

        set({
          presets: presets,
        });

        return newPreset;
      },

      updateDataById: <T,>(id: number, data: Partial<T>) =>
        set((state) => ({
          presets: state.presets.map((preset) =>
            preset.id === id
              ? { ...preset, lastModified: new Date(), data }
              : preset,
          ),
        })),
    }),
    {
      name: "preset-storage",
      merge: (persistedState, state: PresetState) => {
        const oldState = persistedState as PresetState;
        const presets = oldState.presets.map((preset) => ({
          ...preset,
          lastModified: new Date(preset.lastModified),
        }));

        return {
          ...state,
          ...oldState,
          presets,
        };
      },
    },
  ),
);

export function Mock1PresetsPage() {
  const { mockSetFrequency1, mockSetFrequency2, mockSetFrequency3, mockState } =
    useMock1();

  const presetStore = usePresetStore();

  const handleApplyPreset = (preset: Preset<Mock1PresetData>) => {
    // TODO: this needs to be a proper modal and display the values about to be loaded
    const msg = `Are you sure you want to apply the preset "${preset.name}"? If done carelessly, this could damage machines.`;

    if (!confirm(msg)) {
      return;
    }

    if (preset.data.frequency1 !== undefined) {
      mockSetFrequency1(preset.data.frequency1);
    }

    if (preset.data.frequency2 !== undefined) {
      mockSetFrequency2(preset.data.frequency2);
    }

    if (preset.data.frequency3 !== undefined) {
      mockSetFrequency3(preset.data.frequency3);
    }
  };

  const handleDeletePreset = (preset: Preset<Mock1PresetData>) => {
    const msg = `Are you sure you want to delete the preset "${preset.name}"? This cannot be undone.`;

    if (!confirm(msg)) {
      return;
    }

    presetStore.removeById(preset.id);
  };

  const [newName, setNewName] = useState("");

  const handleNewPreset = () => {
    const data = mockState?.data || {};
    presetStore.create<Mock1PresetData>(newName, data, 1);
  };

  const handleOverwritePreset = (preset) => {
    const msg = `Are you sure you want to overwrite the preset "${preset.name}" with the current settings? This cannot be undone.`;

    if (!confirm(msg)) {
      return;
    }

    const data = mockState?.data || {};
    presetStore.updateDataById(preset.id, data);
  };

  return (
    <Page>
      <TouchButton onClick={() => presetStore.loadMockPresets()}>
        Load Mock Presets
      </TouchButton>
      <TouchInput
        placeholder="Preset Name"
        onChange={(e) => setNewName(e.target.value)}
      />
      <TouchButton onClick={handleNewPreset}>Create new Preset</TouchButton>
      <ControlGrid columns={2}>
        {presetStore.presets.map((preset) => (
          <PresetCard
            key={preset.id}
            preset={preset}
            onOverwrite={handleOverwritePreset}
            onApply={handleApplyPreset}
            onDelete={handleDeletePreset}
          />
        ))}
      </ControlGrid>
    </Page>
  );
}
