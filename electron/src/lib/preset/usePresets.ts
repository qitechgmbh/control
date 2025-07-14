import {
  MachineIdentification,
  machineIdentificationEquals,
} from "@/machines/types";
import { usePresetStore } from "./presetStore";
import { Preset } from "./preset";
import { deepEquals } from "@/lib/objects";
import { useEffect } from "react";

export type Presets<T> = {
  get: () => Preset<T>[];
  createFromCurrentState: (name: string) => Preset<T>;
  updateFromCurrentState: (preset: Preset<T>) => Preset<T>;
  remove: (preset: Preset<T>) => void;
  defaultPreset?: Preset<T>;
  isLatest: (preset: Preset<T>) => boolean;
  isActive: (preset: Preset<T>) => boolean;
};

export type UsePresetsParams<T> = {
  machine_identification: MachineIdentification;
  schemaVersion: number;
  currentState: Partial<T>;
  defaultData: Partial<T>;
};

export function usePresets<T>({
  machine_identification,
  schemaVersion,
  currentState,
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
    const preset = store.insert({
      name,
      machine_identification,
      lastModified: new Date(),
      schemaVersion,
      data: currentState,
    });

    return preset;
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

  const getPresetsForMachine = () =>
    store.presets
      .sort((a, b) => b.lastModified.getTime() - a.lastModified.getTime())
      .filter((preset) =>
        machineIdentificationEquals(
          preset.machine_identification,
          machine_identification,
        ),
      );

  const getLatestPreset = (): Preset<T> => {
    const latestPresetId = store.getLatestPresetId(machine_identification);
    let latestPreset: Preset<T>;

    if (latestPresetId === undefined) {
      latestPreset = store.insert({
        name: "Latest Machine Stettings",
        machine_identification,
        lastModified: new Date(),
        schemaVersion,
        data: currentState,
      });

      store.setLatestPresetId(machine_identification, latestPreset.id);
    } else {
      latestPreset = store.getById(latestPresetId!)!;
      latestPreset.data = currentState;
      latestPreset.lastModified = new Date();
      store.update(latestPreset);
    }

    return latestPreset;
  };

  useEffect(() => {
    getLatestPreset();
  }, [currentState]);

  const isLatest = (preset: Preset<T>) => {
    return preset.id === store.latestPresetIds.get(machine_identification);
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
