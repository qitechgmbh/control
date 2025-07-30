import {
  MachineIdentification,
  machineIdentificationEquals,
} from "@/machines/types";
import { usePresetStore } from "./presetStore";
import { Preset, PresetData, presetSchema, PresetSchema } from "./preset";
import { deepEquals } from "@/lib/objects";
import { useEffect } from "react";
import { toastError } from "@/components/Toast";
import { z } from "zod";

export type Presets<T extends PresetSchema> = {
  get: () => Preset<T>[];
  createFromCurrentState: (name: string) => Preset<T>;
  updateFromCurrentState: (preset: Preset<T>) => Preset<T>;
  remove: (preset: Preset<T>) => void;
  defaultPreset?: Preset<T>;
  isLatest: (preset: Preset<T>) => boolean;
  isActive: (preset: Preset<T>) => boolean;
  import: (json: any) => void;
};

export type UsePresetsParams<T extends PresetSchema> = {
  machine_identification: MachineIdentification;
  schemas: Map<number, T>;
  schemaVersion: number;
  currentState?: PresetData<T>;
  defaultState?: PresetData<T>;
};

export function usePresets<T extends PresetSchema>({
  machine_identification,
  schemas,
  schemaVersion,
  currentState,
  defaultState,
}: UsePresetsParams<T>): Presets<T> {
  const store = usePresetStore();

  const parsePreset = (preset: Preset<any>): Preset<T> | undefined => {
    const schema = schemas.get(preset.schemaVersion);

    if (schema === undefined) {
      toastError(
        `Unsupported Preset Version`,
        `Cannot load preset "${preset.name}" because version ${preset.schemaVersion} is not supported.`,
      );
      return undefined;
    }

    try {
      const parsed = presetSchema(schema).parse(preset);

      if (parsed.schemaVersion < schemaVersion) {
        parsed.schemaVersion = schemaVersion;
        store.update(parsed);
      }

      return parsed;
    } catch (e) {
      console.error(e);
      toastError(
        `Invalid Preset`,
        `Cannot load preset "${preset.name}" because validation has failed.`,
      );
    }

    return undefined;
  };

  const defaultPreset: Preset<T> | undefined =
    defaultState === undefined
      ? undefined
      : {
          id: -1,
          name: "Machine Defaults",
          machineIdentification: machine_identification,
          lastModified: new Date(0),
          schemaVersion,
          data: defaultState,
        };

  const createFromCurrentState = (name: string): Preset<T> => {
    const preset = store.insert({
      id: undefined,
      name,
      machineIdentification: machine_identification,
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
          preset.machineIdentification,
          machine_identification,
        ),
      )
      .map(parsePreset)
      .filter((p) => p !== undefined);

  const getLatestPreset = (): Preset<T> | undefined => {
    const latestPresetId = store.getLatestPresetId(machine_identification);

    if (latestPresetId === undefined) {
      const preset = store.insert({
        id: undefined,
        name: "Latest Machine Stettings",
        machineIdentification: machine_identification,
        lastModified: new Date(),
        schemaVersion,
        data: currentState,
      });

      store.setLatestPresetId(machine_identification, preset.id);
      return parsePreset(preset);
    }

    const latestPersist = store.getById(latestPresetId!)!;
    latestPersist.data = currentState;
    latestPersist.lastModified = new Date();
    store.update(latestPersist);

    return parsePreset(latestPersist);
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

  const importPreset = (json: any) => {
    try {
      const anyPreset = presetSchema(z.any()).parse(json);

      if (
        !machineIdentificationEquals(
          anyPreset.machineIdentification,
          machine_identification,
        )
      ) {
        toastError(
          `Wrong Machine`,
          `The preset you entered is for a different machine`,
        );
        return;
      }

      const preset = parsePreset(anyPreset);

      if (preset !== undefined) {
        store.insert(preset);
      }
    } catch (e) {
      console.error(e);
      toastError(
        `Not a Preset File`,
        `Cannot import preset from file because validation has failed.`,
      );
    }
  };

  return {
    get: getPresetsForMachine,
    createFromCurrentState,
    remove: store.remove,
    updateFromCurrentState,
    defaultPreset,
    isLatest,
    isActive,
    import: importPreset,
  };
}
