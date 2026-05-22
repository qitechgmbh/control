/**
 * @file buffer1Namespace.ts
 * @description TypeScript implementation of Buffer1 namespace with Zod schema validation.
 */

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
} from "../../../client/socketioStore";
import { MachineIdentificationUnique } from "@/machines/types";

// ========== Event Schema Definitions ==========

/**
 * Machine operation mode enum
 */
export const modeSchema = z.enum([
  "Standby",
  "FillingBuffer",
  "EmptyingBuffer",
]);
export type Mode = z.infer<typeof modeSchema>;

/**
 * Consolidated live values event schema (30FPS data)
 */
export const liveValuesEventDataSchema = z.object({});

/**
 * Mode state event schema
 */
export const modeStateSchema = z.object({
  mode: modeSchema,
});

/**
 *  Connected machine state scheme
 */
export const machineIdentificationSchema = z.object({
  vendor: z.number(),
  machine: z.number(),
});

export const machineIdentificationUniqueSchema = z.object({
  machine_identification: machineIdentificationSchema,
  serial: z.number(),
});

export const connectedMachineStateSchema = z.object({
  machine_identification_unique: machineIdentificationUniqueSchema.nullable(),
  is_available: z.boolean(),
});

/**
 * Consolidated state event schema (state changes only)
 */
export const stateEventDataSchema = z.object({
  mode_state: modeStateSchema,
  connected_machine_state: connectedMachineStateSchema,
});

// ========== Event Schemas with Wrappers ==========

export const liveValuesEventSchema = eventSchema(liveValuesEventDataSchema);
export const stateEventSchema = eventSchema(stateEventDataSchema);

// ========== Type Inferences ==========

export type StateEvent = z.infer<typeof stateEventSchema>;

export type Buffer1NamespaceStore = {
  // State events (latest only)
  state: StateEvent | null;
};

/**
 * Creates a message handler for Buffer1 namespace events with validation and appropriate caching strategies
 * @param store The store to update when messages are received
 * @param throttledUpdater Throttled updater for batching updates at 30 FPS
 * @returns A message handler function
 */
export function buffer1MessageHandler(
  store: StoreApi<Buffer1NamespaceStore>,
  throttledUpdater: ThrottledStoreUpdater<Buffer1NamespaceStore>,
): EventHandler {
  return (event: Event<any>) => {
    const eventName = event.name;

    // Helper function to update store through buffer
    const updateStore = (
      updater: (state: Buffer1NamespaceStore) => Buffer1NamespaceStore,
    ) => {
      throttledUpdater.updateWith(updater);
    };
    try {
      // Apply appropriate caching strategy based on event type
      // State events (keep only the latest)
      if (eventName === "StateEvent") {
        const stateEvent = stateEventSchema.parse(event);
        updateStore((state) => ({
          ...state,
          state: stateEvent,
        }));
      } else if (eventName === "LiveValuesEvent") {
        const liveValuesEvent = liveValuesEventSchema.parse(event);
        const timestamp = event.ts;
        updateStore((state) => ({
          ...state,
        }));
      } else {
        handleUnhandledEventError(eventName);
      }
    } catch (error) {
      console.error(`Unexpected error processing ${eventName} event:`, error);
      throw error;
    }
  };
}

/**
 * Factory function to create a new Buffer1 namespace store
 * @returns A new Zustand store instance for Buffer1 namespace
 */
export const createBuffer1NamespaceStore =
  (): StoreApi<Buffer1NamespaceStore> =>
    create<Buffer1NamespaceStore>(() => {
      return {
        state: null,
      };
    });

/**
 * Create the Buffer1 namespace implementation
 */

const useBuffer1NamespaceImplementation =
  createNamespaceHookImplementation<Buffer1NamespaceStore>({
    createStore: createBuffer1NamespaceStore,
    createEventHandler: buffer1MessageHandler,
  });

export function useBuffer1Namespace(
  machine_identification_unique: MachineIdentificationUnique,
): Buffer1NamespaceStore {
  // Generate namespace ID from validated machine ID
  const namespaceId: NamespaceId = {
    type: "machine",
    machine_identification_unique,
  };

  // Use the implementation with validated namespace ID
  return useBuffer1NamespaceImplementation(namespaceId);
}
