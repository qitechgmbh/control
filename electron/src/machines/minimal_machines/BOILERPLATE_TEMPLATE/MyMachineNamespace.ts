// ============================================================================
// MyMachineNamespace.ts — WebSocket event schema, store, and namespace hook
// ============================================================================
// This file is the glue between the backend socket.io events and the React
// component tree.  It has four sections:
//
//   1. Event Schema  — Zod schemas that mirror the Rust `StateEvent` struct
//   2. Store         — Zustand store holding the latest parsed state
//   3. Message Handler — maps incoming event names to store updates
//   4. Namespace Hook  — the public hook consumed by useMyMachine.ts
//
// FIND & REPLACE to adapt this template:
//   MyMachine             → YourMachineName  (e.g. WagoDiMachine)
//   myMachine             → yourMachineName  (camelCase, e.g. wagoDiMachine)
//   myMachineSerialRoute  → the route export from routes.tsx
// ============================================================================

import { StoreApi } from "zustand";
import { create } from "zustand";
import { z } from "zod";
import {
  EventHandler,
  eventSchema,
  Event,
  handleUnhandledEventError,
  NamespaceId,
  createNamespaceHookImplementation,
  ThrottledStoreUpdater,
} from "@/client/socketioStore";
import { MachineIdentificationUnique } from "@/machines/types";

// ============================================================================
// 1. Event Schema
//
// Mirror every field from the Rust `StateEvent` struct in api.rs exactly.
// Field names must be snake_case to match the JSON Rust serialises.
//
// Examples:
//   z.boolean()                 — bool
//   z.number()                  — f32 / f64 / u16 / i32 …
//   z.array(z.boolean()).length(4) — [bool; 4]
//   z.string()                  — String
//   z.tuple([z.number(), z.string()]) — (f64, String)
// ============================================================================
export const stateEventDataSchema = z.object({
  // TODO: mirror your Rust StateEvent fields here, e.g.:
  //   outputs: z.array(z.boolean()).length(4),
  //   value:   z.number(),
});

export const stateEventSchema = eventSchema(stateEventDataSchema);

export type StateEvent = z.infer<typeof stateEventDataSchema>;

// ============================================================================
// 2. Store
//
// One store per machine instance (keyed by namespace ID internally).
// `state: null` means "not yet received from backend".
// ============================================================================
export type MyMachineNamespaceStore = {
  state: StateEvent | null;
};

export const createMyMachineNamespaceStore =
  (): StoreApi<MyMachineNamespaceStore> =>
    create<MyMachineNamespaceStore>(() => ({
      state: null,
    }));

// ============================================================================
// 3. Message Handler
//
// Called for every socket.io event on this machine's namespace.
// Add a branch for each event name your backend emits.
//
// The most common pattern (single "StateEvent") is shown below.
// For machines with multiple event types, add more `if` branches.
// ============================================================================
export function myMachineMessageHandler(
  store: StoreApi<MyMachineNamespaceStore>,
  throttledUpdater: ThrottledStoreUpdater<MyMachineNamespaceStore>,
): EventHandler {
  return (event: Event<any>) => {
    const updateStore = (
      updater: (state: MyMachineNamespaceStore) => MyMachineNamespaceStore,
    ) => throttledUpdater.updateWith(updater);

    try {
      if (event.name === "StateEvent") {
        // TODO: if your Rust api.rs emits a different event name, change it here.
        const parsed = stateEventSchema.parse(event);
        updateStore(() => ({ state: parsed.data }));
      } else {
        handleUnhandledEventError(event.name);
      }
    } catch (error) {
      console.error(`Error processing ${event.name}:`, error);
      throw error;
    }
  };
}

// ============================================================================
// 4. Namespace Hook
//
// Returns the store for a given machine instance.
// Used by useMyMachine.ts — do not call this directly from components.
// ============================================================================
const useMyMachineNamespaceImplementation =
  createNamespaceHookImplementation<MyMachineNamespaceStore>({
    createStore: createMyMachineNamespaceStore,
    createEventHandler: myMachineMessageHandler,
  });

export function useMyMachineNamespace(
  machine_identification_unique: MachineIdentificationUnique,
): MyMachineNamespaceStore {
  const namespaceId: NamespaceId = {
    type: "machine",
    machine_identification_unique,
  };

  return useMyMachineNamespaceImplementation(namespaceId);
}
