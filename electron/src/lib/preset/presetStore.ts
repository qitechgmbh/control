import { create } from "zustand";
import { Preset, presetSchema } from "./preset";
import { MachineIdentification } from "@/machines/types";
import { z } from "zod";
import { toastError } from "@/components/Toast";

const persistedPresetSchema = presetSchema(z.any()).extend({
  id: z.number().int().nonnegative(),
});

export type PersistedPreset = z.infer<typeof persistedPresetSchema>;

const latestPresetIdEntrySchema = z.tuple([z.string(), z.number()]);

/** On-disk format of presets.json. */
const presetFileSchema = z.object({
  version: z.literal(1),
  presets: z.array(z.unknown()).default([]),
  latestPresetIds: z.array(z.unknown()).default([]),
  quarantined: z.array(z.unknown()).default([]),
});

type PresetFile = z.infer<typeof presetFileSchema>;

const FALLBACK_LOCAL_STORAGE_KEY = "presets";

type PresetStoreData = {
  presets: PersistedPreset[];
  latestPresetIds: Map<string, number>;
  /**
   * Raw presets that failed validation on load. They are kept verbatim and
   * written back unchanged so a single broken entry can never cause the
   * other presets (or itself) to be lost.
   */
  quarantined: unknown[];
  /** True once presets have been loaded from disk. Nothing is saved before. */
  hydrated: boolean;
};

export type PresetStore = PresetStoreData & {
  insert: <T>(preset: Preset<T>) => PersistedPreset;
  update: <T>(preset: Preset<T>) => void;
  remove: <T>(preset: Preset<T>) => void;
  getById: (id: number) => PersistedPreset | undefined;

  setLatestPresetId: (
    machineIdentification: MachineIdentification,
    id: number,
  ) => void;
  getLatestPresetId: (
    machineIdentification: MachineIdentification,
  ) => number | undefined;
};

const machineKey = (m: MachineIdentification) => `${m.vendor}:${m.machine}`;

const quarantinedId = (raw: unknown): number | undefined => {
  if (typeof raw === "object" && raw !== null && "id" in raw) {
    const id = (raw as { id: unknown }).id;
    if (typeof id === "number" && Number.isFinite(id)) {
      return id;
    }
  }
  return undefined;
};

export const usePresetStore = create<PresetStore>()((set, get) => ({
  presets: [],
  latestPresetIds: new Map(),
  quarantined: [],
  hydrated: false,

  insert: <T>(preset: Preset<T>) => {
    const { presets, quarantined } = get();
    // Quarantined ids are reserved so they don't collide once repaired.
    const ids = [
      ...presets.map((p) => p.id),
      ...quarantined
        .map(quarantinedId)
        .filter((id): id is number => id !== undefined),
    ];
    const id = Math.max(0, ...ids) + 1;

    const persistedPreset: PersistedPreset = { ...preset, id };
    set({ presets: [...presets, persistedPreset] });

    return persistedPreset;
  },

  update: <T>(preset: Preset<T>) => {
    set({
      presets: get().presets.map((p) =>
        p.id === preset.id ? (preset as PersistedPreset) : p,
      ),
    });
  },

  remove: <T>(preset: Preset<T>) => {
    set({ presets: get().presets.filter((p) => p.id !== preset.id) });
  },

  getById: (id: number) => {
    return get().presets.find((preset) => preset.id === id);
  },

  setLatestPresetId: (
    machineIdentification: MachineIdentification,
    id: number,
  ) => {
    const latestPresetIds = new Map(get().latestPresetIds);
    latestPresetIds.set(machineKey(machineIdentification), id);
    set({ latestPresetIds });
  },

  getLatestPresetId: (machineIdentification: MachineIdentification) => {
    return get().latestPresetIds.get(machineKey(machineIdentification));
  },
}));

// ---------------------------------------------------------------------------
// Persistence
// ---------------------------------------------------------------------------

type StorageBackend = {
  read: () => Promise<
    | { status: "ok"; content: string }
    | { status: "missing" }
    | { status: "error"; error: string }
  >;
  write: (
    content: string,
  ) => Promise<{ status: "ok" } | { status: "error"; error: string }>;
};

/** Used when running outside Electron (e.g. plain browser dev server). */
const localStorageBackend: StorageBackend = {
  read: async () => {
    const content = localStorage.getItem(FALLBACK_LOCAL_STORAGE_KEY);
    return content === null
      ? { status: "missing" }
      : { status: "ok", content };
  },
  write: async (content) => {
    try {
      localStorage.setItem(FALLBACK_LOCAL_STORAGE_KEY, content);
      return { status: "ok" };
    } catch (e) {
      return { status: "error", error: String(e) };
    }
  },
};

function getBackend(): StorageBackend {
  return typeof window !== "undefined" && window.presetStorage
    ? window.presetStorage
    : localStorageBackend;
}

/** Validates every entry on its own; invalid ones are quarantined, not dropped. */
function parseEntries(
  rawPresets: unknown[],
  rawLatestPresetIds: unknown[],
  previouslyQuarantined: unknown[],
): Omit<PresetStoreData, "hydrated"> {
  const presets: PersistedPreset[] = [];
  const quarantined: unknown[] = [];
  const seenIds = new Set<number>();

  for (const raw of [...rawPresets, ...previouslyQuarantined]) {
    const result = persistedPresetSchema.safeParse(raw);
    if (result.success && !seenIds.has(result.data.id)) {
      seenIds.add(result.data.id);
      presets.push(result.data);
    } else {
      if (!result.success) {
        console.error("Quarantining invalid preset", raw, result.error);
      }
      quarantined.push(raw);
    }
  }

  const latestPresetIds = new Map<string, number>();
  for (const raw of rawLatestPresetIds) {
    const result = latestPresetIdEntrySchema.safeParse(raw);
    if (result.success && seenIds.has(result.data[1])) {
      latestPresetIds.set(result.data[0], result.data[1]);
    }
  }

  return { presets, latestPresetIds, quarantined };
}

function serialize(state: PresetStoreData): string {
  const file: PresetFile = {
    version: 1,
    presets: state.presets,
    latestPresetIds: Array.from(state.latestPresetIds.entries()),
    quarantined: state.quarantined,
  };
  return JSON.stringify(file, null, 2);
}

// Writes are serialized and coalesced: at most one in flight, and only the
// most recent state is written afterwards.
let writeInFlight = false;
let pendingContent: string | null = null;
let lastWrittenContent: string | null = null;
let writesBlocked = false;

async function flushWrites() {
  if (writeInFlight || writesBlocked) {
    return;
  }
  writeInFlight = true;
  try {
    while (pendingContent !== null) {
      const content = pendingContent;
      pendingContent = null;
      if (content === lastWrittenContent) {
        continue;
      }
      const result = await getBackend().write(content);
      if (result.status === "ok") {
        lastWrittenContent = content;
      } else {
        toastError("Saving presets failed", result.error);
      }
    }
  } finally {
    writeInFlight = false;
  }
}

function scheduleWrite(state: PresetStoreData) {
  pendingContent = serialize(state);
  void flushWrites();
}

async function hydrate() {
  const backend = getBackend();
  let data: Omit<PresetStoreData, "hydrated"> | null = null;

  try {
    const result = await backend.read();
    if (result.status === "ok") {
      const file = presetFileSchema.parse(JSON.parse(result.content));
      data = parseEntries(file.presets, file.latestPresetIds, file.quarantined);
      lastWrittenContent = result.content;
    } else if (result.status === "error") {
      throw new Error(result.error);
    }
    // "missing": first start, nothing stored yet.
  } catch (e) {
    console.error("Loading presets failed", e);
    // Do not overwrite whatever is on disk with an empty store.
    writesBlocked = true;
    toastError(
      "Loading presets failed",
      "Presets could not be loaded. Changes will not be saved until the app is restarted.",
    );
  }

  if (data !== null && data.quarantined.length > 0) {
    toastError(
      "Some presets are invalid",
      `${data.quarantined.length} preset(s) could not be loaded and were kept aside unchanged.`,
    );
  }

  usePresetStore.setState({ ...(data ?? {}), hydrated: true });
}

usePresetStore.subscribe((state, prev) => {
  if (!state.hydrated || !prev.hydrated) {
    return;
  }
  if (
    state.presets !== prev.presets ||
    state.latestPresetIds !== prev.latestPresetIds ||
    state.quarantined !== prev.quarantined
  ) {
    scheduleWrite(state);
  }
});

void hydrate();
