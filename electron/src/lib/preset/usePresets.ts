import {
  MachineIdentification,
  machineIdentificationEquals,
} from "@/machines/types";
import { usePresetStore } from "./presetStore";
import { Preset } from "./preset";
import { deepEquals } from "@/lib/objects";

export type Presets<T> = {
  get: () => Preset<T>[];
  createFromCurrentState: (name: string) => Preset<T>;
  updateFromCurrentState: (preset: Preset<T>) => Preset<T>;
  remove: (preset: Preset<T>) => void;
  defaultPreset?: Preset<T>;
  getLatestPreset: () => Preset<T>;
  isActive: (preset: Preset<T>) => boolean;
};

export type UsePresetsParams<T> = {
  machine_identification: MachineIdentification;
  schemaVersion: number;
  readCurrentState: () => Partial<T>;
  defaultData: Partial<T>;
};

export function usePresets<T>({
  machine_identification,
  schemaVersion,
  readCurrentState,
  defaultData,
}: UsePresetsParams<T>): Presets<T> {
  const store = usePresetStore();

  const defaultPreset =
    defaultData === undefined
      ? undefined
      : {
          id: -1,
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
      isLatestPreset: false,
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

  const getLatestPreset = () => {
    let latestPreset = getPresetsForMachine().find(
      (preset) => preset.isLatestPreset,
    );

    if (latestPreset === undefined) {
      latestPreset = store.insert({
        name: "Latest Machine Stettings",
        machine_identification,
        lastModified: new Date(),
        schemaVersion,
        isLatestPreset: true,
        data: readCurrentState(),
      });
    }

    return latestPreset;
  };

  const isActive = (preset: Preset<T>) => {
      const state = readCurrentState();
      // TODO: fill in defaults
      return deepEquals(state, preset.data);
  }

  return {
    get: getPresetsForMachine,
    createFromCurrentState,
    remove: store.remove,
    updateFromCurrentState,
    defaultPreset,
    getLatestPreset,
    isActive,
  };
}
