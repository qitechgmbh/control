// ========== Event Schema Definitions ==========

import {
  EventHandler,
  eventSchema,
  Event,
  handleUnhandledEventError,
  ThrottledStoreUpdater,
  createNamespaceHookImplementation,
  NamespaceId,
} from "@/client/socketioStore";
import {
  createTimeSeries,
  TimeSeries,
  TimeSeriesValue,
} from "@/lib/timeseries";
import z from "zod";
import { create, StoreApi } from "zustand";
import { MachineIdentificationUnique } from "../../types";
import { useMemo } from "react";

/**
 * Consolidated live values event schema (30FPS data)
 */
export const liveValuesEventDataSchema = z.object({
  fan_rpm: z.number(),
  water_temperature: z.number(),
  flow_rate: z.number(),
});

/**
 * Consolidated state event schema (state changes only)
 */
export const stateEventDataSchema = z.object({
  is_default_state: z.boolean(),
  is_fan_on: z.boolean(),
  target_temperature: z.number(),
});

// ========== Event Schemas with Wrappers ==========

export const liveValuesEventSchema = eventSchema(liveValuesEventDataSchema);
export const stateEventSchema = eventSchema(stateEventDataSchema);

// ========== Type Inferences ==========

export type StateEvent = z.infer<typeof stateEventSchema>;

// Individual type exports for backward compatibility

export type Aquapath1NamespaceStore = {
  // State event from server
  state: StateEvent | null;
  defaultState: StateEvent | null;

  // Time series data for live values
  fanRpm: TimeSeries;
  waterTemperature: TimeSeries;
  flowRate: TimeSeries;
};

// Constants for time durations
const TWENTY_MILLISECOND = 20;
const ONE_SECOND = 1000;
const FIVE_SECOND = 5 * ONE_SECOND;
const ONE_HOUR = 60 * 60 * ONE_SECOND;

const { initialTimeSeries: fanRpm, insert: addFanRpm } = createTimeSeries(
  TWENTY_MILLISECOND,
  ONE_SECOND,
  FIVE_SECOND,
  ONE_HOUR,
);

const { initialTimeSeries: waterTemperature, insert: addWaterTemperature } =
  createTimeSeries(TWENTY_MILLISECOND, ONE_SECOND, FIVE_SECOND, ONE_HOUR);

const { initialTimeSeries: flowRate, insert: addFlowRate } = createTimeSeries(
  TWENTY_MILLISECOND,
  ONE_SECOND,
  FIVE_SECOND,
  ONE_HOUR,
);

/**
 * Factory function to create a new Aquapath1 namespace store
 * @returns A new Zustand store instance for Aquapath1 namespace
 */

export const createAquapath1NamespaceStore =
  (): StoreApi<Aquapath1NamespaceStore> =>
    create<Aquapath1NamespaceStore>(() => {
      return {
        // State event from server
        state: null,
        defaultState: null,

        // Time series data for live values
        fanRpm,
        waterTemperature,
        flowRate,
      };
    });

/**
 * @file aquapath1Namespace.ts (continued)
 */

/**
 * Creates a message handler for Aquapath1 namespace events with validation and appropriate caching strategies
 * @param store The store to update when messages are received
 * @param throttledUpdater Throttled updater for batching updates at 30 FPS
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
      if (eventName === "StateEvent") {
        // Parse and validate the state event
        const stateEvent = stateEventSchema.parse(event);

        updateStore((state) => ({
          ...state,
          state: stateEvent,
          // only set default state if is_default_state is true
          defaultState: stateEvent.data.is_default_state
            ? stateEvent
            : state.defaultState,
        }));
      } else if (eventName === "LiveValuesEvent") {
        // Parse and validate the live values event
        const liveValuesEvent = liveValuesEventSchema.parse(event);

        // Extract values and add to time series
        const { fan_rpm, water_temperature, flow_rate } = liveValuesEvent.data;
        const timestamp = liveValuesEvent.ts;

        updateStore((state) => {
          const newState = { ...state };

          // Add fan RPM
          const fanRpmValue: TimeSeriesValue = {
            value: fan_rpm,
            timestamp,
          };

          // Add water temperature
          const waterTemperatureValue: TimeSeriesValue = {
            value: water_temperature,
            timestamp,
          };

          // Add flow rate
          const flowRateValue: TimeSeriesValue = {
            value: water_temperature,
            timestamp,
          };

          newState.fanRpm = addFanRpm(state.fanRpm, fanRpmValue);
          newState.waterTemperature = addWaterTemperature(
            state.waterTemperature,
            waterTemperatureValue,
          );
          newState.flowRate = addFlowRate(state.flowRate, flowRateValue);

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
  const namespaceId = useMemo<NamespaceId>(
    () => ({
      type: "machine",
      machine_identification_unique,
    }),
    [machine_identification_unique],
  );

  // Use the implementation with validated namespace ID
  return useAquapath1NamespaceImplementation(namespaceId);
}
