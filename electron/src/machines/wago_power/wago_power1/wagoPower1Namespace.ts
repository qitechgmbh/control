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
import { useMemo } from "react";
import {
  createTimeSeries,
  TimeSeries,
  TimeSeriesValue,
} from "@/lib/timeseries";

export const liveValuesEventSchema = eventSchema(
  z.object({
    voltage: z.number(),
    current: z.number(),
  }),
);

export type LiveValuesEvent = z.infer<typeof liveValuesEventSchema>;

export const modeSchema = z.enum(["Off", "On24V"]);
export type Mode = z.infer<typeof modeSchema>;

export const stateEventSchema = eventSchema(
  z.object({
    mode: modeSchema,
    is_default_state: z.boolean(),
  }),
);

export type StateEvent = z.infer<typeof stateEventSchema>;

export type WagoPower1NamespaceStore = {
  state: StateEvent | null;
  defaultState: StateEvent | null;

  current: TimeSeries;
  voltage: TimeSeries;
};

const { initialTimeSeries: voltage, insert: addVoltage } = createTimeSeries();
const { initialTimeSeries: current, insert: addCurrent } = createTimeSeries();

const createWagoPower1NamespaceStore = (): StoreApi<WagoPower1NamespaceStore> =>
  create<WagoPower1NamespaceStore>(() => {
    return {
      state: null,
      defaultState: null,
      voltage,
      current,
    };
  });

function wagoPower1MessageHandler(
  store: StoreApi<WagoPower1NamespaceStore>,
  throttledUpdater: ThrottledStoreUpdater<WagoPower1NamespaceStore>,
): EventHandler {
  return (event: Event<any>) => {
    const eventName = event.name;

    // Helper function to update store through buffer
    const updateStore = (
      updater: (state: WagoPower1NamespaceStore) => WagoPower1NamespaceStore,
    ) => {
      throttledUpdater.updateWith(updater);
    };

    try {
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
      } else if (eventName === "LiveValuesEvent") {
        console.log(event);
        const liveValues = liveValuesEventSchema.parse(event);
        const { voltage, current } = liveValues.data;
        const voltageValue: TimeSeriesValue = {
          value: voltage,
          timestamp: liveValues.ts,
        };
        const currentValue: TimeSeriesValue = {
          value: current,
          timestamp: liveValues.ts,
        };
        updateStore((store: WagoPower1NamespaceStore) => ({
          ...store,
          voltage: addVoltage(store.voltage, voltageValue),
          current: addCurrent(store.current, currentValue),
        }));
      } else {
        handleUnhandledEventError(eventName);
      }
    } catch (e) {
      console.error(e);
    }
  };
}

const useWagoPower1NamespaceImplementation =
  createNamespaceHookImplementation<WagoPower1NamespaceStore>({
    createStore: createWagoPower1NamespaceStore,
    createEventHandler: wagoPower1MessageHandler,
  });

export function useWagoPower1Namespace(
  machine_identification_unique: MachineIdentificationUnique,
): WagoPower1NamespaceStore {
  // Generate namespace ID from validated machine ID
  const namespaceId = useMemo<NamespaceId>(
    () => ({
      type: "machine",
      machine_identification_unique,
    }),
    [machine_identification_unique],
  );

  // Use the implementation with validated namespace ID
  return useWagoPower1NamespaceImplementation(namespaceId);
}
