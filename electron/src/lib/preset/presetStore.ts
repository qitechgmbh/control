import { create } from "zustand";
import { persist } from "zustand/middleware";
import { Preset, PresetData, presetSchema } from "./preset";
import { MachineIdentification } from "@/machines/types";
import { z } from "zod";
import { toastError } from "@/components/Toast";

const persistedPresetSchema = presetSchema(z.any()).extend({
  lastModified: z.string(),
});

const persistedStateSchema = z.object({
  presets: z.array(persistedPresetSchema),
  latestPresetIds: z.map(z.string(), z.number()),
});

const localStoreItemSchema = z.object({
  state: z.object({
    presets: z.array(z.any()).default([]),
    latestPresetIds: z.array(z.tuple([z.string(), z.number()])).default([]),
  }),
});

type PresetStoreData = {
  presets: Preset<any>[];
  latestPresetIds: Map<string, number>;
};

export type PresetStore = PresetStoreData & {
  insert: <T extends PresetData>(preset: Omit<Preset<T>, "id">) => Preset<T>;
  update: <T extends PresetData>(preset: Preset<T>) => void;
  remove: <T extends PresetData>(preset: Preset<T>) => void;
  getById: (id: number) => Preset<any> | undefined;

  setLatestPresetId: (
    machineIdentification: MachineIdentification,
    id: number,
  ) => void;
  getLatestPresetId: (
    machineIdentification: MachineIdentification,
  ) => number | undefined;
};

const storage = {
  getItem: (name: string) => {
    const str = localStorage.getItem(name);

    if (!str) {
      return null;
    }

    try {
      const json = JSON.parse(str);
      const { state } = localStoreItemSchema.parse(json);

      const latestPresetIds = new Map(state.latestPresetIds);

      return {
        state: {
          presets: state.presets,
          latestPresetIds,
        },
      };
    } catch (e) {
      console.error(e);
      return null;
    }
  },

  setItem: (name: string, newValue: any) => {
    const latestPresetIds = Array.from(
      newValue.state.latestPresetIds.entries(),
    );

    const serialized = JSON.stringify({
      state: {
        ...newValue.state,
        latestPresetIds,
      },
    });

    localStorage.setItem(name, serialized);
  },

  removeItem: (name: string) => localStorage.removeItem(name),
};

export const usePresetStore = create<PresetStore>()(
  persist(
    (set, get) => ({
      presets: [],
      latestPresetIds: new Map(),

      insert: <T extends PresetData>(preset: Omit<Preset<T>, "id">) => {
        const state = get();
        const { presets } = state;
        const ids = presets.map((p: Preset<any>) => p.id);

        const presetWithId = preset as Preset<T>;
        presetWithId.id = Math.max(0, ...ids) + 1;
        presets.push(presetWithId);

        set({ ...state, presets });

        return presetWithId;
      },

      update: <T extends PresetData>(preset: Preset<T>) => {
        const state = get();
        const presets = state.presets.map((p: Preset<any>) =>
          p.id === preset.id ? preset : p,
        );
        set({ ...state, presets });
      },

      remove: <T extends PresetData>(preset: Preset<T>) => {
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
        try {
          const persistedState = persistedStateSchema.parse(persisted);

          const presets = persistedState.presets.map(
            (preset): Preset<any> => ({
              ...preset,
              lastModified: new Date(preset.lastModified),
            }),
          );

          return {
            ...store,
            latestPresetIds: persistedState.latestPresetIds,
            presets,
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
