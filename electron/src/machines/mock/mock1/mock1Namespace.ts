/**
 * @file mock1Namespace.ts
 * @description TypeScript implementation of Mock1 namespace with Zod schema validation.
 */

import { StoreApi } from "zustand";
import { create } from "zustand";
import { produce } from "immer";
import { z } from "zod";
import {
  EventHandler,
  eventSchema,
  Event,
  handleEventValidationError,
  handleUnhandledEventError,
  NamespaceId,
  createNamespaceHookImplementation,
} from "../../../client/socketioStore";
import { MachineIdentificationUnique } from "@/machines/types";
import { useRef } from "react";
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
const ONE_SECOND = 1000;
const ONE_HOUR = 60 * 60 * ONE_SECOND;

const { initialTimeSeries: sineWave, insert: addSineWave } = createTimeSeries(
  ONE_SECOND,
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
 * @returns A message handler function
 */
export function mock1MessageHandler(
  store: StoreApi<Mock1NamespaceStore>,
): EventHandler {
  return (event: Event<any>) => {
    const eventName = event.name;

    try {
      // Apply appropriate caching strategy based on event type
      if (eventName === "MockStateEvent") {
        store.setState(
          produce(store.getState(), (state) => {
            state.mockState = {
              name: event.name,
              data: mockStateEventDataSchema.parse(event.data),
              ts: event.ts,
            };
          }),
        );
      }
      // Mode state events (latest only)
      else if (eventName === "ModeStateEvent") {
        store.setState(
          produce(store.getState(), (state) => {
            state.modeState = {
              name: event.name,
              data: modeStateEventDataSchema.parse(event.data),
              ts: event.ts,
            };
          }),
        );
      }
      // Metric events (keep for 1 hour)
      else if (eventName === "SineWaveEvent") {
        const parsed = sineWaveEventSchema.parse(event);
        const timeseriesValue: TimeSeriesValue = {
          value: parsed.data.amplitude,
          timestamp: event.ts,
        };
        store.setState(
          produce(store.getState(), (state) => {
            state.sineWave = addSineWave(state.sineWave, timeseriesValue);
          }),
        );
      } else {
        handleUnhandledEventError(eventName);
      }
    } catch (error) {
      if (error instanceof z.ZodError) {
        handleEventValidationError(error, eventName);
      } else {
        console.error(`Unexpected error processing ${eventName} event:`, error);
        throw error;
      }
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
