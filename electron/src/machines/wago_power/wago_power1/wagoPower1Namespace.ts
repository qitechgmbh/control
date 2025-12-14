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

export const stateEventDataSchema = z.object({
  voltage_milli_volt: z.number(),
  current_milli_ampere: z.number(),
});

export const stateEventSchema = eventSchema(stateEventDataSchema);

export type StateEvent = z.infer<typeof stateEventSchema>;

export type WagoPower1NamespaceStore = {
  state: StateEvent | null;
};

const createWagoPower1NamespaceStore =
  (): StoreApi<WagoPower1NamespaceStore> =>
    create<WagoPower1NamespaceStore>(() => {
      return {
        state: null,
      };
    });

function wagoPower1MessageHandler(
  store: StoreApi<WagoPower1NamespaceStore>,
  throttledUpdater: ThrottledStoreUpdater<WagoPower1NamespaceStore>,
): EventHandler {
  return (event: Event<any>) => {
    const eventName = event.name;

    // Helper function to update store through buffer
    // const updateStore = (
    //   updater: (state: WagoPower1NamespaceStore) => WagoPower1NamespaceStore,
    // ) => {
    //   throttledUpdater.updateWith(updater);
    // };

    try {
      if (eventName === "StateEvent") {
        console.log({ state: event });
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
