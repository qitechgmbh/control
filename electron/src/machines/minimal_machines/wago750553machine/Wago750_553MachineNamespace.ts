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
  outputs: z.array(z.number()).length(4),
  outputs_ma: z.array(z.number()).length(4),
});

export const stateEventSchema = eventSchema(stateEventDataSchema);

export type StateEvent = z.infer<typeof stateEventDataSchema>;

// ========== Store ==========

export type Wago750_553MachineNamespaceStore = {
  state: StateEvent | null;
};

export const createWago750_553MachineNamespaceStore =
  (): StoreApi<Wago750_553MachineNamespaceStore> =>
    create<Wago750_553MachineNamespaceStore>(() => ({
      state: null,
    }));

// ========== Message Handler ==========

export function wago750_553MachineMessageHandler(
  store: StoreApi<Wago750_553MachineNamespaceStore>,
  throttledUpdater: ThrottledStoreUpdater<Wago750_553MachineNamespaceStore>,
): EventHandler {
  return (event: Event<any>) => {
    const updateStore = (
      updater: (
        state: Wago750_553MachineNamespaceStore,
      ) => Wago750_553MachineNamespaceStore,
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

const useWago750_553MachineNamespaceImplementation =
  createNamespaceHookImplementation<Wago750_553MachineNamespaceStore>({
    createStore: createWago750_553MachineNamespaceStore,
    createEventHandler: wago750_553MachineMessageHandler,
  });

export function useWago750_553MachineNamespace(
  machine_identification_unique: MachineIdentificationUnique,
): Wago750_553MachineNamespaceStore {
  const namespaceId: NamespaceId = {
    type: "machine",
    machine_identification_unique,
  };

  return useWago750_553MachineNamespaceImplementation(namespaceId);
}
