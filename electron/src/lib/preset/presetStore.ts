import { create } from "zustand";
import { persist } from "zustand/middleware";
import { Preset } from "./preset";
import { machineIdentificaiton, MachineIdentification } from "@/machines/types";

export type PresetStore = {
    presets: Preset<any>[];
    latestPresetIds: Map<MachineIdentification, number>;

    insert: <T>(preset: Omit<Preset<T>, "id">) => Preset<T>;
    update: <T>(preset: Preset<T>) => void;
    remove: <T>(preset: Preset<T>) => void;
    getById: <T>(id: number) => Preset<T> | undefined;

    setLatestPresetId: (machineIdentification: MachineIdentification, id: number) => void;
    getLatestPresetId: (machineIdentification: MachineIdentification) => number | undefined;
};

const storage = {
    getItem: (name: string) => {
        const str = localStorage.getItem(name);

        if (!str) {
            return null;
        }

        try {
            const parsed = JSON.parse(str);
            const latestPresetIds = new Map(parsed.state.latestPresetIds);

            return {
                state: {
                    ...parsed.state,
                    latestPresetIds,
                },
            };

        } catch(e) {
            console.error(e);
            return null;
        }
    },

    setItem: (name: string, newValue: any) => {
        const latestPresetIds = Array.from(newValue.state.latestPresetIds.entries());

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

            insert: <T>(preset: Omit<Preset<T>, "id">) => {
                const state = get();
                const { presets } = state;
                const ids = presets.map((p) => p.id);

                const presetWithId = preset as Preset<T>;
                presetWithId.id = Math.max(0, ...ids) + 1;
                presets.push(presetWithId);

                set({ ...state, presets });

                return presetWithId;
            },

            update: <T>(preset: Preset<T>) => {
                const state = get();
                const presets = state.presets.map((p) =>
                    p.id === preset.id ? preset : p,
                );
                set({ ...state, presets });
            },

            remove: <T>(preset: Preset<T>) => {
                const state = get();
                const presets = state.presets.filter((p) => p.id !== preset.id);
                set({ ...state, presets });
            },

            getById: (id: number) => {
                return get().presets.find(preset => preset.id === id);
            },

            setLatestPresetId: (machineIdentification: MachineIdentification, id: number) => {
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

            merge: (persistedState: any, store: PresetStore) => {
                // TODO: Using zod vaidation here is be the correct way...

                const {
                    presets: presistedPresets,
                    latestPresetIds,
                } = persistedState as {
                    presets: Preset<any>[];
                } | any;

                const presets = presistedPresets.map((preset: Preset<any>) => ({
                    ...preset,
                    lastModified: new Date(preset.lastModified),
                }));

                debugger;

                return {
                    ...store,
                    latestPresetIds,
                    presets,
                };
            },

            storage,
        }
    ),
);
