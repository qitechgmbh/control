/**
 * @file mock1Namespace.ts
 * @description TypeScript implementation of Mock1 namespace with Zod schema validation.
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
 * Mode enum for Mock Machine
 */
export const modeSchema = z.enum(["Standby", "Running"]);

/**
 * Mode state schema
 */
export const modeStateSchema = z.object({
  mode: modeSchema,
});

/**
 * Live values event schema (time-series data)
 */
export const liveValuesEventDataSchema = z.object({
  amplitude_sum: z.number(),
  amplitude1: z.number(),
  amplitude2: z.number(),
  amplitude3: z.number(),
});

/**
 * State event schema (consolidated state)
 */
export const stateEventDataSchema = z.object({
  is_default_state: z.boolean(),
  frequency1: z.number(),
  frequency2: z.number(),
  frequency3: z.number(),
  mode_state: modeStateSchema,
});

// ========== Event Schemas with Wrappers ==========
export const liveValuesEventSchema = eventSchema(liveValuesEventDataSchema);
export const stateEventSchema = eventSchema(stateEventDataSchema);

// ========== Type Inferences ==========
export type Mode = z.infer<typeof modeSchema>;
export type ModeState = z.infer<typeof modeStateSchema>;
export type LiveValuesEvent = z.infer<typeof liveValuesEventSchema>;
export type StateEvent = z.infer<typeof stateEventSchema>;

export type Mock1NamespaceStore = {
  // Single state event from server
  state: StateEvent | null;
  defaultState: StateEvent | null;

  // Time series data for live values
  sineWaveSum: TimeSeries;
  sineWave1: TimeSeries;
  sineWave2: TimeSeries;
  sineWave3: TimeSeries;
};

const { initialTimeSeries: sineWave, insert: addSineWave } = createTimeSeries();

/**
 * Factory function to create a new Mock1 namespace store
 * @returns A new Zustand store instance for Mock1 namespace
 */
export const createMock1NamespaceStore = (): StoreApi<Mock1NamespaceStore> => {
  return create<Mock1NamespaceStore>(() => {
    return {
      state: null,
      defaultState: null,
      sineWaveSum: sineWave,
      sineWave1: sineWave,
      sineWave2: sineWave,
      sineWave3: sineWave,
    };
  });
};

/**
 * Creates a message handler for Mock1 namespace events with validation and appropriate caching strategies
 * @param store The store to update when messages are received
 * @param throttledUpdater Throttled updater for batching updates at 30 FPS
 * @returns A message handler function
 */
export function mock1MessageHandler(
  store: StoreApi<Mock1NamespaceStore>,
  throttledUpdater: ThrottledStoreUpdater<Mock1NamespaceStore>,
): EventHandler {
  return (event: Event<any>) => {
    const eventName = event.name;

    // Helper function to update store through buffer
    const updateStore = (
      updater: (state: Mock1NamespaceStore) => Mock1NamespaceStore,
    ) => {
      throttledUpdater.updateWith(updater);
    };

    try {
      // State events (latest only)
      if (eventName === "StateEvent") {
        const stateEvent = stateEventSchema.parse(event);
        console.log("StateEvent", stateEvent);
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
        const wave1Value: TimeSeriesValue = {
          value: liveValuesEvent.data.amplitude1 ?? 0,
          timestamp: liveValuesEvent.ts,
        };
        const wave2Value: TimeSeriesValue = {
          value: liveValuesEvent.data.amplitude2 ?? 0,
          timestamp: liveValuesEvent.ts,
        };
        const wave3Value: TimeSeriesValue = {
          value: liveValuesEvent.data.amplitude3 ?? 0,
          timestamp: liveValuesEvent.ts,
        };
        const waveSumValue: TimeSeriesValue = {
          value: liveValuesEvent.data.amplitude_sum ?? 0,
          timestamp: liveValuesEvent.ts,
        };
        updateStore((state) => ({
          ...state,
          sineWaveSum: addSineWave(state.sineWaveSum, waveSumValue),
          sineWave1: addSineWave(state.sineWave1, wave1Value),
          sineWave2: addSineWave(state.sineWave2, wave2Value),
          sineWave3: addSineWave(state.sineWave3, wave3Value),
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
 * Create the Mock1 namespace implementation
 */
const useMock1NamespaceImplementation =
  createNamespaceHookImplementation<Mock1NamespaceStore>({
    createStore: createMock1NamespaceStore,
    createEventHandler: mock1MessageHandler,
  });

export function useMock1Namespace(
  machine_identification_unique: MachineIdentificationUnique,
): Mock1NamespaceStore {
  // Generate namespace ID from validated machine ID
  const namespaceId: NamespaceId = {
    type: "machine",
    machine_identification_unique,
  };

  // Use the implementation with validated namespace ID
  return useMock1NamespaceImplementation(namespaceId);
}
