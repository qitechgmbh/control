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
import {
  createTimeSeries,
  TimeSeries,
  TimeSeriesValue,
} from "@/lib/timeseries";

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
export const liveValuesEventDataSchema = z.object({
  puller_speed: z.number(),
});

/**
 * Puller regulation type enum
 */
export const pullerRegulationSchema = z.enum(["Speed", "Diameter"]);
export type PullerRegulation = z.infer<typeof pullerRegulationSchema>;

/**
 * Puller state event schmema
 */
export const pullerStateSchema = z.object({
  regulation: pullerRegulationSchema,
  target_speed: z.number(),
  target_diameter: z.number(),
  forward: z.boolean(),
});

/**
 * Mode state event schema
 */
export const modeStateSchema = z.object({
  mode: modeSchema,
});

export const currentInputSpeedSchema = z.object({
  current_input_speed: z.number(),
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
  puller_state: pullerStateSchema,
  connected_machine_state: connectedMachineStateSchema,
  current_input_speed_state: currentInputSpeedSchema,
});

// ========== Event Schemas with Wrappers ==========

export const liveValuesEventSchema = eventSchema(liveValuesEventDataSchema);
export const stateEventSchema = eventSchema(stateEventDataSchema);

// ========== Type Inferences ==========

export type StateEvent = z.infer<typeof stateEventSchema>;

export type Buffer1NamespaceStore = {
  // State events (latest only)
  state: StateEvent | null;
  defaultState: StateEvent | null;

  // Time series data for live values
  pullerSpeed: TimeSeries;
};

// Constants for time durations
const TWENTY_MILLISECOND = 20;
const ONE_SECOND = 1000;
const FIVE_SECOND = 5 * ONE_SECOND;
const ONE_HOUR = 60 * 60 * ONE_SECOND;

const { initialTimeSeries: pullerSpeed, insert: addPullerSpeed } =
  createTimeSeries(TWENTY_MILLISECOND, ONE_SECOND, FIVE_SECOND, ONE_HOUR);

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
        // Parse and validate the live values event
        const liveValuesEvent = liveValuesEventSchema.parse(event);

        // Extract values and add to time series
        const { puller_speed } = liveValuesEvent.data;

        const timestamp = liveValuesEvent.ts;
        updateStore((state) => {
          const newState = { ...state };

          // Add puller speed
          const pullerSpeedValue: TimeSeriesValue = {
            value: puller_speed,
            timestamp,
          };
          newState.pullerSpeed = addPullerSpeed(
            state.pullerSpeed,
            pullerSpeedValue,
          );

          return newState;
        });
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
        // State event from server
        state: null,
        defaultState: null,

        // Time series data for live values
        pullerSpeed,
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
