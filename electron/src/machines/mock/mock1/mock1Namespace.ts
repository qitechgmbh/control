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
 * Sine wave data from Mock Machine (only amplitude)
 */
export const sineWaveEventDataSchema = z.object({
  amplitude: z.number(),
});

/**
 * Sine wave state event (frequency only)
 */
export const mockStateEventDataSchema = z.object({
  frequency: z.number(),
});

/**
 * Mode state event schema
 */
export const modeStateEventDataSchema = z.object({
  mode: modeSchema,
});

// ========== Event Schemas with Wrappers ==========
export const sineWaveEventSchema = eventSchema(sineWaveEventDataSchema);
export const mockStateEventSchema = eventSchema(mockStateEventDataSchema);
export const modeStateEventSchema = eventSchema(modeStateEventDataSchema);

// ========== Type Inferences ==========
export type Mode = z.infer<typeof modeSchema>;
export type SineWaveEvent = z.infer<typeof sineWaveEventSchema>;
export type MockStateEvent = z.infer<typeof mockStateEventSchema>;
export type ModeStateEvent = z.infer<typeof modeStateEventSchema>;

export type Mock1NamespaceStore = {
  // State events (latest only)
  mockState: MockStateEvent | null;
  modeState: ModeStateEvent | null;
  // Metric events (cached for 1 hour)
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
 * Factory function to create a new Mock1 namespace store
 * @returns A new Zustand store instance for Mock1 namespace
 */
export const createMock1NamespaceStore = (): StoreApi<Mock1NamespaceStore> =>
  create<Mock1NamespaceStore>(() => {
    return {
      mockState: null,
      modeState: null,
      sineWave: sineWave,
    };
  });

/**
 * Creates a message handler for Mock1 namespace events with validation and appropriate caching strategies
 * @param store The store to update when messages are received
 * @param throttledUpdater Throttled updater for batching updates at 60 FPS
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
      // Apply appropriate caching strategy based on event type
      if (eventName === "MockStateEvent") {
        console.log("MockStateEvent", event);
        updateStore((state) => ({
          ...state,
          mockState: event as MockStateEvent,
        }));
      }
      // Mode state events (latest only)
      else if (eventName === "ModeStateEvent") {
        console.log("ModeStateEvent", event);
        updateStore((state) => ({
          ...state,
          modeState: event as ModeStateEvent,
        }));
      }
      // Metric events (keep for 1 hour)
      else if (eventName === "SineWaveEvent") {
        const timeseriesValue: TimeSeriesValue = {
          value: event.data.amplitude ?? 0,
          timestamp: event.ts,
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
