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
import { createTimeSeries, TimeSeries } from "@/lib/timeseries";

// ========== Event Schema Definitions ==========
/**
 * Mode enum for Aquapath Machine
 */
export const modeSchema = z.enum(["Standby", "Auto"]);

/**
 * Mode state schema
 */
export const modeStateSchema = z.object({
  mode: modeSchema,
});
export const tempStateSchema = z.object({
  temperature: z.number(),
  target_temperature: z.number(),
});
/**
 * Cooling states schema
 */
export const tempStatesSchema = z.object({
  front: tempStateSchema,
  back: tempStateSchema,
});

export const flowStateSchema = z.object({
  flow: z.number(),
  should_flow: z.boolean(),
});
export const flowStatesSchema = z.object({
  front: flowStateSchema,
  back: flowStateSchema,
});

/**
 * Live values event schema (time-series data)
 */
export const liveValuesEventDataSchema = z.object({
  front_flow: z.number(),
  back_flow: z.number(),
  front_temperature: z.number(),
  back_temperature: z.number(),
  front_temp_reservoir: z.number(),
  back_temp_reservoir: z.number(),
});

/**
 * State event schema (consolidated state)
 */
export const stateEventDataSchema = z.object({
  is_default_state: z.boolean(),
  mode_state: modeStateSchema,
  flow_states: flowStatesSchema,
  temperature_states: tempStatesSchema,
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

  front_flow: TimeSeries;
  back_flow: TimeSeries;

  front_temperature: TimeSeries;
  back_temperature: TimeSeries;

  front_temp_reservoir: TimeSeries;
  back_temp_reservoir: TimeSeries;
};

const { initialTimeSeries: front_temperature, insert: addTemperature1 } =
  createTimeSeries();
const { initialTimeSeries: back_temperature, insert: addTemperature2 } =
  createTimeSeries();
const { initialTimeSeries: front_temp_reservoir, insert: addTempReserv1 } =
  createTimeSeries();
const { initialTimeSeries: back_temp_reservoir, insert: addTempReserv2 } =
  createTimeSeries();
const { initialTimeSeries: front_flow, insert: addFlow1 } = createTimeSeries();
const { initialTimeSeries: back_flow, insert: addFlow2 } = createTimeSeries();

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
        front_temperature: front_temperature,
        back_temperature: back_temperature,
        front_flow: front_flow,
        back_flow: back_flow,
        front_temp_reservoir: front_temp_reservoir,
        back_temp_reservoir: back_temp_reservoir,
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
          front_temperature: addTemperature1(state.front_temperature, {
            value: liveValuesEvent.data.front_temperature,
            timestamp: event.ts,
          }),
          back_temperature: addTemperature2(state.back_temperature, {
            value: liveValuesEvent.data.back_temperature,
            timestamp: event.ts,
          }),
          front_flow: addFlow1(state.front_flow, {
            value: liveValuesEvent.data.front_flow,
            timestamp: event.ts,
          }),
          back_flow: addFlow2(state.back_flow, {
            value: liveValuesEvent.data.back_flow,
            timestamp: event.ts,
          }),
          front_temp_reservoir: addTempReserv1(state.front_temp_reservoir, {
            value: liveValuesEvent.data.front_temp_reservoir,
            timestamp: event.ts,
          }),
          back_temp_reservoir: addTempReserv2(state.back_temp_reservoir, {
            value: liveValuesEvent.data.back_temp_reservoir,
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
