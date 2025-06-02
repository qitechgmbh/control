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
} from "../../../client/socketioStore";
import { MachineIdentificationUnique } from "@/machines/types";
import {
  createTimeSeries,
  TimeSeries,
  TimeSeriesValue,
} from "@/lib/timeseries";

// ========== Event Schema Definitions ==========
/**
 * Measurements diameter from Laser
 */
export const diameterEventDataSchema = z.object({
  diameter: z.number(),
});

export const laserStateEventDataSchema = z.object({
  higher_tolerance: z.number(),
  lower_tolerance: z.number(),
  target_diameter: z.number(),
});
// ========== Event Schemas with Wrappers ==========
export const diameterEventSchema = eventSchema(diameterEventDataSchema);
export const laserStateEventSchema = eventSchema(laserStateEventDataSchema);

// ========== Type Inferences ==========
export type DiameterEvent = z.infer<typeof diameterEventSchema>;

export type LaserStateEvent = z.infer<typeof laserStateEventSchema>;

export type Laser1NamespaceStore = {
  // State events (latest only)
  laserState: LaserStateEvent | null;
  // Metric events (cached for 1 hour)
  laserDiameter: TimeSeries;
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
      laserState: null,
      laserDiameter: diameter,
    };
  });

/**
 * Creates a message handler for Laser1 namespace events with validation and appropriate caching strategies
 * @param store The store to update when messages are received
 * @returns A message handler function
 */
export function laser1MessageHandler(
  store: StoreApi<Laser1NamespaceStore>,
): EventHandler {
  return (event: Event<any>) => {
    const eventName = event.name;

    try {
      // Apply appropriate caching strategy based on event type
      if (eventName === "LaserStateEvent") {
        store.setState((state) => ({
          ...state,
          laserState: event as LaserStateEvent,
        }));
      }
      // Metric events (keep for 1 hour)
      else if (eventName === "DiameterEvent") {
        const diameterEvent = event as DiameterEvent;
        const timeseriesValue: TimeSeriesValue = {
          value: diameterEvent.data.diameter,
          timestamp: event.ts,
        };
        store.setState((state) => ({
          ...state,
          dreDiameter: addDiameter(state.laserDiameter, timeseriesValue),
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
