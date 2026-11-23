import { create } from "zustand";
import { MachineIdentificationUnique } from "@/machines/types";
import {
  createMarker,
  deleteAllMarkers,
  deleteMarker,
  fetchMarkers,
  type Marker,
  type NewMarker,
} from "@/lib/markers";

export type { Marker, NewMarker } from "@/lib/markers";

/** How often a mounted graph page re-reads markers, to pick up other clients' changes. */
const REFRESH_INTERVAL_MS = 10_000;

/**
 * Client-side cache of the markers stored in the database, per machine. The database
 * is the source of truth: every mutation goes to the backend and the cache follows.
 */
export type MarkerStoreState = {
  markersByMachine: Record<string, Marker[]>;
};

export type MarkerStoreActions = {
  refresh: (machine: MachineIdentificationUnique) => Promise<void>;
  addMarker: (
    machine: MachineIdentificationUnique,
    marker: NewMarker,
  ) => Promise<Marker>;
  removeMarker: (
    machine: MachineIdentificationUnique,
    id: string,
  ) => Promise<void>;
  clearMarkers: (machine: MachineIdentificationUnique) => Promise<void>;
};

export type MarkerStore = MarkerStoreState & MarkerStoreActions;

export function machineKey(machine: MachineIdentificationUnique): string {
  const { vendor, machine: type } = machine.machine_identification;
  return `${vendor}-${type}-${machine.serial}`;
}

function sortByTime(markers: Marker[]): Marker[] {
  return [...markers].sort((a, b) => a.timestamp - b.timestamp);
}

/**
 * Bumped by every local mutation, so a list request that started before the
 * mutation cannot overwrite its result with a stale list.
 */
const generations = new Map<string, number>();
const bump = (key: string) =>
  generations.set(key, (generations.get(key) ?? 0) + 1);

export const useMarkerStore = create<MarkerStore>()((set) => {
  const update = (key: string, fn: (markers: Marker[]) => Marker[]) =>
    set((state) => ({
      markersByMachine: {
        ...state.markersByMachine,
        [key]: fn(state.markersByMachine[key] ?? []),
      },
    }));

  return {
    markersByMachine: {},

    refresh: async (machine) => {
      const key = machineKey(machine);
      const generation = generations.get(key) ?? 0;
      const markers = await fetchMarkers(machine);
      if ((generations.get(key) ?? 0) !== generation) return;
      update(key, () => sortByTime(markers));
    },

    addMarker: async (machine, marker) => {
      const key = machineKey(machine);
      const created = await createMarker(machine, marker);
      bump(key);
      update(key, (markers) => sortByTime([...markers, created]));
      return created;
    },

    removeMarker: async (machine, id) => {
      const key = machineKey(machine);
      await deleteMarker(machine, id);
      bump(key);
      update(key, (markers) => markers.filter((m) => m.id !== id));
    },

    clearMarkers: async (machine) => {
      const key = machineKey(machine);
      await deleteAllMarkers(machine);
      bump(key);
      update(key, () => []);
    },
  };
});

// --- shared refresh loop: one per machine, however many graphs show it ---

const subscribers = new Map<
  string,
  { count: number; timer: ReturnType<typeof setInterval> }
>();

function refreshQuietly(machine: MachineIdentificationUnique) {
  useMarkerStore
    .getState()
    .refresh(machine)
    .catch((error) => console.warn("Failed to load markers", error));
}

/** Keeps the markers of `machine` loaded and fresh until the returned function is called. */
export function subscribeMarkers(
  machine: MachineIdentificationUnique,
): () => void {
  const key = machineKey(machine);
  const existing = subscribers.get(key);
  if (existing) {
    existing.count++;
  } else {
    refreshQuietly(machine);
    subscribers.set(key, {
      count: 1,
      timer: setInterval(() => refreshQuietly(machine), REFRESH_INTERVAL_MS),
    });
  }
  return () => {
    const entry = subscribers.get(key);
    if (!entry) return;
    entry.count--;
    if (entry.count === 0) {
      clearInterval(entry.timer);
      subscribers.delete(key);
    }
  };
}
