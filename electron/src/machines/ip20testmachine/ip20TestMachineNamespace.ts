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
  outputs: z.array(z.boolean()).length(8),
});

export const liveValuesEventDataSchema = z.object({
  inputs: z.array(z.boolean()).length(8),
});

export const stateEventSchema = eventSchema(stateEventDataSchema);
export const liveValuesEventSchema = eventSchema(liveValuesEventDataSchema);

export type StateEvent = z.infer<typeof stateEventDataSchema>;
export type LiveValuesEvent = z.infer<typeof liveValuesEventDataSchema>;

// ========== Store ==========
export type IP20TestMachineNamespaceStore = {
  state: StateEvent | null;
  liveValues: LiveValuesEvent | null;
};

export const createIP20TestMachineNamespaceStore =
  (): StoreApi<IP20TestMachineNamespaceStore> =>
    create<IP20TestMachineNamespaceStore>(() => ({
      state: null,
      liveValues: null,
    }));

// ========== Message Handler ==========
export function ip20TestMachineMessageHandler(
  store: StoreApi<IP20TestMachineNamespaceStore>,
  throttledUpdater: ThrottledStoreUpdater<IP20TestMachineNamespaceStore>,
): EventHandler {
  return (event: Event<any>) => {
    const updateStore = (
      updater: (
        state: IP20TestMachineNamespaceStore,
      ) => IP20TestMachineNamespaceStore,
    ) => throttledUpdater.updateWith(updater);

    try {
      if (event.name === "StateEvent") {
        const parsed = stateEventSchema.parse(event);
        updateStore((current) => ({ ...current, state: parsed.data }));
      } else if (event.name === "LiveValuesEvent") {
        const parsed = liveValuesEventSchema.parse(event);
        updateStore((current) => ({ ...current, liveValues: parsed.data }));
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
const useIP20TestMachineNamespaceImplementation =
  createNamespaceHookImplementation<IP20TestMachineNamespaceStore>({
    createStore: createIP20TestMachineNamespaceStore,
    createEventHandler: ip20TestMachineMessageHandler,
  });

export function useIP20TestMachineNamespace(
  machine_identification_unique: MachineIdentificationUnique,
): IP20TestMachineNamespaceStore {
  const namespaceId: NamespaceId = {
    type: "machine",
    machine_identification_unique,
  };

  return useIP20TestMachineNamespaceImplementation(namespaceId);
}
