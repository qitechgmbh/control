/**
 * @file pellet1Namespace.ts
 * @description TypeScript implementation of Pellet1 namespace with Zod schema validation.
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
 * Live values from Pellet (30 FPS)
 */
export const liveValuesEventDataSchema = z.object({
    inverter_values: z.object({
        frequency:   z.number(),
        temperature: z.number(),
        voltage:     z.number(),
        current:     z.number(),
    })
});

/**
 * State event from Pellet (on state changes only)
 */
export const stateEventDataSchema = z.object({
  is_default_state: z.boolean(),

    inverter_state: z.object({
    running_state: z.number(),
    frequency_target: z.number(),
    acceleration_level: z.number(),
    deceleration_level: z.number(),

    error_code: z.number().nullable(),
    system_status: z.number(),
  }),
});

// ========== Event Schemas with Wrappers ==========
export const liveValuesEventSchema = eventSchema(liveValuesEventDataSchema);
export const stateEventSchema      = eventSchema(stateEventDataSchema);

// ========== Type Inferences ==========
export type LiveValuesEvent = z.infer<typeof liveValuesEventSchema>;
export type StateEvent      = z.infer<typeof stateEventSchema>;

export type Pellet1NamespaceStore = {
    
    state: StateEvent | null;
    defaultState: StateEvent | null;

    // Time series data for live values
    frequency: TimeSeries;
    temperature: TimeSeries;
    voltage: TimeSeries;
    current: TimeSeries;
};

// Constants for time durations
const HALF_SECOND = 500;

const { initialTimeSeries: frequency, insert: addFrequency } = 
    createTimeSeries({ sampleIntervalLong: HALF_SECOND });
    
const { initialTimeSeries: temperature, insert: addTemperature } =
    createTimeSeries({ sampleIntervalLong: HALF_SECOND });
    
const { initialTimeSeries: voltage, insert: addVoltage } = 
    createTimeSeries({ sampleIntervalLong: HALF_SECOND });
    
const { initialTimeSeries: current, insert: addCurrent } = 
    createTimeSeries({ sampleIntervalLong: HALF_SECOND });

/**
 * Factory function to create a new Pellet1 namespace store
 * @returns A new Zustand store instance for Pellet1 namespace
 */
export const createPellet1NamespaceStore =
    (): StoreApi<Pellet1NamespaceStore> =>
        create<Pellet1NamespaceStore>(() => {
            return {
              state: null,
              defaultState: null,
              frequency,
              temperature,
              voltage,
              current,
            };
        });

/**
 * Creates a message handler for Pellet1 namespace events with validation and appropriate caching strategies
 * @param store The store to update when messages are received
 * @param throttledUpdater Throttled updater for batching updates at 30 FPS
 * @returns A message handler function
 */
export function pellet1MessageHandler(
    store: StoreApi<Pellet1NamespaceStore>,
    throttledUpdater: ThrottledStoreUpdater<Pellet1NamespaceStore>,
): EventHandler {
    return (event: Event<any>) => 
    {
        const eventName = event.name;

        // Helper function to update store through buffer
        const updateStore = (updater: (state: Pellet1NamespaceStore) => Pellet1NamespaceStore) => 
        {
            throttledUpdater.updateWith(updater);
        };

        try 
        {
            // Apply appropriate caching strategy based on event type
            if (eventName === "StateEvent") 
            {
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
            else if (eventName === "LiveValuesEvent") 
            {
                const liveValuesEvent = liveValuesEventSchema.parse(event);

                console.log(liveValuesEvent)

                const frequencyValue: TimeSeriesValue = {
                  value: liveValuesEvent.data.inverter_values.frequency,
                  timestamp: event.ts,
                };
                
                updateStore((state) => ({ ...state, frequency: addFrequency(state.frequency, frequencyValue) }));

                const temperatureValue: TimeSeriesValue = {
                  value: liveValuesEvent.data.inverter_values.temperature,
                  timestamp: event.ts,
                };

                updateStore((state) => ({
                  ...state,
                  temperature: addTemperature(state.temperature, temperatureValue),
                }));

                const voltageValue: TimeSeriesValue = {
                  value: liveValuesEvent.data.inverter_values.voltage,
                  timestamp: event.ts,
                };

                updateStore((state) => ({
                  ...state,
                  voltage: addVoltage(state.voltage, voltageValue),
                }));

                const currentValue: TimeSeriesValue = {
                  value: liveValuesEvent.data.inverter_values.current,
                  timestamp: event.ts,
                };

                updateStore((state) => ({
                  ...state,
                  current: addCurrent(state.current, currentValue),
                }));


            } else {
                handleUnhandledEventError(eventName);
            }
        } catch (error) {
            console.error(
                `Unexpected error processing ${eventName} event:`,
                error,
            );
            throw error;
        }
    };
}

/**
 * Create the Pellet1 namespace implementation
 */
const usePellet1NamespaceImplementation =
    createNamespaceHookImplementation<Pellet1NamespaceStore>({
        createStore: createPellet1NamespaceStore,
        createEventHandler: pellet1MessageHandler,
    });

export function usePellet1Namespace(
    machine_identification_unique: MachineIdentificationUnique,
): Pellet1NamespaceStore {
    // Generate namespace ID from validated machine ID
    const namespaceId: NamespaceId = {
        type: "machine",
        machine_identification_unique,
    };

    // Use the implementation with validated namespace ID
    return usePellet1NamespaceImplementation(namespaceId);
}
