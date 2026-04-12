import {
  Event,
  EventHandler,
  NamespaceId,
  ThrottledStoreUpdater,
  createNamespaceHookImplementation,
  eventSchema,
  handleUnhandledEventError,
} from "@/client/socketioStore";
import { MachineIdentificationUnique } from "@/machines/types";
import { StoreApi, create } from "zustand";
import { z } from "zod";

export const stateEventDataSchema = z.object({
  voltages: z.array(z.number()).length(2),
  normalized: z.array(z.number()).length(2),
  raw_words: z.array(z.number()).length(2),
  wiring_errors: z.array(z.boolean()).length(2),
});

export const stateEventSchema = eventSchema(stateEventDataSchema);
export type StateEvent = z.infer<typeof stateEventDataSchema>;

export type Wago750467MachineNamespaceStore = {
  state: StateEvent | null;
};

export const createWago750467MachineNamespaceStore =
  (): StoreApi<Wago750467MachineNamespaceStore> =>
    create<Wago750467MachineNamespaceStore>(() => ({
      state: null,
    }));

export function wago750467MachineMessageHandler(
  store: StoreApi<Wago750467MachineNamespaceStore>,
  throttledUpdater: ThrottledStoreUpdater<Wago750467MachineNamespaceStore>,
): EventHandler {
  return (event: Event<any>) => {
    const updateStore = (
      updater: (
        state: Wago750467MachineNamespaceStore,
      ) => Wago750467MachineNamespaceStore,
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

const useWago750467MachineNamespaceImplementation =
  createNamespaceHookImplementation<Wago750467MachineNamespaceStore>({
    createStore: createWago750467MachineNamespaceStore,
    createEventHandler: wago750467MachineMessageHandler,
  });

export function useWago750467MachineNamespace(
  machine_identification_unique: MachineIdentificationUnique,
): Wago750467MachineNamespaceStore {
  const namespaceId: NamespaceId = {
    type: "machine",
    machine_identification_unique,
  };

  return useWago750467MachineNamespaceImplementation(namespaceId);
}
