/**
 * @file buffer1Namespace.ts
 * @description TypeScript implementation of Winder1 namespace with Zod schema validation.
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
import { useMemo } from "react";
import {
  createTimeSeries,
  TimeSeries,
  TimeSeriesValue,
} from "@/lib/timeseries";
import { Winder2NamespaceStore } from "@/machines/winder/winder2/winder2Namespace";

// ========== Event Schema Definitions ==========

/**
 * Machine operation mode num
 */
export const modeSchema = z.enum(["Standby", "FillingBuffer", "EmptyingBuffer"]);

/**
 * Mode state event schema
 */
export const modeStateEventDataSchema = z.object({
  mode: modeSchema,
});

// ========== Event Schemas with Wrappers ==========

export const modeStateEventSchema = eventSchema(modeStateEventDataSchema);

// ========== Type Inferences ==========

export type Mode = z.infer<typeof modeSchema>;
export type ModeStateEvent = z.infer<typeof modeStateEventSchema>;

export type Buffer1NamespaceStore = {
  // State events (latest only)
  modeState: ModeStateEvent | null;
};

/**
 * Factory function to create a new Buffer1 namespace store
 * @returns A new Zustand store instance for Buffer1 namespace
 */
export const createBuffer1NamespaceStore =
  (): StoreApi<Buffer1NamespaceStore> =>
    create<Buffer1NamespaceStore>(() => {
      return {
        // State events (latest only)
        modeState: null,

        // Metric events (cached for 1 hour)
      };
    });

/**
 * Creates a message handler for Buffer1 namespace events with validation and appropriate caching strategies
 * @param store The store to update when messages are received
 * @param throttledUpdater Throttled updater for batching updates at 60 FPS
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
      if (eventName === "ModeStateEvent") {
        console.log("ModeStateEvent", event);
        updateStore((state) => ({
          ...state,
          modeState: event as ModeStateEvent,
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

/**
 * Create the Buffer1 namespace implementation
 */

const useBuffer1NamespaceImplementation =
  createNamespaceHookImplementation<Buffer1NamespaceStore>({
    createStore: createBuffer1NamespaceStore,
    createEventHandler: buffer1MessageHandler,
  });