/**
 * @file Namespace.ts
 * @description TypeScript implementation of  namespace with Zod schema validation.
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
 * Live values from  (30 FPS)
 */
export const liveValuesEventDataSchema = z.object({
  weight_peak: z.number().nullable(),
  weight_prev: z.number().nullable(),
});

export const targetRangeDataSchema = z.object({
  min: z.number(),
  max: z.number(),
  desirec: z.number(),
});

export const entryDataSchema = z.object({
  doc_entry:   z.number(),
  line_number: z.number(),
  item_code:   z.string(),
  whs_code:    z.string(),
  weight_bounds: targetRangeDataSchema
});

/**
 * State event from  (on state changes only)
 */
export const stateEventDataSchema = z.object({
  is_default_state: z.boolean(),
  plates_counted: z.number(),
  current_entry: entryDataSchema.nullable(),
});

// ========== Event Schemas with Wrappers ==========
export const liveValuesEventSchema = eventSchema(liveValuesEventDataSchema);
export const stateEventSchema = eventSchema(stateEventDataSchema);

// ========== Type Inferences ==========
export type LiveValuesEvent = z.infer<typeof liveValuesEventSchema>;
export type StateEvent = z.infer<typeof stateEventSchema>;

export type NamespaceStore = {
  // Single state event from server
  state: StateEvent | null;
  defaultState: StateEvent | null;

  // Time series data for live values
  weightPeak: TimeSeries;
  weightPrev: TimeSeries;
};

// time series
const { initialTimeSeries: weightPeak, insert: addWeightPeak } = createTimeSeries();
const { initialTimeSeries: weightPrev, insert: addWeightPrev } = createTimeSeries();

/**
 * Factory function to create a new  namespace store
 * @returns A new Zustand store instance for  namespace
 */
export const createNamespaceStore =
  (): StoreApi<NamespaceStore> =>
    create<NamespaceStore>(() => {
      return {
        state: null,
        defaultState: null,
        weightPeak,
        weightPrev,
      };
    });

/**
 * Creates a message handler for  namespace events with validation and appropriate caching strategies
 * @param store The store to update when messages are received
 * @param throttledUpdater Throttled updater for batching updates at 30 FPS
 * @returns A message handler function
 */
export function MessageHandler(
  store: StoreApi<NamespaceStore>,
  throttledUpdater: ThrottledStoreUpdater<NamespaceStore>,
): EventHandler {
  return (event: Event<any>) => {
    const eventName = event.name;

    // Helper function to update store through buffer
    const updateStore = (
      updater: (state: NamespaceStore) => NamespaceStore,
    ) => {
      throttledUpdater.updateWith(updater);
    };

    try {
      // Apply appropriate caching strategy based on event type
      if (eventName === "State") {
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
      // Live values events (keep for 1 hour)
      else if (eventName === "LiveValues") {
        const liveValuesEvent = liveValuesEventSchema.parse(event);
        const weightPeakValue: TimeSeriesValue = {
          value: liveValuesEvent.data.weight_peak ?? 0.0,
          timestamp: event.ts,
        };
        const weightPrevValue: TimeSeriesValue = {
          value: liveValuesEvent.data.weight_prev ?? 0.0,
          timestamp: event.ts,
        };
        updateStore((state) => ({
          ...state,
          weightPeak: addWeightPeak(state.weightPeak, weightPeakValue),
          weightPrev: addWeightPrev(state.weightPrev, weightPrevValue),
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
 * Create the  namespace implementation
 */
const useNamespaceImplementation =
  createNamespaceHookImplementation<NamespaceStore>({
    createStore: createNamespaceStore,
    createEventHandler: MessageHandler,
  });

export function useNamespace(
  machine_identification_unique: MachineIdentificationUnique,
): NamespaceStore {
  // Generate namespace ID from validated machine ID
  const namespaceId: NamespaceId = {
    type: "machine",
    machine_identification_unique,
  };

  // Use the implementation with validated namespace ID
  return useNamespaceImplementation(namespaceId);
}
