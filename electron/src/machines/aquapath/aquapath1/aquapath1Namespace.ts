/**
 * @file Aquapath1Namespace.ts
 * @description TypeScript implementation of Aquapath1 namespace with Zod schema validation.
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
 * Mode enum for Aquapath Machine
 */
export const modeSchema = z.enum(["Standby", "Cool", "Heat"]);

/**
 * Mode state schema
 */
export const modeStateSchema = z.object({
  mode: modeSchema,
});
export const coolingStateSchema = z.object({
  temperature: z.number(),
  target_temperature: z.number(),
  wiring_error: z.boolean(),
});
/**
 * Cooling states schema
 */
export const coolingStatesSchema = z.object({
  front: coolingStateSchema,
  back: coolingStateSchema,
});

// export const flowStateSchema = z.object({
//   flow: z.number(),
// });
/**
 * Live values event schema (time-series data)
 */
export const liveValuesEventDataSchema = z.object({
  front_temperature: z.number(),
  back_temperature: z.number(),

  flow_sensor1: z.number(),
  flow_sensor2: z.number(),
});

/**
 * State event schema (consolidated state)
 */
export const stateEventDataSchema = z.object({
  is_default_state: z.boolean(),
  mode_state: modeStateSchema,
  //flow_state: flowStateSchema,
  cooling_states: coolingStatesSchema,
});

// ========== Event Schemas with Wrappers ==========
export const liveValuesEventSchema = eventSchema(liveValuesEventDataSchema);
export const stateEventSchema = eventSchema(stateEventDataSchema);

// ========== Type Inferences ==========
export type Mode = z.infer<typeof modeSchema>;
export type ModeState = z.infer<typeof modeStateSchema>;
export type LiveValuesEvent = z.infer<typeof liveValuesEventSchema>;
export type StateEvent = z.infer<typeof stateEventSchema>;

export type Aquapath1NamespaceStore = {
  // Single state event from server
  state: StateEvent | null;
  defaultState: StateEvent | null;

  flow_sensor1: TimeSeries;
  flow_sensor2: TimeSeries;

  temperature_sensor1: TimeSeries;
  temperature_sensor2: TimeSeries;
};

// Constants for time durations
const TWENTY_MILLISECOND = 20;
const ONE_SECOND = 1000;
const FIVE_SECOND = 5 * ONE_SECOND;
const ONE_HOUR = 60 * 60 * ONE_SECOND;

const { initialTimeSeries: temperature_sensor1, insert: addTemperature1 } =
  createTimeSeries(TWENTY_MILLISECOND, ONE_SECOND, FIVE_SECOND, ONE_HOUR);

const { initialTimeSeries: temperature_sensor2, insert: addTemperature2 } =
  createTimeSeries(TWENTY_MILLISECOND, ONE_SECOND, FIVE_SECOND, ONE_HOUR);

const { initialTimeSeries: flow_sensor1, insert: addFlow1 } = createTimeSeries(
  TWENTY_MILLISECOND,
  ONE_SECOND,
  FIVE_SECOND,
  ONE_HOUR,
);

const { initialTimeSeries: flow_sensor2, insert: addFlow2 } = createTimeSeries(
  TWENTY_MILLISECOND,
  ONE_SECOND,
  FIVE_SECOND,
  ONE_HOUR,
);
/**
 * Factory function to create a new Aquapath namespace store
 * @returns A new Zustand store instance for Aquapath namespace
 */
export const createAquapath1NamespaceStore =
  (): StoreApi<Aquapath1NamespaceStore> => {
    return create<Aquapath1NamespaceStore>(() => {
      return {
        state: null,
        defaultState: null,
        temperature_sensor1: temperature_sensor1,
        temperature_sensor2: temperature_sensor2,
        flow_sensor1: flow_sensor1,
        flow_sensor2: flow_sensor2,
      };
    });
  };

/**
 * Creates a message handler for Mock1 namespace events with validation and appropriate caching strategies
 * @param store The store to update when messages are received
 * @param throttledUpdater Throttled updater for batching updates at 60 FPS
 * @returns A message handler function
 */
export function aquapath1MessageHandler(
  store: StoreApi<Aquapath1NamespaceStore>,
  throttledUpdater: ThrottledStoreUpdater<Aquapath1NamespaceStore>,
): EventHandler {
  return (event: Event<any>) => {
    const eventName = event.name;

    // Helper function to update store through buffer
    const updateStore = (
      updater: (state: Aquapath1NamespaceStore) => Aquapath1NamespaceStore,
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

        updateStore((state) => ({
          ...state,
          temperature_sensor1: addTemperature1(state.temperature_sensor1, {
            value: liveValuesEvent.data.front_temperature,
            timestamp: event.ts,
          }),
          temperature_sensor2: addTemperature2(state.temperature_sensor2, {
            value: liveValuesEvent.data.back_temperature,
            timestamp: event.ts,
          }),
          flow_sensor1: addFlow1(state.flow_sensor1, {
            value: liveValuesEvent.data.flow_sensor1,
            timestamp: event.ts,
          }),
          flow_sensor2: addFlow2(state.flow_sensor2, {
            value: liveValuesEvent.data.flow_sensor2,
            timestamp: event.ts,
          }),
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
 * Create the Aquapath1 namespace implementation
 */
const useAquapath1NamespaceImplementation =
  createNamespaceHookImplementation<Aquapath1NamespaceStore>({
    createStore: createAquapath1NamespaceStore,
    createEventHandler: aquapath1MessageHandler,
  });

export function useAquapath1Namespace(
  machine_identification_unique: MachineIdentificationUnique,
): Aquapath1NamespaceStore {
  // Generate namespace ID from validated machine ID
  const namespaceId: NamespaceId = {
    type: "machine",
    machine_identification_unique,
  };

  // Use the implementation with validated namespace ID
  return useAquapath1NamespaceImplementation(namespaceId);
}
