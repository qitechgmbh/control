/**
 * @file xtremZebraNamespace.ts
 * @description TypeScript implementation of XtremZebra namespace with Zod schema validation.
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
 * Live values from XtremZebra (30 FPS)
 */
export const liveValuesEventDataSchema = z.object({
  total_weight: z.number(),
  current_weight: z.number(),
  plate1_counter: z.number(),
  plate2_counter: z.number(),
  plate3_counter: z.number(),
});

/**
 * State event from XtremZebra (on state changes only)
 */
export const stateEventDataSchema = z.object({
  is_default_state: z.boolean(),
  xtrem_zebra_state: z.object({
    plate1_target: z.number(),
    plate2_target: z.number(),
    plate3_target: z.number(),
    tolerance: z.number(),
    target_quantity: z.number(),
  }),
  configuration: z.object({
    // <--- This new property is assumed
    config_string: z.string().nullable(),
    password: z.string().nullable(),
  }),
});

// ========== Event Schemas with Wrappers ==========
export const liveValuesEventSchema = eventSchema(liveValuesEventDataSchema);
export const stateEventSchema = eventSchema(stateEventDataSchema);

// ========== Type Inferences ==========
export type LiveValuesEvent = z.infer<typeof liveValuesEventSchema>;
export type StateEvent = z.infer<typeof stateEventSchema>;

export type XtremZebraNamespaceStore = {
  // Single state event from server
  state: StateEvent | null;
  defaultState: StateEvent | null;

  // Time series data for live values
  total_weight: TimeSeries;
  current_weight: TimeSeries;
  plate1_counter: TimeSeries;
  plate2_counter: TimeSeries;
  plate3_counter: TimeSeries;
};

// Constants for time durations
const TWENTY_MILLISECOND = 20;
const ONE_SECOND = 1000;
const FIVE_SECOND = 5 * ONE_SECOND;
const ONE_HOUR = 60 * 60 * ONE_SECOND;

const { initialTimeSeries: total_weight, insert: addTotalWeight } =
  createTimeSeries(TWENTY_MILLISECOND, ONE_SECOND, FIVE_SECOND, ONE_HOUR);

const { initialTimeSeries: current_weight, insert: addCurrentWeight } =
  createTimeSeries(TWENTY_MILLISECOND, ONE_SECOND, FIVE_SECOND, ONE_HOUR);

const { initialTimeSeries: plate1_counter, insert: addPlate1Counter } =
  createTimeSeries(TWENTY_MILLISECOND, ONE_SECOND, FIVE_SECOND, ONE_HOUR);

const { initialTimeSeries: plate2_counter, insert: addPlate2Counter } =
  createTimeSeries(TWENTY_MILLISECOND, ONE_SECOND, FIVE_SECOND, ONE_HOUR);

const { initialTimeSeries: plate3_counter, insert: addPlate3Counter } =
  createTimeSeries(TWENTY_MILLISECOND, ONE_SECOND, FIVE_SECOND, ONE_HOUR);

/**
 * Factory function to create a new XtremZebra namespace store
 * @returns A new Zustand store instance for XtremZebra namespace
 */
export const createXtremZebraNamespaceStore =
  (): StoreApi<XtremZebraNamespaceStore> =>
    create<XtremZebraNamespaceStore>(() => {
      return {
        state: null,
        defaultState: null,
        total_weight,
        current_weight,
        plate1_counter,
        plate2_counter,
        plate3_counter,
      };
    });

/**
 * Creates a message handler for XtremZebra namespace events with validation and appropriate caching strategies
 * @param store The store to update when messages are received
 * @param throttledUpdater Throttled updater for batching updates at 30 FPS
 * @returns A message handler function
 */
export function xtremZebraMessageHandler(
  store: StoreApi<XtremZebraNamespaceStore>,
  throttledUpdater: ThrottledStoreUpdater<XtremZebraNamespaceStore>,
): EventHandler {
  return (event: Event<any>) => {
    const eventName = event.name;

    // Helper function to update store through buffer
    const updateStore = (
      updater: (state: XtremZebraNamespaceStore) => XtremZebraNamespaceStore,
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
          // only set default state if is_default_state is true
          defaultState: stateEvent.data.is_default_state
            ? stateEvent
            : state.defaultState,
        }));
      }
      // Live values events (keep for 1 hour)
      else if (eventName === "LiveValuesEvent") {
        const liveValuesEvent = liveValuesEventSchema.parse(event);
        const totalWeightValue: TimeSeriesValue = {
          value: liveValuesEvent.data.total_weight,
          timestamp: event.ts,
        };
        const currentWeightValue: TimeSeriesValue = {
          value: liveValuesEvent.data.current_weight,
          timestamp: event.ts,
        };
        const plate1CounterValue: TimeSeriesValue = {
          value: liveValuesEvent.data.plate1_counter,
          timestamp: event.ts,
        };
        const plate2CounterValue: TimeSeriesValue = {
          value: liveValuesEvent.data.plate2_counter,
          timestamp: event.ts,
        };
        const plate3CounterValue: TimeSeriesValue = {
          value: liveValuesEvent.data.plate3_counter,
          timestamp: event.ts,
        };
        updateStore((state) => ({
          ...state,
          total_weight: addTotalWeight(state.total_weight, totalWeightValue),
          current_weight: addCurrentWeight(
            state.current_weight,
            currentWeightValue,
          ),
          plate1_counter: addPlate1Counter(
            state.plate1_counter,
            plate1CounterValue,
          ),
          plate2_counter: addPlate2Counter(
            state.plate2_counter,
            plate2CounterValue,
          ),
          plate3_counter: addPlate3Counter(
            state.plate3_counter,
            plate3CounterValue,
          ),
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
 * Create the XtremZebra namespace implementation
 */
const useXtremZebraNamespaceImplementation =
  createNamespaceHookImplementation<XtremZebraNamespaceStore>({
    createStore: createXtremZebraNamespaceStore,
    createEventHandler: xtremZebraMessageHandler,
  });

export function useXtremZebraNamespace(
  machine_identification_unique: MachineIdentificationUnique,
): XtremZebraNamespaceStore {
  // Generate namespace ID from validated machine ID
  const namespaceId: NamespaceId = {
    type: "machine",
    machine_identification_unique,
  };

  // Use the implementation with validated namespace ID
  return useXtremZebraNamespaceImplementation(namespaceId);
}
