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
} from "../../client/socketioStore";
import { MachineIdentificationUnique } from "@/machines/types";
import { useMemo } from "react";
import {
  createTimeSeries,
  TimeSeries,
  TimeSeriesValue,
} from "@/lib/timeseries";

export const stateEventSchema = eventSchema(
  z.object({
    current_message: z.string().nullable(),
  }),
);

export type StateEvent = z.infer<typeof stateEventSchema>;

export type WagoSerialNamespaceStore = {
  state: StateEvent | null;
  defaultState: StateEvent | null;
};

const createWagoSerialNamespaceStore = (): StoreApi<WagoSerialNamespaceStore> =>
  create<WagoSerialNamespaceStore>(() => {
    return {
      state: null,
      defaultState: null,
    };
  });

function wagoSerialMessageHandler(
  store: StoreApi<WagoPower1NamespaceStore>,
  throttledUpdater: ThrottledStoreUpdater<WagoSerialNamespaceStore>,
): EventHandler {
  return (event: Event<any>) => {
    const eventName = event.name;

    // Helper function to update store through buffer
    const updateStore = (
      updater: (state: WagoSerialNamespaceStore) => WagoSerialNamespaceStore,
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
      } else {
        handleUnhandledEventError(eventName);
      }
    } catch (e) {
      console.error(e);
    }
  };
}

const useWagoSerialNamespaceImplementation =
  createNamespaceHookImplementation<WagoSerialNamespaceStore>({
    createStore: createWagoSerialNamespaceStore,
    createEventHandler: wagoSerialMessageHandler,
  });

export function useWagoSerialNamespace(
  machine_identification_unique: MachineIdentificationUnique,
): WagoSerialNamespaceStore {
  // Generate namespace ID from validated machine ID
  const namespaceId = useMemo<NamespaceId>(
    () => ({
      type: "machine",
      machine_identification_unique,
    }),
    [machine_identification_unique],
  );

  // Use the implementation with validated namespace ID
  return useWagoSerialNamespaceImplementation(namespaceId);
}
