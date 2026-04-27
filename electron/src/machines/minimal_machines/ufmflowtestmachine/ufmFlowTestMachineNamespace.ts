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
  sensor_error: z.boolean(),
});

export const stateEventSchema = eventSchema(stateEventDataSchema);
export type StateEvent = z.infer<typeof stateEventDataSchema>;

export type UfmFlowTestMachineNamespaceStore = {
  state: StateEvent | null;
};

export const createUfmFlowTestMachineNamespaceStore =
  (): StoreApi<UfmFlowTestMachineNamespaceStore> =>
    create<UfmFlowTestMachineNamespaceStore>(() => ({
      state: null,
    }));

export function ufmFlowTestMachineMessageHandler(
  store: StoreApi<UfmFlowTestMachineNamespaceStore>,
  throttledUpdater: ThrottledStoreUpdater<UfmFlowTestMachineNamespaceStore>,
): EventHandler {
  return (event: Event<any>) => {
    const updateStore = (
      updater: (
        state: UfmFlowTestMachineNamespaceStore,
      ) => UfmFlowTestMachineNamespaceStore,
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

const useUfmFlowTestMachineNamespaceImplementation =
  createNamespaceHookImplementation<UfmFlowTestMachineNamespaceStore>({
    createStore: createUfmFlowTestMachineNamespaceStore,
    createEventHandler: ufmFlowTestMachineMessageHandler,
  });

export function useUfmFlowTestMachineNamespace(
  machine_identification_unique: MachineIdentificationUnique,
): UfmFlowTestMachineNamespaceStore {
  const namespaceId: NamespaceId = {
    type: "machine",
    machine_identification_unique,
  };

  return useUfmFlowTestMachineNamespaceImplementation(namespaceId);
}
