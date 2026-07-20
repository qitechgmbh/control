import { create } from "zustand";
import { persist } from "zustand/middleware";

const RECENTLY_USED_MAX = 10;

type MaterialStoreState = {
  favorites: string[];
  recentlyUsed: string[];
  selectedAbbrev: string | null;
  throughput: number;
  targetTimeMin: number;

  toggleFavorite: (abbrev: string) => void;
  selectMaterial: (abbrev: string) => void;
  clearMaterial: () => void;
  setThroughput: (value: number) => void;
  setTargetTimeMin: (value: number) => void;
};

function createMaterialStore(persistName: string) {
  return create<MaterialStoreState>()(
    persist(
      (set) => ({
        favorites: [],
        recentlyUsed: [],
        selectedAbbrev: null,
        throughput: 5,
        targetTimeMin: 240,

        toggleFavorite: (abbrev) =>
          set((s) => ({
            favorites: s.favorites.includes(abbrev)
              ? s.favorites.filter((a) => a !== abbrev)
              : [...s.favorites, abbrev],
          })),

        selectMaterial: (abbrev) =>
          set((s) => ({
            selectedAbbrev: abbrev,
            recentlyUsed: [
              abbrev,
              ...s.recentlyUsed.filter((a) => a !== abbrev),
            ].slice(0, RECENTLY_USED_MAX),
          })),

        clearMaterial: () => set({ selectedAbbrev: null }),
        setThroughput: (value) => set({ throughput: value }),
        setTargetTimeMin: (value) => set({ targetTimeMin: value }),
      }),
      { name: persistName },
    ),
  );
}

export const useDryerV1MaterialStore = createMaterialStore("dryer-v1-material");
export const useDryerSmartMaterialStore = createMaterialStore(
  "dryer-smart-material",
);
