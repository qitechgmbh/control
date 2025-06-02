/**
 * @file dre1Namespace.ts
 * @description TypeScript implementation of Dre1 namespace with Zod schema validation.
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
import {
  createTimeSeries,
  TimeSeries,
  TimeSeriesValue,
} from "@/lib/timeseries";

// ========== Event Schema Definitions ==========
/**
 * Measurements diameter from DRE
 */
export const diameterEventDataSchema = z.object({
  diameter: z.number(),
});

export const dreStateEventDataSchema = z.object({
  higher_tolerance: z.number(),
  lower_tolerance: z.number(),
  target_diameter: z.number(),
});
// ========== Event Schemas with Wrappers ==========
export const diameterEventSchema = eventSchema(diameterEventDataSchema);
export const dreStateEventSchema = eventSchema(dreStateEventDataSchema);

// ========== Type Inferences ==========
export type DiameterEvent = z.infer<typeof diameterEventSchema>;

export type DreStateEvent = z.infer<typeof dreStateEventSchema>;

export type Dre1NamespaceStore = {
  // State events (latest only)
  dreState: DreStateEvent | null;
  // Metric events (cached for 1 hour)
  dreDiameter: TimeSeries;
};

// Constants for time durations
const TWENTY_MILLISECOND = 20;
const ONE_SECOND = 1000;
const FIVE_SECOND = 5 * ONE_SECOND;
const ONE_HOUR = 60 * 60 * ONE_SECOND;
const { initialTimeSeries: diameter, insert: addDiameter } = createTimeSeries(
  TWENTY_MILLISECOND,
  ONE_SECOND,
  FIVE_SECOND,
  ONE_HOUR,
);
/**
 * Factory function to create a new Dre1 namespace store
 * @returns A new Zustand store instance for Dre1 namespace
 */
export const createDre1NamespaceStore = (): StoreApi<Dre1NamespaceStore> =>
  create<Dre1NamespaceStore>(() => {
    return {
      dreState: null,
      dreDiameter: diameter,
    };
  });

/**
 * Creates a message handler for Dre1 namespace events with validation and appropriate caching strategies
 * @param store The store to update when messages are received
 * @param throttledUpdater Throttled updater for batching updates at 60 FPS
 * @returns A message handler function
 */
export function dre1MessageHandler(
  store: StoreApi<Dre1NamespaceStore>,
  throttledUpdater: ThrottledStoreUpdater<Dre1NamespaceStore>,
): EventHandler {
  return (event: Event<any>) => {
    const eventName = event.name;

    // Helper function to update store through buffer
    const updateStore = (
      updater: (state: Dre1NamespaceStore) => Dre1NamespaceStore,
    ) => {
      throttledUpdater.updateWith(updater);
    };

    try {
      // Apply appropriate caching strategy based on event type
      if (eventName === "DreStateEvent") {
        updateStore((state) => ({
          ...state,
          dreState: event as DreStateEvent,
        }));
      }
      // Metric events (keep for 1 hour)
      else if (eventName === "DiameterEvent") {
        const diameterEvent = event as DiameterEvent;
        const timeseriesValue: TimeSeriesValue = {
          value: diameterEvent.data.diameter,
          timestamp: event.ts,
        };
        updateStore((state) => ({
          ...state,
          dreDiameter: addDiameter(state.dreDiameter, timeseriesValue),
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
 * Create the Dre1 namespace implementation
 */
const useDre1NamespaceImplementation =
  createNamespaceHookImplementation<Dre1NamespaceStore>({
    createStore: createDre1NamespaceStore,
    createEventHandler: dre1MessageHandler,
  });

export function useDre1Namespace(
  machine_identification_unique: MachineIdentificationUnique,
): Dre1NamespaceStore {
  // Generate namespace ID from validated machine ID
  const namespaceId: NamespaceId = {
    type: "machine",
    machine_identification_unique,
  };

  // Use the implementation with validated namespace ID
  return useDre1NamespaceImplementation(namespaceId);
}
