import { create } from "zustand";
import { createJSONStorage, persist } from "zustand/middleware";

export type Marker = {
  timestamp: number;
  name: string;
  value?: number;
  color?: string;
};

const MAX_MARKERS_PER_MACHINE = 200;
const STORAGE_NAME = "machine-markers-storage";
const OLD_STORAGE_PREFIX = "machine-markers-";

export type MarkerStoreState = {
  markersByMachine: Record<string, Marker[]>;
};

export type MarkerStoreActions = {
  getMarkers: (machineId: string) => Marker[];
  addMarker: (machineId: string, marker: Marker) => void;
  removeMarker: (machineId: string, timestamp: number) => void;
  clearMarkers: (machineId: string) => void;
};

export type MarkerStore = MarkerStoreState & MarkerStoreActions;

function trimToMax(markers: Marker[]): Marker[] {
  return markers.length > MAX_MARKERS_PER_MACHINE
    ? markers.slice(-MAX_MARKERS_PER_MACHINE)
    : markers;
}

/** Migrate from legacy per-machine keys (machine-markers-{id}) to unified store */
function migrateFromLegacyStorage(): Partial<MarkerStoreState> | null {
  try {
    const markersByMachine: Record<string, Marker[]> = {};
    for (let i = 0; i < localStorage.length; i++) {
      const key = localStorage.key(i);
      if (
        key?.startsWith(OLD_STORAGE_PREFIX) &&
        key !== STORAGE_NAME
      ) {
        const machineId = key.slice(OLD_STORAGE_PREFIX.length);
        try {
          const stored = localStorage.getItem(key);
          if (stored) {
            const parsed = JSON.parse(stored) as Marker[];
            if (Array.isArray(parsed) && parsed.length > 0) {
              markersByMachine[machineId] = parsed;
            }
          }
        } catch {
          // Skip malformed entries
        }
      }
    }
    if (Object.keys(markersByMachine).length > 0) {
      return { markersByMachine };
    }
  } catch {
    // Ignore migration errors
  }
  return null;
}

const markerStorage = createJSONStorage<MarkerStoreState>(() => ({
  getItem: (name: string): string | null => {
    const next = localStorage.getItem(name);
    if (next) return next;
    const migrated = migrateFromLegacyStorage();
    if (migrated) {
      return JSON.stringify({ state: migrated, version: 0 });
    }
    return null;
  },
  setItem: (name: string, value: string): void => {
    localStorage.setItem(name, value);
  },
  removeItem: (name: string): void => {
    localStorage.removeItem(name);
  },
}));

export const useMarkerStore = create<MarkerStore>()(
  persist(
    (set, get) => ({
      markersByMachine: {},

      getMarkers: (machineId: string) => {
        return get().markersByMachine[machineId] ?? [];
      },

      addMarker: (machineId: string, marker: Marker) => {
        set((state) => {
          const current = state.markersByMachine[machineId] ?? [];
          const updated = trimToMax([...current, marker]);
          return {
            markersByMachine: {
              ...state.markersByMachine,
              [machineId]: updated,
            },
          };
        });
      },

      removeMarker: (machineId: string, timestamp: number) => {
        set((state) => {
          const current = state.markersByMachine[machineId] ?? [];
          const updated = current.filter((m) => m.timestamp !== timestamp);
          return {
            markersByMachine: {
              ...state.markersByMachine,
              [machineId]: updated,
            },
          };
        });
      },

      clearMarkers: (machineId: string) => {
        set((state) => ({
          markersByMachine: {
            ...state.markersByMachine,
            [machineId]: [],
          },
        }));
      },
    }),
    {
      name: STORAGE_NAME,
      storage: markerStorage,
    },
  ),
);
