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
  override_inputs: z.array(z.boolean()).length(8),
  stepper_target_speed: z.number(),
  stepper_enabled: z.boolean(),
  stepper_freq: z.number(),
  stepper_acc_freq: z.number(),
});

export const stateEventSchema = eventSchema(stateEventDataSchema);

export type StateEvent = z.infer<typeof stateEventDataSchema>;

export type BottlecapsTestMachineNamespaceStore = {
  state: StateEvent | null;
};

export const createBottlecapsTestMachineNamespaceStore =
  (): StoreApi<BottlecapsTestMachineNamespaceStore> =>
    create<BottlecapsTestMachineNamespaceStore>(() => ({
      state: null,
    }));

export function bottlecapsTestMachineMessageHandler(
  store: StoreApi<BottlecapsTestMachineNamespaceStore>,
  throttledUpdater: ThrottledStoreUpdater<BottlecapsTestMachineNamespaceStore>,
): EventHandler {
  return (event: Event<any>) => {
    const updateStore = (
      updater: (
        state: BottlecapsTestMachineNamespaceStore,
      ) => BottlecapsTestMachineNamespaceStore,
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

const useBottlecapsTestMachineNamespaceImplementation =
  createNamespaceHookImplementation<BottlecapsTestMachineNamespaceStore>({
    createStore: createBottlecapsTestMachineNamespaceStore,
    createEventHandler: bottlecapsTestMachineMessageHandler,
  });

export function useBottlecapsTestMachineNamespace(
  machine_identification_unique: MachineIdentificationUnique,
): BottlecapsTestMachineNamespaceStore {
  const namespaceId: NamespaceId = {
    type: "machine",
    machine_identification_unique,
  };

  return useBottlecapsTestMachineNamespaceImplementation(namespaceId);
}
