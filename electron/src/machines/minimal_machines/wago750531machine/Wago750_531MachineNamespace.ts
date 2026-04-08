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
  outputs_on: z.array(z.boolean()).length(4),
});

export const stateEventSchema = eventSchema(stateEventDataSchema);

export type StateEvent = z.infer<typeof stateEventDataSchema>;

// ========== Store ==========
export type Wago750_531MachineNamespaceStore = {
  state: StateEvent | null;
};

export const createWago750_531MachineNamespaceStore =
  (): StoreApi<Wago750_531MachineNamespaceStore> =>
    create<Wago750_531MachineNamespaceStore>(() => ({
      state: null,
    }));

// ========== Message Handler ==========
export function wago750_531MachineMessageHandler(
  store: StoreApi<Wago750_531MachineNamespaceStore>,
  throttledUpdater: ThrottledStoreUpdater<Wago750_531MachineNamespaceStore>,
): EventHandler {
  return (event: Event<any>) => {
    const updateStore = (
      updater: (
        state: Wago750_531MachineNamespaceStore,
      ) => Wago750_531MachineNamespaceStore,
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
const useWago750_531MachineNamespaceImplementation =
  createNamespaceHookImplementation<Wago750_531MachineNamespaceStore>({
    createStore: createWago750_531MachineNamespaceStore,
    createEventHandler: wago750_531MachineMessageHandler,
  });

export function useWago750_531MachineNamespace(
  machine_identification_unique: MachineIdentificationUnique,
): Wago750_531MachineNamespaceStore {
  const namespaceId: NamespaceId = {
    type: "machine",
    machine_identification_unique,
  };

  return useWago750_531MachineNamespaceImplementation(namespaceId);
}
