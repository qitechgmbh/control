import { create } from "zustand";
import { persist, PersistStorage, StorageValue } from "zustand/middleware";
import { Preset, PresetSchema, presetSchema } from "./preset";
import { MachineIdentification } from "@/machines/types";
import { z } from "zod";
import { toastError } from "@/components/Toast";

const persistedPresetSchema = presetSchema(z.any()).extend({
  id: z.number().int().nonnegative(),
});

export type PersistedPreset = z.infer<typeof persistedPresetSchema>;

const persistedStateSchema = z.object({
  presets: z.array(persistedPresetSchema),
  latestPresetIds: z.map(z.string(), z.number()),
});

type PersistedState = z.infer<typeof persistedStateSchema>;

const localStoreItemSchema = z.object({
  state: z.object({
    presets: z.array(z.any()).default([]),
    latestPresetIds: z.array(z.tuple([z.string(), z.number()])).default([]),
  }),
});

type LocalStoreItem = z.infer<typeof localStoreItemSchema>;

type PresetStoreData = {
  presets: PersistedPreset[];
  latestPresetIds: Map<string, number>;
};

export type PresetStore = PresetStoreData & {
  insert: <T extends PresetSchema>(preset: Preset<T>) => PersistedPreset;
  update: <T extends PresetSchema>(preset: Preset<T>) => void;
  remove: <T extends PresetSchema>(preset: Preset<T>) => void;
  getById: (id: number) => PersistedPreset | undefined;

  setLatestPresetId: (
    machineIdentification: MachineIdentification,
    id: number,
  ) => void;
  getLatestPresetId: (
    machineIdentification: MachineIdentification,
  ) => number | undefined;
};

const storage: PersistStorage<PersistedState> = {
  getItem: (name: string): StorageValue<PersistedState> | null => {
    const str = localStorage.getItem(name);

    if (!str) {
      return null;
    }

    try {
      const json = JSON.parse(str);
      const { state: item } = localStoreItemSchema.parse(json);

      const latestPresetIds = new Map(item.latestPresetIds);

      const state: PersistedState = {
        presets: item.presets,
        latestPresetIds,
      };

      return { state };
    } catch (e) {
      console.error(e);
    }

    return null;
  },

  setItem: (name: string, newValue: StorageValue<PersistedState>) => {
    const latestPresetIds = Array.from(
      newValue.state?.latestPresetIds?.entries(),
    );

    const item: LocalStoreItem = {
      state: {
        ...newValue.state,
        latestPresetIds,
      },
    };

    const json = JSON.stringify(item);

    localStorage.setItem(name, json);
  },

  removeItem: (name: string) => localStorage.removeItem(name),
};

export const usePresetStore = create<PresetStore>()(
  persist(
    (set, get) => ({
      presets: [],
      latestPresetIds: new Map(),

      insert: <T extends PresetSchema>(preset: Preset<T>) => {
        const state = get();
        const { presets } = state;
        const ids = presets.map((p: PersistedPreset) => p.id);

        const id = Math.max(0, ...ids) + 1;

        const persistedPreset: PersistedPreset = { ...preset, id };
        presets.push(persistedPreset);

        set({ ...state, presets });

        return persistedPreset;
      },

      update: <T extends PresetSchema>(preset: Preset<T>) => {
        const state = get();
        const presets: PersistedPreset[] = state.presets.map(
          (p: PersistedPreset) =>
            p.id === preset.id ? (preset as PersistedPreset) : p,
        );
        set({ ...state, presets });
      },

      remove: <T extends PresetSchema>(preset: Preset<T>) => {
        const state = get();
        const presets = state.presets.filter(
          (p: Preset<any>) => p.id !== preset.id,
        );
        set({ ...state, presets });
      },

      getById: (id: number) => {
        return get().presets.find((preset: Preset<any>) => preset.id === id);
      },

      setLatestPresetId: (
        machineIdentification: MachineIdentification,
        id: number,
      ) => {
        const state = get();
        const { latestPresetIds } = state;
        const key = `${machineIdentification.vendor}:${machineIdentification.machine}`;
        latestPresetIds.set(key, id);
        set({ ...state, latestPresetIds });
      },

      getLatestPresetId: (machineIdentification: MachineIdentification) => {
        const state = get();
        const { latestPresetIds } = state;
        const key = `${machineIdentification.vendor}:${machineIdentification.machine}`;
        return latestPresetIds.get(key);
      },
    }),
    {
      name: "preset-storage",

      merge: (persisted: any, store: PresetStore): PresetStore => {
        if (!persisted) {
          return store;
        }

        try {
          const persistedState = persistedStateSchema.parse(persisted);

          return {
            ...store,
            ...persistedState,
          };
        } catch (e) {
          console.error(e);
          toastError(
            `Loading presets failed`,
            `Could not load presets from storage.`,
          );
        }

        return store;
      },

      storage,
    },
  ),
);
