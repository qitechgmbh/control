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
  digital_input: z.array(z.boolean()).length(8),
  digital_output: z.array(z.boolean()).length(8),
});

export const stateEventSchema = eventSchema(stateEventDataSchema);

export type StateEvent = z.infer<typeof stateEventDataSchema>;

// ========== Store ==========
export type Wago8chDioTestMachineNamespaceStore = {
  state: StateEvent | null;
};

export const createWago8chDioTestMachineNamespaceStore =
  (): StoreApi<Wago8chDioTestMachineNamespaceStore> =>
    create<Wago8chDioTestMachineNamespaceStore>(() => ({
      state: null,
    }));

// ========== Message Handler ==========
export function wago8chDioTestMachineMessageHndler(
  store: StoreApi<Wago8chDioTestMachineNamespaceStore>,
  throttledUpdater: ThrottledStoreUpdater<Wago8chDioTestMachineNamespaceStore>,
): EventHandler {
  return (event: Event<any>) => {
    const updateStore = (
      updater: (
        state: Wago8chDioTestMachineNamespaceStore,
      ) => Wago8chDioTestMachineNamespaceStore,
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
const useWago8chDioTestMachineImplementation =
  createNamespaceHookImplementation<Wago8chDioTestMachineNamespaceStore>({
    createStore: createWago8chDioTestMachineNamespaceStore,
    createEventHandler: wago8chDioTestMachineMessageHndler,
  });

export function useWago8chDioTestMachineNamespace(
  machine_identification_unique: MachineIdentificationUnique,
): Wago8chDioTestMachineNamespaceStore {
  const namespaceId: NamespaceId = {
    type: "machine",
    machine_identification_unique,
  };

  return useWago8chDioTestMachineImplementation(namespaceId);
}
