/**
 * @file mock2Namespace.ts
 * @description TypeScript implementation of Mock2 namespace with Zod schema validation.
 */

import { StoreApi } from "zustand";
import { create } from "zustand";
import { boolean, z } from "zod";
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
 * Mode enum for Mock Machine
 */
export const modeSchema = z.enum(["Standby", "Running"]);

/**
 * Sine wave state schema
 */
export const sineWaveStateSchema = z.object({
  frequency: z.number(),
});

/**
 * Mode state schema
 */
export const modeStateSchema = z.object({
  mode: modeSchema,
});

/**
 * Connected machine state schema
 */
const machineIdentificationSchema = z.object({
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
 * Live values event schema (time-series data)
 */
export const liveValuesEventDataSchema = z.object({
  sine_wave_amplitude: z.number(),
});

/**
 * State event schema (consolidated state)
 */
export const stateEventDataSchema = z.object({
  is_default_state: z.boolean(),
  sine_wave_state: sineWaveStateSchema,
  mode_state: modeStateSchema,
  connected_machine_state: connectedMachineStateSchema,
});

// ========== Event Schemas with Wrappers ==========
export const liveValuesEventSchema = eventSchema(liveValuesEventDataSchema);
export const stateEventSchema = eventSchema(stateEventDataSchema);

// ========== Type Inferences ==========
export type Mode = z.infer<typeof modeSchema>;
export type SineWaveState = z.infer<typeof sineWaveStateSchema>;
export type ModeState = z.infer<typeof modeStateSchema>;
export type LiveValuesEvent = z.infer<typeof liveValuesEventSchema>;
export type StateEvent = z.infer<typeof stateEventSchema>;

export type Mock2NamespaceStore = {
  // Single state event from server
  state: StateEvent | null;
  defaultState: StateEvent | null;

  // Time series data for live values
  sineWave: TimeSeries;
};

// Constants for time durations
const TWENTY_MILLISECOND = 20;
const ONE_SECOND = 1000;
const FIVE_SECOND = 5 * ONE_SECOND;
const ONE_HOUR = 60 * 60 * ONE_SECOND;

const { initialTimeSeries: sineWave, insert: addSineWave } = createTimeSeries(
  TWENTY_MILLISECOND,
  ONE_SECOND,
  FIVE_SECOND,
  ONE_HOUR,
);

/**
 * Factory function to create a new Mock2 namespace store
 * @returns A new Zustand store instance for Mock2 namespace
 */
export const createMock2NamespaceStore = (): StoreApi<Mock2NamespaceStore> =>
  create<Mock2NamespaceStore>(() => {
    return {
      state: null,
      defaultState: null,
      sineWave: sineWave,
    };
  });

/**
 * Creates a message handler for Mock2 namespace events with validation and appropriate caching strategies
 * @param store The store to update when messages are received
 * @param throttledUpdater Throttled updater for batching updates at 60 FPS
 * @returns A message handler function
 */
export function mock2MessageHandler(
  store: StoreApi<Mock2NamespaceStore>,
  throttledUpdater: ThrottledStoreUpdater<Mock2NamespaceStore>,
): EventHandler {
  return (event: Event<any>) => {
    const eventName = event.name;

    // Helper function to update store through buffer
    const updateStore = (
      updater: (state: Mock2NamespaceStore) => Mock2NamespaceStore,
    ) => {
      throttledUpdater.updateWith(updater);
    };

    try {
      // State events (latest only)
      if (eventName === "StateEvent") {
        const stateEvent = stateEventSchema.parse(event);
        updateStore((state) => ({
          ...state,
          state: stateEvent,
          // only set default state if is_default_state is true
          defaultState: stateEvent.data.is_default_state
            ? stateEvent
            : state.defaultState,
        }));
      }
      // Live values events (time-series data)
      else if (eventName === "LiveValuesEvent") {
        const liveValuesEvent = liveValuesEventSchema.parse(event);
        const timeseriesValue: TimeSeriesValue = {
          value: liveValuesEvent.data.sine_wave_amplitude ?? 0,
          timestamp: liveValuesEvent.ts,
        };
        updateStore((state) => ({
          ...state,
          sineWave: addSineWave(state.sineWave, timeseriesValue),
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
 * Create the Mock2 namespace implementation
 */
const useMock2NamespaceImplementation =
  createNamespaceHookImplementation<Mock2NamespaceStore>({
    createStore: createMock2NamespaceStore,
    createEventHandler: mock2MessageHandler,
  });

export function useMock2Namespace(
  machine_identification_unique: MachineIdentificationUnique,
): Mock2NamespaceStore {
  // Generate namespace ID from validated machine ID
  const namespaceId: NamespaceId = {
    type: "machine",
    machine_identification_unique,
  };

  // Use the implementation with validated namespace ID
  return useMock2NamespaceImplementation(namespaceId);
}
