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
} from "@/client/socketioStore";
import { MachineIdentificationUnique } from "@/machines/types";
import { createTimeSeries, TimeSeries, TimeSeriesValue } from "@/lib/timeseries";
import { randomBytes } from "crypto";

// ========== Event Schema ==========

/**
 * Machine operation mode enum
 */
export const modeSchema = z.enum(["Idle", "On", "Auto", "Interval"]);
export type Mode = z.infer<typeof modeSchema>;

export const stateEventDataSchema = z.object({
  mode: modeSchema,

  interval_time_off: z.number(),
  interval_time_on: z.number(),

  running: z.boolean(),
});

export const liveValuesEventDataSchema = z.object({
  remaining_time: z.number(),
});

export const stateEventSchema = eventSchema(stateEventDataSchema);
export const liveValuesEventSchema = eventSchema(liveValuesEventDataSchema);

export type StateEvent = z.infer<typeof stateEventDataSchema>;
export type LiveValuesEvent = z.infer<typeof liveValuesEventDataSchema>;

// ========== Store ==========
export type VacuumNamespaceStore = {
  state: StateEvent | null;
  liveValues: LiveValuesEvent | null;

  // Timer series data for live values
  remaining_time: TimeSeries,

  spin_shitter: TimeSeries,
};

const { initialTimeSeries: remaining_time, insert: addRemainingTime } =
  createTimeSeries({ sampleIntervalLong: 1000 });

const { initialTimeSeries: spin_shitter, insert: addSpinShitter } =
  createTimeSeries({ sampleIntervalLong: 1000 });

export const createVacuumNamespaceStore =
  (): StoreApi<VacuumNamespaceStore> =>
    create<VacuumNamespaceStore>(() => ({
      state: null,
      liveValues: null,
      remaining_time,
      spin_shitter,
    }));

// ========== Message Handler ==========
export function vacuumTestMachineMessageHandler(
  store: StoreApi<VacuumNamespaceStore>,
  throttledUpdater: ThrottledStoreUpdater<VacuumNamespaceStore>,
): EventHandler {
  return (event: Event<any>) => {
    const updateStore = (
      updater: (
        state: VacuumNamespaceStore,
      ) => VacuumNamespaceStore,
    ) => throttledUpdater.updateWith(updater);

    try {
      if (event.name === "StateEvent") 
      {
        console.log("STATE EVENT");

        const stateEvent = stateEventSchema.parse(event);

        const rpm = Math.floor(Math.random() * (125 - 115 + 1)) + 115;

          const value: TimeSeriesValue = {
            value: stateEvent.data.running ? rpm : 0,
            timestamp: event.ts,
          };

        updateStore((state) => ({ 
          ...state, 
          state: stateEvent.data,
          spin_shitter: addSpinShitter(state.spin_shitter, value)
        }));
      } 
      
      else if (event.name === "LiveValuesEvent") 
      {
        console.log("LIVE VALUES EVENT");

        const rpm = Math.floor(Math.random() * (125 - 115 + 1)) + 115;

        const parsed = liveValuesEventSchema.parse(event);


          const rpm_value: TimeSeriesValue = {
            value: rpm,
            timestamp: event.ts,
          };


          const rpm_value_0: TimeSeriesValue = {
            value: 0,
            timestamp: event.ts,
          };

        updateStore((liveValuesEvent) => ({ 
          ...liveValuesEvent, 
          liveValues: parsed.data,          
          spin_shitter: addSpinShitter(liveValuesEvent.spin_shitter, liveValuesEvent.state?.running ? rpm_value : rpm_value_0),
        }));

        if (parsed.data.remaining_time !== null) 
        {
          const value: TimeSeriesValue = {
            value: parsed.data.remaining_time,
            timestamp: event.ts,
          };
          updateStore((state) => ({
            ...state,
            remaining_time: addRemainingTime(state.remaining_time, value),
            
          }));
        }
      } 
      
      else {
        handleUnhandledEventError(event.name);
      }
    } catch (error) {
      console.error(`Error processing ${event.name}:`, error);
      throw error;
    }
  };
}

// ========== Namespace Hook ==========
const useVacuumNamespaceImplementation =
  createNamespaceHookImplementation<VacuumNamespaceStore>({
    createStore: createVacuumNamespaceStore,
    createEventHandler: vacuumTestMachineMessageHandler,
  });

export function useVacuumNamespace(
  machine_identification_unique: MachineIdentificationUnique,
): VacuumNamespaceStore {
  const namespaceId: NamespaceId = {
    type: "machine",
    machine_identification_unique,
  };

  return useVacuumNamespaceImplementation(namespaceId);
}
