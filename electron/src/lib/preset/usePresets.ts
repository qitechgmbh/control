import { MachineIdentification } from "@/machines/types";
import { usePresetStore } from "./presetStore";
import { Preset } from "./preset";

export type Presets<T> = {
  get: Preset<T>[];
  createFromCurrentState: (name: string) => Preset<T>;
  updateFromCurrentState: (preset: Preset<T>) => Preset<T>;
  remove: (preset: Preset<T>) => void;
};

export type UsePresetsParams<T> = {
  machine_identification: MachineIdentification;
  schemaVersion: number;
  readCurrentState: () => Partial<T>;
};

export function usePresets<T>({
  machine_identification,
  schemaVersion,
  readCurrentState,
}: UsePresetsParams<T>): Presets<T> {
  const store = usePresetStore();

  const createFromCurrentState = (name: string): Preset<T> => {
    const data = readCurrentState();

    const preset = store.insert({
      name,
      machine_identification,
      lastModified: new Date(),
      schemaVersion,
      data,
    });

    return preset;
  };

  const updateFromCurrentState = (preset: Preset<T>) => {
    const data = readCurrentState();

    const newPreset = {
      ...preset,
      lastModified: new Date(),
      data: { ...preset.data, ...data },
    };

    store.update(newPreset);

    return newPreset;
  };

  return {
    get: store.presets, // TODO: filter and convert
    createFromCurrentState,
    remove: store.remove,
    updateFromCurrentState,
  };
}
