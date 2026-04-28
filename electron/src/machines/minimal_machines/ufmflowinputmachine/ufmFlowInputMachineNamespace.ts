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
  flow_lph: z.number(),
  total_volume_m3: z.number(),
  error: z.boolean(),
  total_pulses: z.number(),
});

export const stateEventSchema = eventSchema(stateEventDataSchema);

export type StateEvent = z.infer<typeof stateEventDataSchema>;

export type UfmFlowInputMachineNamespaceStore = {
  state: StateEvent | null;
};

export const createUfmFlowInputMachineNamespaceStore =
  (): StoreApi<UfmFlowInputMachineNamespaceStore> =>
    create<UfmFlowInputMachineNamespaceStore>(() => ({
      state: null,
    }));

export function ufmFlowInputMachineMessageHandler(
  store: StoreApi<UfmFlowInputMachineNamespaceStore>,
  throttledUpdater: ThrottledStoreUpdater<UfmFlowInputMachineNamespaceStore>,
): EventHandler {
  return (event: Event<any>) => {
    const updateStore = (
      updater: (
        state: UfmFlowInputMachineNamespaceStore,
      ) => UfmFlowInputMachineNamespaceStore,
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

const useUfmFlowInputMachineNamespaceImplementation =
  createNamespaceHookImplementation<UfmFlowInputMachineNamespaceStore>({
    createStore: createUfmFlowInputMachineNamespaceStore,
    createEventHandler: ufmFlowInputMachineMessageHandler,
  });

export function useUfmFlowInputMachineNamespace(
  machine_identification_unique: MachineIdentificationUnique,
): UfmFlowInputMachineNamespaceStore {
  const namespaceId: NamespaceId = {
    type: "machine",
    machine_identification_unique,
  };

  return useUfmFlowInputMachineNamespaceImplementation(namespaceId);
}
