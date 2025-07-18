import {
  MachineIdentification,
  machineIdentificationEquals,
} from "@/machines/types";
import { PersistedPreset, usePresetStore } from "./presetStore";
import { Preset, PresetData, PresetSchema } from "./preset";
import { deepEquals } from "@/lib/objects";
import { useEffect } from "react";

export type Presets<T extends PresetSchema> = {
  get: () => Preset<T>[];
  createFromCurrentState: (name: string) => Preset<T>;
  updateFromCurrentState: (preset: Preset<T>) => Preset<T>;
  remove: (preset: Preset<T>) => void;
  defaultPreset?: Preset<T>;
  isLatest: (preset: Preset<T>) => boolean;
  isActive: (preset: Preset<T>) => boolean;
};

export type UsePresetsParams<T extends PresetSchema> = {
  machine_identification: MachineIdentification;
  schemaVersion: number;
  currentState?: PresetData<T>;
  defaultState?: PresetData<T>;
};

export function usePresets<T extends PresetSchema>({
  machine_identification,
  schemaVersion,
  currentState,
  defaultState,
}: UsePresetsParams<T>): Presets<T> {
  const store = usePresetStore();

  const defaultPreset: Preset<T> | undefined =
    defaultState === undefined
      ? undefined
      : {
          id: -1,
          name: "Machine Defaults",
          machineIdentificaiton: machine_identification,
          lastModified: new Date(0),
          schemaVersion,
          data: defaultState,
        };

  const createFromCurrentState = (name: string): Preset<T> => {
    const preset = store.insert({
      id: undefined,
      name,
      machineIdentificaiton: machine_identification,
      lastModified: new Date(),
      schemaVersion,
      data: currentState,
    });

    return { ...preset, data: currentState };
  };

  const updateFromCurrentState = (preset: Preset<T>) => {
    const data = currentState;

    const newPreset = {
      ...preset,
      lastModified: new Date(),
      data: { ...preset.data, ...data },
    };

    store.update(newPreset);

    return newPreset;
  };

  const getPresetsForMachine = (): Preset<T>[] =>
    store.presets
      .sort((a, b) => b.lastModified.getTime() - a.lastModified.getTime())
      .filter((preset) =>
        machineIdentificationEquals(
          preset.machineIdentificaiton,
          machine_identification,
        ),
      ) as Preset<T>[]; // TODO: here we will also use zod und also handle migration

  const getLatestPreset = (): Preset<T> => {
    const latestPresetId = store.getLatestPresetId(machine_identification);
    let latestPreset: PersistedPreset;

    if (latestPresetId === undefined) {
      latestPreset = store.insert({
        id: undefined,
        name: "Latest Machine Stettings",
        machineIdentificaiton: machine_identification,
        lastModified: new Date(),
        schemaVersion,
        data: currentState,
      });

      store.setLatestPresetId(machine_identification, latestPreset.id);
    } else {
      // TODO: use zod here
      latestPreset = store.getById(latestPresetId!)!;
      latestPreset.data = currentState;
      latestPreset.lastModified = new Date();
      store.update(latestPreset);
    }

    // TODO: zod will fix the types
    return latestPreset as Preset<T>;
  };

  useEffect(() => {
    getLatestPreset();
  }, [currentState]);

  const isLatest = (preset: Preset<T>) => {
    return preset.id === store.getLatestPresetId(machine_identification);
  };

  const isActive = (preset: Preset<T>) => {
    return deepEquals(currentState, preset.data);
  };

  return {
    get: getPresetsForMachine,
    createFromCurrentState,
    remove: store.remove,
    updateFromCurrentState,
    defaultPreset,
    isLatest,
    isActive,
  };
}
