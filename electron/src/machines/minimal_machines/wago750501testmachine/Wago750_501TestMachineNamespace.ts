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
  outputs: z.array(z.boolean()).length(2),
});

export const stateEventSchema = eventSchema(stateEventDataSchema);

export type StateEvent = z.infer<typeof stateEventDataSchema>;

// ========== Store ==========
export type Wago750_501TestMachineNamespaceStore = {
  state: StateEvent | null;
};

export const createWago750_501TestMachineNamespaceStore =
  (): StoreApi<Wago750_501TestMachineNamespaceStore> =>
    create<Wago750_501TestMachineNamespaceStore>(() => ({
      state: null,
    }));

// ========== Message Handler ==========
export function wago750_501TestMachineMessageHandler(
  store: StoreApi<Wago750_501TestMachineNamespaceStore>,
  throttledUpdater: ThrottledStoreUpdater<Wago750_501TestMachineNamespaceStore>,
): EventHandler {
  return (event: Event<any>) => {
    const updateStore = (
      updater: (
        state: Wago750_501TestMachineNamespaceStore,
      ) => Wago750_501TestMachineNamespaceStore,
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
const useWago750_501TestMachineNamespaceImplementation =
  createNamespaceHookImplementation<Wago750_501TestMachineNamespaceStore>({
    createStore: createWago750_501TestMachineNamespaceStore,
    createEventHandler: wago750_501TestMachineMessageHandler,
  });

export function useWago750_501TestMachineNamespace(
  machine_identification_unique: MachineIdentificationUnique,
): Wago750_501TestMachineNamespaceStore {
  const namespaceId: NamespaceId = {
    type: "machine",
    machine_identification_unique,
  };

  return useWago750_501TestMachineNamespaceImplementation(namespaceId);
}
