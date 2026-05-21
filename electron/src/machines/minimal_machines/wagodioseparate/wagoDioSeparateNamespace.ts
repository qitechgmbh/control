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

// ========== Event Schema ==========
export const stateEventDataSchema = z.object({
  inputs: z.array(z.boolean()).length(8), // from 750-430
  led_on: z.array(z.boolean()).length(8), // from 750-530
});

export const stateEventSchema = eventSchema(stateEventDataSchema);
export type StateEvent = z.infer<typeof stateEventDataSchema>;

// ========== Store ==========
export type WagoDioSeparateNamespaceStore = {
  state: StateEvent | null;
};

export const createWagoDioSeparateNamespaceStore =
  (): StoreApi<WagoDioSeparateNamespaceStore> =>
    create<WagoDioSeparateNamespaceStore>(() => ({
      state: null,
    }));

// ========== Message Handler ==========
export function wagoDioSeparateMessageHandler(
  store: StoreApi<WagoDioSeparateNamespaceStore>,
  throttledUpdater: ThrottledStoreUpdater<WagoDioSeparateNamespaceStore>,
): EventHandler {
  return (event: Event<any>) => {
    const updateStore = (
      updater: (
        state: WagoDioSeparateNamespaceStore,
      ) => WagoDioSeparateNamespaceStore,
    ) => throttledUpdater.updateWith(updater);
    try {
      if (event.name === "StateEvent") {
        const parsed = stateEventSchema.parse(event);
        updateStore(() => ({ state: parsed.data }));
      } else {
        handleUnhandledEventError(event.name);
      }
    } catch (error) {
      console.error(`Error processing ${event.name}:`, error);
      throw error;
    }
  };
}

// ========== Namespace Hook ==========
const useWagoDioSeparateNamespaceImplementation =
  createNamespaceHookImplementation<WagoDioSeparateNamespaceStore>({
    createStore: createWagoDioSeparateNamespaceStore,
    createEventHandler: wagoDioSeparateMessageHandler,
  });

export function useWagoDioSeparateNamespace(
  machine_identification_unique: MachineIdentificationUnique,
): WagoDioSeparateNamespaceStore {
  const namespaceId: NamespaceId = {
    type: "machine",
    machine_identification_unique,
  };
  return useWagoDioSeparateNamespaceImplementation(namespaceId);
}
