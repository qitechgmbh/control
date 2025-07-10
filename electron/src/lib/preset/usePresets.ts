import {
  MachineIdentification,
  machineIdentificationEquals,
} from "@/machines/types";
import { usePresetStore } from "./presetStore";
import { Preset } from "./preset";

export type Presets<T> = {
  get: () => Preset<T>[];
  createFromCurrentState: (name: string) => Preset<T>;
  updateFromCurrentState: (preset: Preset<T>) => Preset<T>;
  remove: (preset: Preset<T>) => void;
  defaultPreset: Preset<T>;
};

export type UsePresetsParams<T> = {
  machine_identification: MachineIdentification;
  schemaVersion: number;
  readCurrentState: () => Partial<T>;
  defaultData: T;
};

export function usePresets<T>({
  machine_identification,
  schemaVersion,
  readCurrentState,
  defaultData,
}: UsePresetsParams<T>): Presets<T> {
  const store = usePresetStore();

  const defaultPreset = {
    id: 0,
    name: "Machine Defaults",
    machine_identification,
    lastModified: new Date(0),
    schemaVersion,
    data: defaultData,
  };

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

  const getPresetsForMachine = () =>
    store.presets
      .sort((a, b) => b.lastModified.getTime() - a.lastModified.getTime())
      .filter((preset) =>
        machineIdentificationEquals(
          preset.machine_identification,
          machine_identification,
        ),
      );

  return {
    get: getPresetsForMachine,
    createFromCurrentState,
    remove: store.remove,
    updateFromCurrentState,
    defaultPreset,
  };
}
