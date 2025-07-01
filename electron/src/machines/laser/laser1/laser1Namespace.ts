/**
 * @file laser1Namespace.ts
 * @description TypeScript implementation of Laser1 namespace with Zod schema validation.
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
 * Live values from Laser (60 FPS)
 */
export const liveValuesEventDataSchema = z.object({
  diameter: z.number(),
});

/**
 * State event from Laser (on state changes only)
 */
export const stateEventDataSchema = z.object({
  laser_state: z.object({
    higher_tolerance: z.number(),
    lower_tolerance: z.number(),
    target_diameter: z.number(),
  }),
});

// ========== Event Schemas with Wrappers ==========
export const liveValuesEventSchema = eventSchema(liveValuesEventDataSchema);
export const stateEventSchema = eventSchema(stateEventDataSchema);

// ========== Type Inferences ==========
export type LiveValuesEvent = z.infer<typeof liveValuesEventSchema>;
export type StateEvent = z.infer<typeof stateEventSchema>;

export type Laser1NamespaceStore = {
  // Single state event from server
  state: StateEvent | null;

  // Time series data for live values
  diameter: TimeSeries;
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
 * Factory function to create a new Laser1 namespace store
 * @returns A new Zustand store instance for Laser1 namespace
 */
export const createLaser1NamespaceStore = (): StoreApi<Laser1NamespaceStore> =>
  create<Laser1NamespaceStore>(() => {
    return {
      state: null,
      diameter: diameter,
    };
  });

/**
 * Creates a message handler for Laser1 namespace events with validation and appropriate caching strategies
 * @param store The store to update when messages are received
 * @param throttledUpdater Throttled updater for batching updates at 60 FPS
 * @returns A message handler function
 */
export function laser1MessageHandler(
  store: StoreApi<Laser1NamespaceStore>,
  throttledUpdater: ThrottledStoreUpdater<Laser1NamespaceStore>,
): EventHandler {
  return (event: Event<any>) => {
    const eventName = event.name;

    // Helper function to update store through buffer
    const updateStore = (
      updater: (state: Laser1NamespaceStore) => Laser1NamespaceStore,
    ) => {
      throttledUpdater.updateWith(updater);
    };

    try {
      // Apply appropriate caching strategy based on event type
      if (eventName === "StateEvent") {
        const stateEvent = stateEventSchema.parse(event);
        updateStore((state) => ({
          ...state,
          state: stateEvent,
        }));
      }
      // Live values events (keep for 1 hour)
      else if (eventName === "LiveValuesEvent") {
        const liveValuesEvent = liveValuesEventSchema.parse(event);
        const timeseriesValue: TimeSeriesValue = {
          value: liveValuesEvent.data.diameter,
          timestamp: event.ts,
        };
        updateStore((state) => ({
          ...state,
          diameter: addDiameter(state.diameter, timeseriesValue),
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
 * Create the Laser1 namespace implementation
 */
const useLaser1NamespaceImplementation =
  createNamespaceHookImplementation<Laser1NamespaceStore>({
    createStore: createLaser1NamespaceStore,
    createEventHandler: laser1MessageHandler,
  });

export function useLaser1Namespace(
  machine_identification_unique: MachineIdentificationUnique,
): Laser1NamespaceStore {
  // Generate namespace ID from validated machine ID
  const namespaceId: NamespaceId = {
    type: "machine",
    machine_identification_unique,
  };

  // Use the implementation with validated namespace ID
  return useLaser1NamespaceImplementation(namespaceId);
}
