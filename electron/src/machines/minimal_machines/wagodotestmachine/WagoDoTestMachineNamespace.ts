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
  led_on: z.array(z.boolean()).length(8),
});

export const stateEventSchema = eventSchema(stateEventDataSchema);

export type StateEvent = z.infer<typeof stateEventDataSchema>;

// ========== Store ==========
export type WagoDoTestMachineNamespaceStore = {
  state: StateEvent | null;
};

export const createWagoDoTestMachineNamespaceStore =
  (): StoreApi<WagoDoTestMachineNamespaceStore> =>
    create<WagoDoTestMachineNamespaceStore>(() => ({
      state: null,
    }));

// ========== Message Handler ==========
export function wagoDoTestMachineMessageHandler(
  store: StoreApi<WagoDoTestMachineNamespaceStore>,
  throttledUpdater: ThrottledStoreUpdater<WagoDoTestMachineNamespaceStore>,
): EventHandler {
  return (event: Event<any>) => {
    const updateStore = (
      updater: (
        state: WagoDoTestMachineNamespaceStore,
      ) => WagoDoTestMachineNamespaceStore,
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
const useWagoDoTestMachineNamespaceImplementation =
  createNamespaceHookImplementation<WagoDoTestMachineNamespaceStore>({
    createStore: createWagoDoTestMachineNamespaceStore,
    createEventHandler: wagoDoTestMachineMessageHandler,
  });

export function useWagoDoTestMachineNamespace(
  machine_identification_unique: MachineIdentificationUnique,
): WagoDoTestMachineNamespaceStore {
  const namespaceId: NamespaceId = {
    type: "machine",
    machine_identification_unique,
  };

  return useWagoDoTestMachineNamespaceImplementation(namespaceId);
}
