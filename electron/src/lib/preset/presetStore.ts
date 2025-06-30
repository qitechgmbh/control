import { create } from "zustand";
import { persist } from "zustand/middleware";
import { Preset } from "./preset";

export type PresetStore = {
  presets: Preset<any>[];
  insert: <T>(preset: Omit<Preset<T>, "id">) => Preset<T>;
  update: <T>(preset: Preset<T>) => void;
  remove: <T>(preset: Preset<T>) => void;
};

export const usePresetStore = create<PresetStore>()(
  persist(
    (set, get) => ({
      presets: [],

      insert: <T>(preset: Omit<Preset<T>, "id">) => {
        const { presets } = get();
        const ids = presets.map((p) => p.id);

        const presetWithId = preset as Preset<T>;
        presetWithId.id = Math.max(0, ...ids) + 1;
        presets.push(presetWithId);

        set({ presets });

        return presetWithId;
      },

      update: <T>(preset: Preset<T>) => {
        const presets = get().presets.map((p) =>
          p.id === preset.id ? preset : p,
        );
        set({ presets });
      },

      remove: <T>(preset: Preset<T>) => {
        const presets = get().presets.filter((p) => p.id !== preset.id);
        set({ presets });
      },
    }),
    {
      name: "preset-storage",

      merge: (persistedState: any, store: PresetStore) => {
        if (!persistedState || persistedState.presets === undefined) {
          return store;
        }

        // TODO: Using zod vaidation here might be the correct way...
        const { presets: presistedPresets } = persistedState as {
          presets: Preset<any>[];
        };

        const presets = presistedPresets.map((preset) => ({
          ...preset,
          lastModified: new Date(preset.lastModified),
        }));

        return {
          ...store,
          presets,
        };
      },
    },
  ),
);
