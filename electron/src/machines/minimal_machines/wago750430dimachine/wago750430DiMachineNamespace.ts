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

export type Wago750430DiMachineNamespaceStore = {
  state: StateEvent | null;
};

export const createWago750430DiMachineNamespaceStore =
  (): StoreApi<Wago750430DiMachineNamespaceStore> =>
    create<Wago750430DiMachineNamespaceStore>(() => ({
      state: null,
    }));

export function wago750430DiMachineMessageHandler(
  store: StoreApi<Wago750430DiMachineNamespaceStore>,
  throttledUpdater: ThrottledStoreUpdater<Wago750430DiMachineNamespaceStore>,
): EventHandler {
  return (event: Event<any>) => {
    const updateStore = (
      updater: (
        state: Wago750430DiMachineNamespaceStore,
      ) => Wago750430DiMachineNamespaceStore,
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

const useWago750430DiMachineNamespaceImplementation =
  createNamespaceHookImplementation<Wago750430DiMachineNamespaceStore>({
    createStore: createWago750430DiMachineNamespaceStore,
    createEventHandler: wago750430DiMachineMessageHandler,
  });

export function useWago750430DiMachineNamespace(
  machine_identification_unique: MachineIdentificationUnique,
): Wago750430DiMachineNamespaceStore {
  const namespaceId: NamespaceId = {
    type: "machine",
    machine_identification_unique,
  };

  return useWago750430DiMachineNamespaceImplementation(namespaceId);
}
