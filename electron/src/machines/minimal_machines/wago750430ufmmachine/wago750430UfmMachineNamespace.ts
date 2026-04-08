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

export const stateEventDataSchema = z.object({
  inputs: z.array(z.boolean()).length(8),
});

export const stateEventSchema = eventSchema(stateEventDataSchema);
export type StateEvent = z.infer<typeof stateEventDataSchema>;

export type Wago750430UfmMachineNamespaceStore = {
  state: StateEvent | null;
};

export const createWago750430UfmMachineNamespaceStore =
  (): StoreApi<Wago750430UfmMachineNamespaceStore> =>
    create<Wago750430UfmMachineNamespaceStore>(() => ({
      state: null,
    }));

export function wago750430UfmMachineMessageHandler(
  store: StoreApi<Wago750430UfmMachineNamespaceStore>,
  throttledUpdater: ThrottledStoreUpdater<Wago750430UfmMachineNamespaceStore>,
): EventHandler {
  return (event: Event<any>) => {
    const updateStore = (
      updater: (
        state: Wago750430UfmMachineNamespaceStore,
      ) => Wago750430UfmMachineNamespaceStore,
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

const useWago750430UfmMachineNamespaceImplementation =
  createNamespaceHookImplementation<Wago750430UfmMachineNamespaceStore>({
    createStore: createWago750430UfmMachineNamespaceStore,
    createEventHandler: wago750430UfmMachineMessageHandler,
  });

export function useWago750430UfmMachineNamespace(
  machine_identification_unique: MachineIdentificationUnique,
): Wago750430UfmMachineNamespaceStore {
  const namespaceId: NamespaceId = {
    type: "machine",
    machine_identification_unique,
  };

  return useWago750430UfmMachineNamespaceImplementation(namespaceId);
}
